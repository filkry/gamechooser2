#[macro_use] extern crate rocket;

use std::result::{Result};

//use reqwest;
use serde::{Serialize, Deserialize};
use serde_json;
use sublime_fuzzy;
use rocket::response::{Responder, Response};
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

enum EErrorResponse {
    DBError,
    BadRequest(String),
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
            Self::BadRequest(msg) => {
                Response::build()
                    .status(rocket::http::Status::BadRequest)
                    .header(rocket::http::ContentType::Plain)
                    .sized_body(msg.len(), std::io::Cursor::new(msg))
                    .ok()
            }
        }
    }
}

fn load_db() -> Result<core::EDatabase, ()> {
    let cfg : SConfigFile = confy::load("gamechooser2_server").unwrap();
    let mut path = std::path::PathBuf::new();
    path.push(cfg.db_path);
    path.push("database.json");

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

    Ok(updated_db)
}

fn save_db(db: core::EDatabase) -> Result<(), ()> {
    let cfg : SConfigFile = confy::load("gamechooser2_server").unwrap();
    let mut path = std::path::PathBuf::new();
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

    match serde_json::to_writer_pretty(writer, &db) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Failed to serialize database.json with: {:?}", e);
            return Err(());
        }
    };

    Ok(())
}

#[post("/search_igdb/<name>/<games_only>")]
async fn search_igdb(name: &str, games_only: bool) -> Result<RocketJson<Vec<core::SGameInfo>>, String> {
    let session = SReqwestTwitchAPIClient::new_session().await?;
    let results = SReqwestTwitchAPIClient::search(&session, name, games_only).await?;
    Ok(RocketJson(results))
}

#[post("/add_game", data = "<game>")]
async fn add_game(game: RocketJson<core::SAddCollectionGame>) -> Result<(), EErrorResponse> {
    let mut db = load_db().map_err(|_| EErrorResponse::DBError)?;

    let mut max_id = 0;
    for collection_game in &db.games {
        max_id = std::cmp::max(max_id, collection_game.internal_id);
    }

    db.games.push(core::SCollectionGame::new(game.into_inner(), max_id + 1));

    save_db(db).map_err(|_| EErrorResponse::DBError)?;

    Ok(())
}

#[post("/edit_game", data = "<game>")]
async fn edit_game(game: RocketJson<core::SCollectionGame>) -> Result<(), EErrorResponse> {
    let mut db = load_db().map_err(|_| EErrorResponse::DBError)?;

    let edit_internal_id = game.internal_id;

    for collection_game in &mut db.games {
        let internal_id = collection_game.internal_id;
        if internal_id == edit_internal_id {
            *collection_game = game.into_inner();
            break;
        }
    }

    save_db(db).map_err(|_| EErrorResponse::DBError)?;

    Ok(())
}

#[post("/get_recent_collection_games")]
async fn get_recent_collection_games() -> Result<RocketJson<Vec<core::SCollectionGame>>, EErrorResponse> {
    let mut db = load_db().map_err(|_| EErrorResponse::DBError)?;

    let mut result = Vec::with_capacity(10);

    let mut count = 0;
    while count < 10 && db.games.len() > 0 {
        result.push(db.games.pop().expect("len checked above"));
        count += 1;
    }

    Ok(RocketJson(result))
}

#[post("/search_collection/<query>")]
async fn search_collection(query: &str) -> Result<RocketJson<Vec<core::SCollectionGame>>, EErrorResponse> {
    let db = load_db().map_err(|_| EErrorResponse::DBError)?;

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

    let mut result = Vec::with_capacity(10);
    for i in 0..std::cmp::min(10, scores.len()) {
        result.push(db.games[scores[i].idx].clone());
    }

    Ok(RocketJson(result))
}

#[post("/start_session/<game_internal_id>")]
async fn start_session(game_internal_id: u32) -> Result<(), EErrorResponse> {
    // -- $$$FRK(TODO): Need a custom responder I think to handle different error codes from the same routine?
    let mut db = load_db().map_err(|_| EErrorResponse::DBError)?;

    for session in &db.sessions {
        if session.game_internal_id == game_internal_id {
            return Err(EErrorResponse::BadRequest(format!("There is already a session started for the game with ID {}", game_internal_id)));
        }
    }

    let mut found_game = false;
    for game in &db.games {
        if game.internal_id == game_internal_id {
            found_game = true;
            break;
        }
    }

    if !found_game {
        return Err(EErrorResponse::BadRequest(format!("Could not find a game with internal_id {} to start session for.", game_internal_id)));
    }

    let mut max_id = 0;
    for session in &db.sessions {
        max_id = std::cmp::max(max_id, session.internal_id);
    }

    db.sessions.push(core::SSession::new(max_id + 1, game_internal_id));

    save_db(db).map_err(|_| EErrorResponse::DBError)?;

    Ok(())
}

