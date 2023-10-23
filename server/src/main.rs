#[macro_use] extern crate rocket;

use std::collections::{HashMap};
use std::ops::{Deref, DerefMut};
use std::result::{Result};

//use reqwest;
use serde::{Serialize, Deserialize};
use serde_json;
use sublime_fuzzy;
use rocket::response::{Responder, Response};
use rocket::serde::json::Json as RocketJson;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;

use gamechooser_core as core;
use igdb_api_client::SReqwestTwitchAPIClient;

struct SData {
    serialized_db: core::EDatabase,
    game_igdb_id_to_internal_id: HashMap<u32, u32>,
    game_sessions_reverse_lookup: HashMap<u32, Vec<u32>>,
}

static MEMORY_DB : Lazy<RwLock<Result<SData, ()>>> = Lazy::new(|| RwLock::new(load_db()));

#[derive(Default, Serialize, Deserialize)]
pub struct SConfigFile {
    db_path: String,
    auth_secret: String,
    auth_pw: String,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
struct SOwnRecord {
    game_id: u32,
    storefront: String,
}

/*
#[derive(Reponder)
enum MyEnum {
   #[response(status = 500]
   InternalServerError
   #[response(status = 400, content_type = "json")]
   BadRequest(&'static str)
   ...
}
*/

#[allow(dead_code)]
enum EErrorResponse {
    DBError,
    ExternalAPIError(String),
    BadRequest(String),
    NotAuthenticated,
}

struct AuthenticatedUser {
}

impl<'r> Responder<'r, 'static> for EErrorResponse {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        match self {
            Self::DBError => {
                let body = "Server was unable to access database file.";
                Response::build()
                    .status(rocket::http::Status::InternalServerError)
                    .header(rocket::http::ContentType::Plain)
                    .sized_body(body.len(), std::io::Cursor::new(body))
                    .ok()
            },
            Self::ExternalAPIError(msg) => {
                Response::build()
                    .status(rocket::http::Status::InternalServerError)
                    .header(rocket::http::ContentType::Plain)
                    .sized_body(msg.len(), std::io::Cursor::new(msg))
                    .ok()
            },
            Self::BadRequest(msg) => {
                Response::build()
                    .status(rocket::http::Status::BadRequest)
                    .header(rocket::http::ContentType::Plain)
                    .sized_body(msg.len(), std::io::Cursor::new(msg))
                    .ok()
            },
            Self::NotAuthenticated => {
                let body = "You have not authenticated, please log in.";
                Response::build()
                    .status(rocket::http::Status::Unauthorized)
                    .header(rocket::http::ContentType::Plain)
                    .sized_body(body.len(), std::io::Cursor::new(body))
                    .ok()
            }
        }
    }
}

#[rocket::async_trait]
impl<'r> rocket::request::FromRequest<'r> for AuthenticatedUser {
    type Error = String;

    async fn from_request(_req: &'r rocket::request::Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        // -- accept anything always auth
        rocket::request::Outcome::Success(Self{})

        /*
        let cfg : SConfigFile = match confy::load("gamechooser2_server") {
            Ok(c) => c,
            Err(_) => {
                //return Err(String::from("Could not load config file"));
                return rocket::request::Outcome::Forward(());
            }
        };

        match req.cookies().get("auth_secret") {
            Some(secret) => if secret.value().eq(cfg.auth_secret.as_str()) {
                rocket::request::Outcome::Success(Self{})
            }
            else {
                rocket::request::Outcome::Forward(())
            },
            None => rocket::request::Outcome::Forward(()),
        }
        */
    }
}

