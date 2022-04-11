#[macro_use] extern crate rocket;

use std::error::{Error};
use std::result::{Result};

//use reqwest;
use tokio::fs::File;
use tokio_stream::StreamExt;
use serde::{Serialize, Deserialize};
use serde_json;
use sublime_fuzzy;
use rocket::serde::json::Json as RocketJson;

use gamechooser_core as core;
use igdb_api_client::SReqwestTwitchAPIClient;

#[derive(Default, Serialize, Deserialize)]
pub struct SConfigFile {
    db_path: String,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
struct SOwnRecord {
    game_id: u32,
    storefront: String,
}

fn load_collection() -> Result<Vec<core::SCollectionGame>, String> {
    let cfg : SConfigFile = confy::load("gamechooser2_server").unwrap();
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
    let cfg : SConfigFile = confy::load("gamechooser2_server").unwrap();
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
    let cfg : SConfigFile = confy::load("gamechooser2_server").unwrap();
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
    let cfg : SConfigFile = confy::load("gamechooser2_server").unwrap();
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
    let cfg : SConfigFile = confy::load("gamechooser2_server").unwrap();

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
    let results = SReqwestTwitchAPIClient::search(&session, name).await?;
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

            //println!("Session {:?} had no valid game in collection!", session);
            let game = game_opt.ok_or(String::from("Server has bad data, won't be able to continue until it's fixed."))?;

            result.push(core::SSessionAndGameInfo{
                session: session.clone(),
                game_info: game.game_info,
            });
        }
    }

    Ok(RocketJson(result))
}

#[post("/get_randomizer_games", data = "<filter>")]
async fn get_randomizer_games(filter: RocketJson<core::SRandomizerFilter>) -> Result<RocketJson<core::SRandomizerList>, String> {
    let filter_inner = filter.into_inner();

    let collection_games = load_collection()?;
    let sessions = load_sessions()?;

    let mut active_session_game_ids = std::collections::HashSet::new();
    for session in sessions {
        if let core::ESessionState::Ongoing = session.state {
            active_session_game_ids.insert(session.game_internal_id);
        }
    }

    let mut result = Vec::with_capacity(collection_games.len());

    for game in collection_games {
        if !active_session_game_ids.contains(&game.internal_id) && filter_inner.game_passes(&game) {
            result.push(game);
        }
    }

    let mut indices = Vec::with_capacity(result.len());
    for i in 0..result.len() {
        indices.push(i);
    }

    use rand::seq::SliceRandom;
    indices.shuffle(&mut rand::thread_rng());

    Ok(RocketJson(core::SRandomizerList{
        games: result,
        shuffled_indices: indices,
    }))
}

#[post("/update_choose_state", data = "<games>")]
async fn update_choose_state(games: RocketJson<Vec<core::SCollectionGame>>) -> Result<(), String> {
    let games_inner = games.into_inner();
    let mut collection_games = load_collection()?;

    let mut input_idx = 0;
    let mut output_idx = 0;

    while input_idx < games_inner.len() && output_idx < collection_games.len() {
        if games_inner[input_idx].internal_id == collection_games[output_idx].internal_id {
            collection_games[output_idx].choose_state = games_inner[input_idx].choose_state;
            input_idx = input_idx + 1;
            output_idx = output_idx + 1;
        }
        else {
            output_idx = output_idx + 1;

            // -- EVERY game in games should be present in collection_games, and both vecs
            // -- should be strictly in order.
            if collection_games[output_idx].internal_id > games_inner[input_idx].internal_id {
                return Err(String::from("During update_choose_state, either games or collection_games as out of order!"));
            }
        }
    }

    save_collection(collection_games)?;

    Ok(())
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
            get_randomizer_games,
            update_choose_state,
        ])
}