#[post("/finish_session/<session_internal_id>/<memorable>")]
async fn finish_session(session_internal_id: u32, memorable: bool) -> Result<(), EErrorResponse> {
    let mut db = load_db().map_err(|_| EErrorResponse::DBError)?;

    let mut found_session = false;
    for s in &mut db.sessions {
        if s.internal_id == session_internal_id {
            s.finish(memorable);
            found_session = true;
            break;
        }
    }

    if !found_session {
        return Err(EErrorResponse::BadRequest(String::from("Could not find session with matching internal_id to finish.")));
    }

    save_db(db).map_err(|_| EErrorResponse::DBError)?;

    Ok(())
}

#[post("/get_sessions", data = "<filter>")]
async fn get_sessions(filter: RocketJson<core::SSessionFilter>) -> Result<RocketJson<Vec<core::SSessionAndGameInfo>>, EErrorResponse> {
    let db = load_db().map_err(|_| EErrorResponse::DBError)?;

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

            result.push(core::SSessionAndGameInfo{
                session: session.clone(),
                game_info: game.game_info,
            });
        }
    }

    Ok(RocketJson(result))
}

#[post("/get_randomizer_games", data = "<filter>")]
async fn get_randomizer_games(filter: RocketJson<core::SRandomizerFilter>) -> Result<RocketJson<core::SRandomizerList>, EErrorResponse> {
    let filter_inner = filter.into_inner();

    let db = load_db().map_err(|_| EErrorResponse::DBError)?;

    let mut active_session_game_ids = std::collections::HashSet::new();
    for session in &db.sessions {
        if let core::ESessionState::Ongoing = session.state {
            active_session_game_ids.insert(session.game_internal_id);
        }
    }

    let mut result = Vec::with_capacity(db.games.len());

    for game in &db.games {
        if !active_session_game_ids.contains(&game.internal_id) && filter_inner.game_passes(&game) {
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
async fn update_choose_state(games: RocketJson<Vec<core::SCollectionGame>>) -> Result<(), EErrorResponse> {
    let games_inner = games.into_inner();
    let mut db = load_db().map_err(|_| EErrorResponse::DBError)?;

    let mut input_idx = 0;
    let mut output_idx = 0;

    while input_idx < games_inner.len() && output_idx < db.games.len() {
        if games_inner[input_idx].internal_id == db.games[output_idx].internal_id {
            db.games[output_idx].choose_state = games_inner[input_idx].choose_state;
            input_idx = input_idx + 1;
            output_idx = output_idx + 1;
        }
        else {
            output_idx = output_idx + 1;

            // -- EVERY game in games should be present in collection_games, and both vecs
            // -- should be strictly in order.
            if db.games[output_idx].internal_id > games_inner[input_idx].internal_id {
                return Err(EErrorResponse::BadRequest(String::from("During update_choose_state, either games or collection_games as out of order!")));
            }
        }
    }

    save_db(db).map_err(|_| EErrorResponse::DBError)?;

    Ok(())
}

#[post("/reset_choose_state/<game_internal_id>")]
async fn reset_choose_state(game_internal_id: u32) -> Result<(), EErrorResponse> {
    let mut db = load_db().map_err(|_| EErrorResponse::DBError)?;

    for game in &mut db.games {
        if game.internal_id == game_internal_id {
            game.choose_state.reset();
            save_db(db).map_err(|_| EErrorResponse::DBError)?;
            return Ok(());
        }
    }

    Err(EErrorResponse::BadRequest(format!("Did not find game with internal_id {} to reset choose_state on", game_internal_id)))
}

#[launch]
fn rocket() -> _ {
    // -- $$$FRK(TODO): verify we have valid config file, all values present

    rocket::build()
        .mount("/static", rocket::fs::FileServer::from("../client/served_files"))
        .mount("/", routes![
            search_igdb,
            add_game,
            edit_game,
            get_recent_collection_games,
            search_collection,
            start_session,
            finish_session,
            get_sessions,
            get_randomizer_games,
            update_choose_state,
            reset_choose_state,
        ])
}