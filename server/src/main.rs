#[macro_use] extern crate rocket;

use std::error::{Error};
use std::result::{Result};

//use reqwest;
use tokio::fs::File;
use tokio_stream::StreamExt;
use serde::{Serialize, Deserialize};
use serde::de::{DeserializeOwned};
use serde_json;
use sublime_fuzzy;
use rocket::serde::json::Json as RocketJson;

use gamechooser_core as core;

#[derive(Debug, Serialize, Clone)]
pub struct STwitchOauthTokenRequest {
    pub client_id: String,
    pub client_secret: String,
    pub grant_type: &'static str,
}

#[derive(Clone, Debug, Deserialize)]
pub struct STwitchOauthTokenResponse {
    pub access_token: String,
    pub expires_in: u32,
    pub token_type: String,
}

pub struct STwitchAPIRequestBuilder {
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

impl STwitchAPIRequestBuilder {
    pub fn new() -> Self {
        Self {
            url: String::new(),
            headers: Vec::new(),
            body: None,
        }
    }

    pub fn url(mut self, url: &str) -> Self {
        self.url = url.to_string();
        self
    }

    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_string(), value.to_string()));
        self
    }

    pub fn body(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());
        self
    }
}

pub trait TConfigStore {
    fn get_twitch_client_id(&self) -> Option<String>;
    fn get_twitch_client_secret(&self) -> Option<String>;
    fn save_twitch_client(&self, client_id: &str, client_secret: &str);
}

#[derive(Clone)]
struct SReqwestTwitchAPISession {
    client: reqwest::Client,
    twitch_client_id: String,
    token_info: Option<STwitchOauthTokenResponse>,
}

struct SReqwestTwitchAPIClient {
}

#[derive(Default, Serialize, Deserialize)]
struct SConfigFile {
    twitch_client_id: String,
    twitch_client_secret: String,
    db_path: String,
}

impl SReqwestTwitchAPIClient {
    async fn new_session() -> Result<SReqwestTwitchAPISession, String> {
        let cfg : SConfigFile = confy::load("gamechooser2_cli_client").unwrap();

        let params = STwitchOauthTokenRequest{
            client_id: cfg.twitch_client_id,
            client_secret: cfg.twitch_client_secret,
            grant_type: "client_credentials",
        };

        Self::init(params).await
    }

    fn prepare_request(session: &SReqwestTwitchAPISession, rb: STwitchAPIRequestBuilder) -> reqwest::RequestBuilder {
        let mut request = session.client.post(rb.url);

        if let Some(b) = rb.body {
            request = request.body(b);
        }

        for (hname, hvalue) in rb.headers {
            request = request.header(hname.as_str(), hvalue.as_str());
        }

        request
    }

    async fn init(params: STwitchOauthTokenRequest) -> Result<SReqwestTwitchAPISession, String> {
        let client = reqwest::Client::new();

        let res = client.post("https://id.twitch.tv/oauth2/token")
            .form(&params)
            .send()
            .await;

        match res {
            Ok(res_) => {
                let resp : STwitchOauthTokenResponse = res_.json().await.unwrap();
                println!("{:?}", resp);
                Ok(SReqwestTwitchAPISession{
                    client,
                    twitch_client_id: params.client_id,
                    token_info: Some(resp),
                })
            },
            Err(e_) => Err(e_.to_string()),
        }
    }

    async fn post_interp_json<T: DeserializeOwned>(session: SReqwestTwitchAPISession, rb: STwitchAPIRequestBuilder) -> Result<T, Box<dyn Error>> {
        let req = Self::prepare_request(&session, rb);
        let resp = req.send().await?;
        Ok(resp.json().await?)
    }

    #[allow(dead_code)]
    async fn post_text(session: SReqwestTwitchAPISession, rb: STwitchAPIRequestBuilder) -> Result<String, String> {
        let req = Self::prepare_request(&session, rb);
        let resp = req.send().await.unwrap();
        Ok(resp.text().await.unwrap())
    }

    fn access_token(session: &SReqwestTwitchAPISession) -> &str {
        session.token_info.as_ref().unwrap().access_token.as_str()
    }
}