fn refresh_db_acceleration(data: &mut SData) -> Result<(), ()> {
    data.game_igdb_id_to_internal_id.clear();
    data.game_sessions_reverse_lookup.clear();

    // -- generate additional data for SData
    for session in &data.serialized_db.sessions {
        if !data.game_sessions_reverse_lookup.contains_key(&session.game_internal_id) {
            data.game_sessions_reverse_lookup.insert(session.game_internal_id, Vec::new());
        }

        match data.game_sessions_reverse_lookup.get_mut(&session.game_internal_id) {
            Some(session_list) => session_list.push(session.internal_id),
            None => {
                eprintln!("Created entry in game_sessions_reverse_lookup but couldn't find it immediately after.");
                return Err(())
            }
        };
    }

    for game in &data.serialized_db.games {
        if let core::EGameInfo::IGDB(igdb_info) = &game.game_info {
            data.game_igdb_id_to_internal_id.insert(igdb_info.id, game.internal_id);
        }
    }

    Ok(())
}

fn load_db() -> Result<SData, ()> {
    let cfg : SConfigFile = confy::load("gamechooser2_server").unwrap();
    let mut path = std::path::PathBuf::new();
    path.push(cfg.db_path);
    path.push("database.json");

    println!("Loading DB from '{:?}'", path);

    // -- read existing collection
    let db : core::EDatabase = {
        if path.exists() {
            let file = match std::fs::File::open(path.clone()) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Failed to open {:?} with: {:?}", path, e);
                    return Err(());
                }
            };
            let reader = std::io::BufReader::new(file);

            // Read the JSON contents of the file as an instance of `User`.
            match serde_json::from_reader(reader) {
                Ok(g) => g,
                Err(e) => {
                    eprintln!("Failed to deserialize {:?} with: {:?}", path, e);
                    return Err(());
                }
            }
        }
        else {
            core::EDatabase::new()
        }
    };

    let updated_db = db.to_latest_version();

    let mut data = SData {
        serialized_db: updated_db,
        game_igdb_id_to_internal_id: HashMap::new(),
        game_sessions_reverse_lookup: HashMap::new(),
    };

    refresh_db_acceleration(&mut data)?;

    Ok(data)
}

fn save_db(data: &mut SData) -> Result<(), ()> {
    let cfg : SConfigFile = confy::load("gamechooser2_server").unwrap();
    let mut path = std::path::PathBuf::new();

    refresh_db_acceleration(data)?;

    path.push(cfg.db_path.clone());
    path.push("database.json");

    if path.exists() {
        let mut backup_path = std::path::PathBuf::new();
        backup_path.push(cfg.db_path);
        backup_path.push("bak");

        if !backup_path.exists() {
            if let Err(_) = std::fs::create_dir(backup_path.clone()) {
                eprintln!("Failed to back up DB before overwriting, aborted.");
                return Err(());
            }
        }

        let bak_file_name = format!("database_{}.json", chrono::offset::Utc::now().timestamp());
        backup_path.push(bak_file_name);

        if let Err(e) = std::fs::rename(path.clone(), backup_path) {
            eprintln!("Failed to delete database.json with: {:?}", e);
            return Err(());
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
            eprintln!("Failed to open database.json with: {:?}", e);
            return Err(());
        }
    };
    let writer = std::io::BufWriter::new(file);

    match serde_json::to_writer_pretty(writer, &data.serialized_db) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Failed to serialize database.json with: {:?}", e);
            return Err(());
        }
    };

    Ok(())
}

#[post("/search_igdb/<name>/<games_only>")]
async fn search_igdb(name: &str, games_only: bool) -> Result<RocketJson<Vec<core::SSearchIGDBResult>>, EErrorResponse> {
    let db_guard = MEMORY_DB.read().await;
    let db = db_guard.deref().as_ref().map_err(|_| EErrorResponse::DBError)?;

    let session = SReqwestTwitchAPIClient::new_session().await.map_err(|e| EErrorResponse::ExternalAPIError(e))?;
    let igdb_games = SReqwestTwitchAPIClient::search(&session, name, games_only).await.map_err(|e| EErrorResponse::ExternalAPIError(e))?;

    let mut results = Vec::with_capacity(igdb_games.len());
    for game in igdb_games {
        let igdb_id = match &game {
            core::EGameInfo::Custom(_) => 0,
            core::EGameInfo::IGDB(igdb_info) => igdb_info.id,
        };

        let in_collection = db.game_igdb_id_to_internal_id.contains_key(&igdb_id);

        results.push(core::SSearchIGDBResult{
            game_info: game,
            in_collection,
        });
    }

    Ok(RocketJson(results))
}

#[post("/add_game", data = "<game>")]
async fn add_game(game: RocketJson<core::SAddCollectionGame>, _user: AuthenticatedUser) -> Result<(), EErrorResponse> {
    let mut db_guard = MEMORY_DB.write().await;
    let mut db = db_guard.deref_mut().as_mut().map_err(|_| EErrorResponse::DBError)?;

    let mut max_id = 0;
    for collection_game in &db.serialized_db.games {
        max_id = std::cmp::max(max_id, collection_game.internal_id);
    }

    db.serialized_db.games.push(core::SCollectionGame::new(game.into_inner(), max_id + 1));

    save_db(&mut db).map_err(|_| EErrorResponse::DBError)?;

    Ok(())
}

#[post("/add_game", data = "<game>", rank = 2)]
#[allow(unused_variables)]
async fn add_game_no_auth(game: RocketJson<core::SAddCollectionGame>) -> Result<(), EErrorResponse> {
    return Err(EErrorResponse::NotAuthenticated);
}

#[post("/edit_game", data = "<game>")]
async fn edit_game(game: RocketJson<core::SCollectionGame>, _user: AuthenticatedUser) -> Result<(), EErrorResponse> {
    let mut db_guard = MEMORY_DB.write().await;
    let mut db = db_guard.deref_mut().as_mut().map_err(|_| EErrorResponse::DBError)?;

    let edit_internal_id = game.internal_id;

    for collection_game in &mut db.serialized_db.games {
        let internal_id = collection_game.internal_id;
        if internal_id == edit_internal_id {
            *collection_game = game.into_inner();
            break;
        }
    }

    save_db(&mut db).map_err(|_| EErrorResponse::DBError)?;

    Ok(())
}

#[post("/edit_game", data = "<game>", rank = 2)]
#[allow(unused_variables)]
async fn edit_game_no_auth(game: RocketJson<core::SAddCollectionGame>, _user: AuthenticatedUser) -> Result<(), EErrorResponse> {
    return Err(EErrorResponse::NotAuthenticated);
}

#[post("/get_recent_collection_games")]
async fn get_recent_collection_games() -> Result<RocketJson<Vec<core::SCollectionGame>>, EErrorResponse> {
    let db_guard = MEMORY_DB.read().await;
    let db = &db_guard.deref().as_ref().map_err(|_| EErrorResponse::DBError)?.serialized_db;

    let mut result = Vec::with_capacity(10);

    let mut count = 0;
    while count < 10 && db.games.len() > 0 {
        result.push(db.games[db.games.len() - 1 - count].clone());
        count += 1;
    }

    Ok(RocketJson(result))
}

#[post("/get_full_collection")]
async fn get_full_collection() -> Result<RocketJson<Vec<core::SCollectionGame>>, EErrorResponse> {
    let db_guard = MEMORY_DB.read().await;
    let db = &db_guard.deref().as_ref().map_err(|_| EErrorResponse::DBError)?.serialized_db;

    Ok(RocketJson(db.games.clone()))
}