#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
pub struct SOwnRecord {
    game_id: u32,
    storefront: String,
}

fn load_collection() -> Result<Vec<core::SCollectionGame>, String> {
    let cfg : SConfigFile = confy::load("gamechooser2_cli_client").unwrap();
    let mut path = std::path::PathBuf::new();
    path.push(cfg.db_path);
    path.push("collection.json");

    // -- read existing collection
    let collection_games : Vec<core::SCollectionGame> = {
        if path.exists() {
            let file = match std::fs::File::open(path.clone()) {
                Ok(f) => f,
                Err(e) => {
                    println!("Failed to open collection.json with: {:?}", e);
                    return Err(String::from("Server had local file issues."));
                }
            };
            let reader = std::io::BufReader::new(file);

            // Read the JSON contents of the file as an instance of `User`.
            match serde_json::from_reader(reader) {
                Ok(g) => g,
                Err(e) => {
                    println!("Failed to deserialize collection.json with: {:?}", e);
                    return Err(String::from("Server had local file issues."));
                }
            }
        }
        else {
            Vec::new()
        }
    };

    Ok(collection_games)
}

fn load_sessions() -> Result<Vec<core::SSession>, String> {
    let cfg : SConfigFile = confy::load("gamechooser2_cli_client").unwrap();
    let mut path = std::path::PathBuf::new();
    path.push(cfg.db_path);
    path.push("sessions.json");

    let sessions : Vec<core::SSession> = {
        if path.exists() {
            let file = match std::fs::File::open(path.clone()) {
                Ok(f) => f,
                Err(e) => {
                    println!("Failed to open sessions.json with: {:?}", e);
                    return Err(String::from("Server had local file issues."));
                }
            };
            let reader = std::io::BufReader::new(file);

            match serde_json::from_reader(reader) {
                Ok(g) => g,
                Err(e) => {
                    println!("Failed to deserialize sessions.json with: {:?}", e);
                    return Err(String::from("Server had local file issues."));
                }
            }
        }
        else {
            Vec::new()
        }
    };

    Ok(sessions)
}

fn save_collection(collection_games: Vec<core::SCollectionGame>) -> Result<(), String> {
    let cfg : SConfigFile = confy::load("gamechooser2_cli_client").unwrap();
    let mut path = std::path::PathBuf::new();
    path.push(cfg.db_path);
    path.push("collection.json");

    if path.exists() {
        if let Err(e) = std::fs::remove_file(path.clone()) {
            println!("Failed to delete collection.json with: {:?}", e);
            return Err(String::from("Server had local file issues."));
        }
    }

    let open_options = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .append(true)
        .open(path);

    let file = match open_options {
        Ok(f) => f,
        Err(e) => {
            println!("Failed to open collection.json with: {:?}", e);
            return Err(String::from("Server had local file issues."));
        }
    };
    let writer = std::io::BufWriter::new(file);

    match serde_json::to_writer_pretty(writer, &collection_games) {
        Ok(_) => {},
        Err(e) => {
            println!("Failed to serialize collection.json with: {:?}", e);
            return Err(String::from("Server had local file issues."));
        }
    };

    Ok(())
}

fn save_sessions(sessions: Vec<core::SSession>) -> Result<(), String> {
    let cfg : SConfigFile = confy::load("gamechooser2_cli_client").unwrap();
    let mut path = std::path::PathBuf::new();
    path.push(cfg.db_path);
    path.push("sessions.json");

    if path.exists() {
        if let Err(e) = std::fs::remove_file(path.clone()) {
            println!("Failed to delete sessions.json with: {:?}", e);
            return Err(String::from("Server had local file issues."));
        }
    }

    let open_options = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .append(true)
        .open(path);

    let file = match open_options {
        Ok(f) => f,
        Err(e) => {
            println!("Failed to open sessions.json with: {:?}", e);
            return Err(String::from("Server had local file issues."));
        }
    };
    let writer = std::io::BufWriter::new(file);

    match serde_json::to_writer_pretty(writer, &sessions) {
        Ok(_) => {},
        Err(e) => {
            println!("Failed to serialize sessions.json with: {:?}", e);
            return Err(String::from("Server had local file issues."));
        }
    };

    Ok(())
}