#[post("/update_igdb_games")]
async fn update_igdb_games() -> Result<(), EErrorResponse> {
    let mut db_guard = MEMORY_DB.write().await;
    let db = db_guard.deref_mut().as_mut().map_err(|_| EErrorResponse::DBError)?;

    let mut games_with_sessions = std::collections::HashSet::with_capacity(db.serialized_db.games.len());
    for session in &db.serialized_db.sessions {
        games_with_sessions.insert(session.game_internal_id);
    }

    let today = chrono::offset::Local::now().naive_local().date();

    // -- we might falsley believe a game came out if it had a bad date, so
    // -- we still update for games that "released" in the last 6 months
    let mut six_months_ago = today;
    for _ in 0..(6*30) {
        six_months_ago = six_months_ago.pred();
    }

    let mut games_to_update = Vec::with_capacity(db.serialized_db.games.len());
    for (i, game) in db.serialized_db.games.iter().enumerate() {
        if let core::EGameInfo::IGDB(igdb_game_info) = &game.game_info {
            if games_with_sessions.contains(&game.internal_id) {
                continue;
            }

            let update = match igdb_game_info.cached_release_date {
                core::EReleaseDate::UnknownUnreleased => true,
                core::EReleaseDate::UnknownReleased => false,
                core::EReleaseDate::Known(d) => d >= six_months_ago,
            };

            if update {
                games_to_update.push(i);
            }
        }
    }

    let session = SReqwestTwitchAPIClient::new_session().await.map_err(|e| EErrorResponse::ExternalAPIError(e))?;

    for i in games_to_update {
        let game = &mut db.serialized_db.games[i];

        println!("Updating game \"{}\"", game.game_info.title());

        let mut new_game_info = None;
        if let core::EGameInfo::IGDB(igdb_info) = &mut game.game_info {
            let igdb_game_info = SReqwestTwitchAPIClient::get_game_info(&session, igdb_info.id).await.map_err(|e| EErrorResponse::ExternalAPIError(e))?;
            new_game_info = Some(igdb_game_info);
        }

        if let Some(gi) = new_game_info {
            println!("Updated as \"{}\"", gi.title());
            game.game_info = gi;
        }

        // -- just hard sleep here to avoid using up our API request budget
        tokio::time::sleep(std::time::Duration::from_secs_f32(0.5)).await;
    }

    save_db(db).map_err(|_| EErrorResponse::DBError)?;

    Ok(())
}

#[post("/search_collection/<query>")]
async fn search_collection(query: &str) -> Result<RocketJson<Vec<core::SCollectionGame>>, EErrorResponse> {
    let db_guard = MEMORY_DB.read().await;
    let db = &db_guard.deref().as_ref().map_err(|_| EErrorResponse::DBError)?.serialized_db;

    #[derive(Debug)]
    struct SScore {
        idx: usize,
        score: isize,
    }
    let mut scores = Vec::with_capacity(db.games.len());

    for (idx, game) in db.games.iter().enumerate() {
        if let Some(m) = sublime_fuzzy::best_match(query, game.game_info.title()) {
            scores.push(SScore{
                idx,
                score: m.score(),
            });
        }
    }

    scores.sort_by(|a, b| b.score.cmp(&a.score));

    let mut result = Vec::with_capacity(20);
    for i in 0..std::cmp::min(20, scores.len()) {
        result.push(db.games[scores[i].idx].clone());
    }

    Ok(RocketJson(result))
}

#[post("/start_session/<game_internal_id>")]
async fn start_session(game_internal_id: u32, _user: AuthenticatedUser) -> Result<(), EErrorResponse> {
    let mut db_guard = MEMORY_DB.write().await;
    let mut db = db_guard.deref_mut().as_mut().map_err(|_| EErrorResponse::DBError)?;

    for session in &db.serialized_db.sessions {
        if matches!(session.state, core::ESessionState::Ongoing) && session.game_internal_id == game_internal_id {
            return Err(EErrorResponse::BadRequest(format!("There is already a session started for the game with ID {}", game_internal_id)));
        }
    }

    let mut found_game = false;
    for game in &db.serialized_db.games {
        if game.internal_id == game_internal_id {
            found_game = true;
            break;
        }
    }

    if !found_game {
        return Err(EErrorResponse::BadRequest(format!("Could not find a game with internal_id {} to start session for.", game_internal_id)));
    }

    let mut max_id = 0;
    for session in &db.serialized_db.sessions {
        max_id = std::cmp::max(max_id, session.internal_id);
    }

    db.serialized_db.sessions.push(core::SSession::new(max_id + 1, game_internal_id));

    save_db(&mut db).map_err(|_| EErrorResponse::DBError)?;

    Ok(())
}

#[post("/start_session/<game_internal_id>", rank = 2)]
#[allow(unused_variables)]
async fn start_session_no_auth(game_internal_id: u32) -> Result<(), EErrorResponse> {
    return Err(EErrorResponse::NotAuthenticated);
}

#[post("/finish_session/<session_internal_id>/<memorable>/<retire>/<set_ignore_passes>")]
async fn finish_session(session_internal_id: u32, memorable: bool, retire: bool, set_ignore_passes: bool, _user: AuthenticatedUser) -> Result<(), EErrorResponse> {
    let mut db_guard = MEMORY_DB.write().await;
    let mut db = db_guard.deref_mut().as_mut().map_err(|_| EErrorResponse::DBError)?;

    let mut game_id_opt = None;
    for s in &mut db.serialized_db.sessions {
        if s.internal_id == session_internal_id {
            s.finish(memorable);
            game_id_opt = Some(s.game_internal_id);
            break;
        }
    }

    if game_id_opt.is_none() {
        return Err(EErrorResponse::BadRequest(String::from("Could not find session with matching internal_id to finish.")));
    }

    let game_id = game_id_opt.expect("checked above");

    for game in &mut db.serialized_db.games {
        if game.internal_id == game_id {
            if retire {
                game.choose_state.retire();
            }
            if set_ignore_passes {
                game.choose_state.set_ignore_passes();
            }
            game.choose_state.push();

            break;
        }
    }

    save_db(&mut db).map_err(|_| EErrorResponse::DBError)?;

    Ok(())
}

#[post("/finish_session/<session_internal_id>/<memorable>", rank = 2)]
#[allow(unused_variables)]
async fn finish_session_no_auth(session_internal_id: u32, memorable: bool) -> Result<(), EErrorResponse> {
    return Err(EErrorResponse::NotAuthenticated);
}

#[post("/get_sessions", data = "<filter>")]
async fn get_sessions(filter: RocketJson<core::SSessionFilter>, _user: AuthenticatedUser) -> Result<RocketJson<Vec<core::SSessionAndCollectionGame>>, EErrorResponse> {
    let db_guard = MEMORY_DB.read().await;
    let db = &db_guard.deref().as_ref().map_err(|_| EErrorResponse::DBError)?.serialized_db;

    let mut result = Vec::with_capacity(10);

    for session in &db.sessions {
        if filter.session_passes(&session) {

            // -- find the game
            let mut game_opt = None;
            for temp_game in &db.games {
                if temp_game.internal_id == session.game_internal_id {
                    game_opt = Some(temp_game.clone());
                    break;
                }
            }

            //println!("Session {:?} had no valid game in collection!", session);
            let game = game_opt.ok_or(EErrorResponse::BadRequest(String::from("Server has bad data, won't be able to continue until it's fixed.")))?;

            result.push(core::SSessionAndCollectionGame{
                session: session.clone(),
                collection_game: game,
            });
        }
    }

    Ok(RocketJson(result))
}

#[post("/get_sessions", data = "<filter>", rank = 2)]
#[allow(unused_variables)]
async fn get_sessions_no_auth(filter: RocketJson<core::SSessionFilter>) -> Result<RocketJson<Vec<core::SSessionAndCollectionGame>>, EErrorResponse> {
    return Err(EErrorResponse::NotAuthenticated);
}