async fn test_csv() -> Result<String, Box<dyn Error>> {
    let cfg : SConfigFile = confy::load("gamechooser2_cli_client").unwrap();

    let mut path = std::path::PathBuf::new();
    path.push(cfg.db_path);
    path.push("_own.csv");

    let mut rdr = csv_async::AsyncDeserializer::from_reader(
        File::open(path).await?
    );

    let mut result = String::new();

    let mut records = rdr.deserialize::<SOwnRecord>();
    while let Some(record) = records.next().await {
        let record = record?;
        result.push_str(record.storefront.as_str());
    }
    Ok(result)
}

#[post("/test")]
async fn test() -> Result<String, String> {
    //test_twitch_api().await
    match test_csv().await {
        Ok(s) => Ok(s),
        Err(e) => Err(e.to_string())
    }
}

#[post("/search_igdb/<name>")]
async fn search_igdb(name: &str) -> Result<RocketJson<Vec<core::SGameInfo>>, String> {
    let session = SReqwestTwitchAPIClient::new_session().await?;

    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct SIGDBSearchResultCover {
        id: u32,
        image_id: String,
    }
    #[derive(Deserialize)]
    struct SIGDBSearchResult {
        id: u32,
        name: String,
        first_release_date: Option<i64>,
        cover: Option<SIGDBSearchResultCover>,
    }

    let search_results : Vec<SIGDBSearchResult> = {
        let body = format!("search \"{}\"; fields name,first_release_date,cover.image_id;", name);

        /*
        Should be equivalent to:
        curl -d "search \"halo\"; fields name,first_release_date,cover.url;" -H 'Client-ID: <ID>' -H 'Authorization: Bearer <BEARER>' -H 'Accept: application/json' https://api.igdb.com/v4/games
        */

        let request = STwitchAPIRequestBuilder::new()
            .url("https://api.igdb.com/v4/games/")
            .header("Client-ID", session.twitch_client_id.as_str())
            .header("Authorization", format!("Bearer {}", SReqwestTwitchAPIClient::access_token(&session)).as_str())
            .header("Accept", "application/json")
            .body(body.as_str());

        match SReqwestTwitchAPIClient::post_interp_json::<Vec<SIGDBSearchResult>>(session.clone(), request).await {
            Ok(res) => Ok(res),
            Err(e) => Err(format!("Failed with error {:?}", e)),
        }
    }?;

    fn timestamp_to_chrono(ts: i64) -> chrono::naive::NaiveDate {
        chrono::naive::NaiveDateTime::from_timestamp(ts, 0).date()
    }
    fn extract_cover_url(cover: SIGDBSearchResultCover) -> String {
        format!("https://images.igdb.com/igdb/image/upload/t_cover_small/{}.jpg", cover.image_id)
    }

    let mut results = Vec::with_capacity(search_results.len());
    for search_res in search_results {
        results.push(core::SGameInfo::new_igdb(
            search_res.name,
            search_res.first_release_date.map(timestamp_to_chrono),
            search_res.id,
            search_res.cover.map(extract_cover_url),
        ));
    }

    Ok(RocketJson(results))
}

#[post("/add_game", data = "<game>")]
async fn add_game(game: RocketJson<core::SAddCollectionGame>) -> Result<(), String> {
    let mut collection_games = load_collection()?;

    let mut max_id = 0;
    for collection_game in &collection_games {
        max_id = std::cmp::max(max_id, collection_game.internal_id);
    }

    collection_games.push(core::SCollectionGame::new(game.into_inner(), max_id + 1));

    save_collection(collection_games)?;

    Ok(())
}

#[post("/edit_game", data = "<game>")]
async fn edit_game(game: RocketJson<core::SCollectionGame>) -> Result<(), String> {
    let mut collection_games = load_collection()?;

    let edit_internal_id = game.internal_id;

    for collection_game in &mut collection_games {
        let internal_id = collection_game.internal_id;
        if internal_id == edit_internal_id {
            *collection_game = game.into_inner();
            break;
        }
    }

    save_collection(collection_games)?;

    Ok(())
}