#[post("/get_randomizer_games", data = "<filter>")]
async fn get_randomizer_games(filter: RocketJson<core::ERandomizerFilter>) -> Result<RocketJson<core::SRandomizerList>, EErrorResponse> {
    let filter_inner = filter.into_inner();

    let db_guard = MEMORY_DB.read().await;
    let db = &db_guard.deref().as_ref().map_err(|_| EErrorResponse::DBError)?.serialized_db;

    let mut active_session_game_ids = std::collections::HashSet::new();
    let mut all_session_game_ids = std::collections::HashSet::new();
    for session in &db.sessions {
        all_session_game_ids.insert(session.game_internal_id);
        if let core::ESessionState::Ongoing = session.state {
            active_session_game_ids.insert(session.game_internal_id);
        }
    }

    let mut result = Vec::with_capacity(db.games.len());

    for game in &db.games {
        if !active_session_game_ids.contains(&game.internal_id) && filter_inner.game_passes(&game, all_session_game_ids.contains(&game.internal_id)) {
            //println!("Passed game with: {:?}", game.choose_state);
            result.push(game.clone());
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
async fn update_choose_state(games: RocketJson<Vec<core::SCollectionGame>>, _user: AuthenticatedUser) -> Result<(), EErrorResponse> {
    let mut games_inner = games.into_inner();
    let mut db_guard = MEMORY_DB.write().await;
    let db = db_guard.deref_mut().as_mut().map_err(|_| EErrorResponse::DBError)?;

    let mut input_idx = 0;
    let mut output_idx = 0;

    games_inner.sort_unstable_by(|a, b| a.internal_id.cmp(&b.internal_id));

    while input_idx < games_inner.len() && output_idx < db.serialized_db.games.len() {
        if games_inner[input_idx].internal_id == db.serialized_db.games[output_idx].internal_id {
            db.serialized_db.games[output_idx].choose_state = games_inner[input_idx].choose_state;
            input_idx = input_idx + 1;
            output_idx = output_idx + 1;
        }
        else {
            output_idx = output_idx + 1;

            // -- EVERY game in games should be present in collection_games, and both vecs
            // -- should be strictly in order.
            if db.serialized_db.games[output_idx].internal_id > games_inner[input_idx].internal_id {
                return Err(EErrorResponse::BadRequest(String::from("During update_choose_state, either games or collection_games as out of order!")));
            }
        }
    }

    save_db(db).map_err(|_| EErrorResponse::DBError)?;

    Ok(())
}

#[post("/update_choose_state", data = "<games>", rank = 2)]
#[allow(unused_variables)]
async fn update_choose_state_no_auth(games: RocketJson<Vec<core::SCollectionGame>>) -> Result<(), EErrorResponse> {
    return Err(EErrorResponse::NotAuthenticated);
}

#[post("/reset_choose_state/<game_internal_id>")]
async fn reset_choose_state(game_internal_id: u32, _user: AuthenticatedUser) -> Result<(), EErrorResponse> {
    let mut db_guard = MEMORY_DB.write().await;
    let db = db_guard.deref_mut().as_mut().map_err(|_| EErrorResponse::DBError)?;

    for game in &mut db.serialized_db.games {
        if game.internal_id == game_internal_id {
            game.choose_state.reset();
            save_db(db).map_err(|_| EErrorResponse::DBError)?;
            return Ok(());
        }
    }

    Err(EErrorResponse::BadRequest(format!("Did not find game with internal_id {} to reset choose_state on", game_internal_id)))
}

#[post("/reset_choose_state/<game_internal_id>", rank = 2)]
#[allow(unused_variables)]
async fn reset_choose_state_no_auth(game_internal_id: u32) -> Result<(), EErrorResponse> {
    return Err(EErrorResponse::NotAuthenticated);
}

#[post("/simple_stats")]
async fn simple_stats() -> Result<RocketJson<core::SSimpleStats>, EErrorResponse> {
    let db_guard = MEMORY_DB.read().await;
    let data = db_guard.deref().as_ref().map_err(|_| EErrorResponse::DBError)?;

    let mut stats = core::SSimpleStats{
        total_collection_size: 0,

        collection_released: 0,
        collection_owned: 0,
        collection_selectable: 0,
        collection_retired: 0,
        collection_passed_many_times: 0,
        collection_cooldown: 0,

        collection_played_before: 0,
        collection_couch_playable_tag: 0,
        collection_japanese_practice_tag: 0,
        collection_portable_playable_tag: 0,

        selectable_owned: 0,
        selectable_played_before: 0,
        selectable_couch_playable_tag: 0,
        selectable_japanese_practice_tag: 0,
        selectable_portable_playable_tag: 0,
    };

    let today = chrono::offset::Local::now().naive_local().date();

    fn inc(stat: &mut u32) {
        *stat = *stat + 1;
    }

    let filter = core::ERandomizerFilter::default();

    for game in &data.serialized_db.games {
        inc(&mut stats.total_collection_size);

        let selectable = filter.game_passes(&game, false);

        if selectable {
            inc(&mut stats.collection_selectable);
        }

        if game.game_info.released() {
            inc(&mut stats.collection_released);
        }

        if game.choose_state.retired {
            inc(&mut stats.collection_retired);
        }
        else if game.choose_state.passes > core::SGameChooseAlgFilter::max_passes() {
            inc(&mut stats.collection_passed_many_times);
        }
        else if game.choose_state.next_valid_proposal_date > today {
            inc(&mut stats.collection_cooldown);
        }

        if game.custom_info.own.owned() {
            inc(&mut stats.collection_owned);

            if selectable {
                inc(&mut stats.selectable_owned);
            }
        }

        if let Some(session_list) = data.game_sessions_reverse_lookup.get(&game.internal_id) {
            if session_list.len() > 0 {
                inc(&mut stats.collection_played_before);

                if selectable {
                    inc(&mut stats.selectable_played_before);
                }
            }
        }

        if game.custom_info.tags.couch_playable {
            inc(&mut stats.collection_couch_playable_tag);

            if selectable {
                inc(&mut stats.selectable_couch_playable_tag);
            }
        }

        if game.custom_info.tags.japanese_practice {
            inc(&mut stats.collection_japanese_practice_tag);

            if selectable {
                inc(&mut stats.selectable_japanese_practice_tag);
            }
        }

        if game.custom_info.tags.portable_playable {
            inc(&mut stats.collection_portable_playable_tag);

            if selectable {
                inc(&mut stats.selectable_portable_playable_tag);
            }
        }
    }

    Ok(RocketJson(stats))
}

#[post("/check_logged_in")]
async fn check_logged_in(_user: AuthenticatedUser) -> Result<(), EErrorResponse> {
    Ok(())
}

#[post("/check_logged_in", rank = 2)]
async fn check_logged_in_no_auth() -> Result<(), EErrorResponse> {
    Err(EErrorResponse::NotAuthenticated)
}

#[post("/login/<secret>")]
async fn login(secret: &str, cookies: &rocket::http::CookieJar<'_>) -> Result<(), EErrorResponse> {
    let cfg : SConfigFile = match confy::load("gamechooser2_server") {
        Ok(c) => c,
        Err(_) => {
            return Err(EErrorResponse::DBError);
        }
    };

    if secret.eq(cfg.auth_pw.as_str()) {
        cookies.add(rocket::http::Cookie::new("auth_secret", cfg.auth_secret));

        return Ok(());
    }

    return Err(EErrorResponse::BadRequest(String::from("Incorrect secret")));
}

#[launch]
fn rocket() -> _ {
    // -- $$$FRK(TODO): verify we have valid config file, all values present

    rocket::build()
        .mount("/static", rocket::fs::FileServer::from("../client/served_files"))
        .mount("/", routes![
            check_logged_in,
            check_logged_in_no_auth,
            login,
            search_igdb,
            add_game,
            add_game_no_auth,
            edit_game,
            edit_game_no_auth,
            get_recent_collection_games,
            get_full_collection,
            update_igdb_games,
            search_collection,
            start_session,
            start_session_no_auth,
            finish_session,
            finish_session_no_auth,
            get_sessions,
            get_sessions_no_auth,
            get_randomizer_games,
            update_choose_state,
            update_choose_state_no_auth,
            reset_choose_state,
            reset_choose_state_no_auth,
            simple_stats,
        ])
}