#[post("/get_recent_collection_games")]
async fn get_recent_collection_games() -> Result<RocketJson<Vec<core::SCollectionGame>>, String> {
    let mut collection_games = load_collection()?;

    let mut result = Vec::with_capacity(10);

    let mut count = 0;
    while count < 10 && collection_games.len() > 0 {
        result.push(collection_games.pop().expect("len checked above"));
        count += 1;
    }

    Ok(RocketJson(result))
}

#[post("/search_collection/<query>")]
async fn search_collection(query: &str) -> Result<RocketJson<Vec<core::SCollectionGame>>, String> {
    let collection_games = load_collection()?;

    struct SScore {
        idx: usize,
        score: isize,
    }
    let mut scores = Vec::with_capacity(collection_games.len());

    for (idx, game) in collection_games.iter().enumerate() {
        if let Some(m) = sublime_fuzzy::best_match(query, game.game_info.title()) {
            scores.push(SScore{
                idx,
                score: m.score(),
            });
        }
    }

    scores.sort_by(|a, b| a.score.cmp(&b.score));
    assert!(scores.len() < 2 || scores[0].score > scores[1].score);

    let mut result = Vec::with_capacity(10);
    for i in 0..std::cmp::min(10, scores.len()) {
        result.push(collection_games[scores[i].idx].clone());
    }

    Ok(RocketJson(result))
}

#[post("/start_session/<game_internal_id>")]
async fn start_session(game_internal_id: u32) -> Result<(), String> {
    let collection_games = load_collection()?;

    let mut found_game = false;
    for game in &collection_games {
        if game.internal_id == game_internal_id {
            found_game = true;
            break;
        }
    }

    if !found_game {
        return Err(String::from("Could not find a game with matching internal_id to start session for."))
    }

    let mut sessions = load_sessions()?;

    let mut max_id = 0;
    for session in &sessions {
        max_id = std::cmp::max(max_id, session.internal_id);
    }

    sessions.push(core::SSession::new(max_id + 1, game_internal_id));

    save_sessions(sessions)?;

    Ok(())
}

#[post("/finish_session/<session_internal_id>/<memorable>")]
async fn finish_session(session_internal_id: u32, memorable: bool) -> Result<(), String> {
    let mut sessions = load_sessions()?;

    let mut found_session = false;
    for s in &mut sessions {
        if s.internal_id == session_internal_id {
            s.finish(memorable);
            found_session = true;
            break;
        }
    }

    if !found_session {
        return Err(String::from("Could not find session with matching internal_id to finish."))
    }

    save_sessions(sessions)?;

    Ok(())
}

#[post("/get_active_sessions")]
async fn get_active_sessions() -> Result<RocketJson<Vec<core::SSessionAndGameInfo>>, String> {
    let games = load_collection()?;
    let sessions = load_sessions()?;

    let mut result = Vec::with_capacity(10);

    for session in &sessions {
        if let core::ESessionState::Ongoing = session.state {

            // -- find the game
            let mut game_opt = None;
            for temp_game in &games {
                if temp_game.internal_id == session.game_internal_id {
                    game_opt = Some(temp_game.clone());
                    break;
                }
            }

            println!("Session {:?} had no valid game in collection!", session);
            let game = game_opt.ok_or(String::from("Server has bad data, won't be able to continue until it's fixed."))?;

            result.push(core::SSessionAndGameInfo{
                session: session.clone(),
                game_info: game.game_info,
            });
        }
    }

    Ok(RocketJson(result))
}

#[launch]
fn rocket() -> _ {
    // -- $$$FRK(TODO): verify we have valid config file, all values present

    rocket::build()
        .mount("/static", rocket::fs::FileServer::from("../client/served_files"))
        .mount("/", routes![
            test,
            search_igdb,
            add_game,
            edit_game,
            get_recent_collection_games,
            search_collection,
            start_session,
            finish_session,
            get_active_sessions,
        ])
}