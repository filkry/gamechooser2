use argh::FromArgs;
use csv;
use confy;
use tokio;
use serde::{Serialize, Deserialize};

use gamechooser_core as core;

use igdb_api_client::SReqwestTwitchAPIClient;

const CONFIG_NAME : &str = "gamechooser2_importer";

#[derive(Serialize, Deserialize)]
enum EGC1PopulateIGDBInfoState {
    NotStarted,
    InProgress,
    Finished,
}

#[derive(Default, Serialize, Deserialize)]
struct SConfigFile {
    gc1_dir: String,
    gc1_data_populate_igdb_info_state: EGC1PopulateIGDBInfoState,
    gc1_data_populate_igdb_next_idx: u32,
    gc2_dir: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SGC1Game {
    id: u32,
    title: String,
    release_year: Option<u16>,

    linux: Option<u8>,
    play_more: Option<u8>,
    couch: Option<u8>,
    portable: Option<u8>,

    passes: u16,

    via: String,

    eternal: Option<u8>,
    next_valid_date: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SGC1Own {
    game_id: u32,
    storefront: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SGC1Session {
    game_id: u32,
    started: Option<String>,
    outcome: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SGC1GameToGameInfoMap {
    gc1_id: u32,
    game_info: core::EGameInfo,
}

impl Default for EGC1PopulateIGDBInfoState {
    fn default() -> Self {
        Self::NotStarted
    }
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "gc1_data_populate_igdb_info")]
#[argh(description = "Starting with gamechooser1 data, create a file which associates gc1 games with IGDB entries. Will only do 10 entries per invocation.")]
struct SArghsGC1DataPopulateIGDBInfo {
    #[argh(switch)]
    #[argh(description = "restart the population process, discarding all results so far.")]
    restart: bool,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "set_gc1_dir")]
#[argh(description = "Set the directory with gamechooser1 data.")]
struct SArghsSetGC1Dir {
    #[argh(positional)]
    dir: String,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "gc1_to_gc2")]
#[argh(description = "Create a GC2 database from GC1 data + igdb info map")]
struct SArghsCreateGC2DBFromGC1Data {
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "db0_to_db1")]
#[argh(description = "Convert GC2 DB V0 to V1")]
struct SArghsConvertV0DBToV1 {
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum EArghsSubcommands {
    SetGC1Dir(SArghsSetGC1Dir),
    GC1DataPopulateIGDBInfo(SArghsGC1DataPopulateIGDBInfo),
    CreateGC2DBFromGC1Data(SArghsCreateGC2DBFromGC1Data),
    ConvertV0DBToV1(SArghsConvertV0DBToV1),
}

#[derive(FromArgs)]
#[argh(description = "Args")]
struct SArghs {
    #[argh(subcommand)]
    subcommand: EArghsSubcommands,
}

fn read_gc1_csv<D: serde::de::DeserializeOwned>(name: &str) -> Result<Vec<D>, String> {
    let cfg : SConfigFile = confy::load(CONFIG_NAME).unwrap();
    let mut path = std::path::PathBuf::new();
    path.push(cfg.gc1_dir.clone());
    path.push(name);

    let mut output = Vec::new();

    let rdr_opt = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(path);
    if let Err(e) = rdr_opt {
        return Err(format!("Failed to load gc1 {} with error: {:?}", name, e));
    }
    let mut rdr = rdr_opt.expect("checked above");

    for result in rdr.deserialize() {
        if let Err(e) = &result {
            return Err(format!("Failed to deserialize gc1 {} with error: {:?}", name, e));
        }
        let data: D = result.expect("checked above");
        output.push(data);
    }

    Ok(output)
}

fn create_gc2_db_from_gc1_data() {
    let cfg : SConfigFile = confy::load(CONFIG_NAME).unwrap();
    let mut path = std::path::PathBuf::new();
    path.push(cfg.gc1_dir.clone());

    let mut map_path = path.clone();
    map_path.push("gc1_to_gc2_game_info_map.json");

    // -- load the existing map from disc
    let map : std::collections::HashMap<u32, SGC1GameToGameInfoMap> = {
        if map_path.exists() {
            let file = match std::fs::File::open(map_path.clone()) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Failed to open gc1_to_gc2_game_info_map.json file with error: {}", e);
                    return;
                }
            };
            let reader = std::io::BufReader::new(file);

            match serde_json::from_reader(reader) {
                Ok(g) => g,
                Err(e) => {
                    eprintln!("Failed to deserialize gc1_to_gc2_game_info_map.json file with error: {}", e);
                    return;
                }
            }
        }
        else {
            eprintln!("No gc1_to_gc2_game_info_map.json file in gc1 dir.");
            return;
        }
    };

    let gc1_games : Vec<SGC1Game> = match read_gc1_csv("_game.csv") {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    let gc1_own : Vec<SGC1Own> = match read_gc1_csv("_own.csv") {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    let gc1_sessions : Vec<SGC1Session> = match read_gc1_csv("_session.csv") {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    let mut db_inner = core::SDatabase::new();

    for game in gc1_games {
        let id = game.id;
        let game_info = match map.get(&id) {
            Some(gi) => {
                assert!(gi.gc1_id == id);
                gi.game_info.clone()
            },
            None => {
                eprintln!("gc1 to gc2 map is missing entry for game with ID {}", id);
                return;
            }
        };

        let custom_info = core::SGameCustomInfo {
            via: game.via,
            tags: core::SGameTags{
                couch_playable: match game.couch {
                    Some(c) => if c > 0 { true } else { false },
                    None => false,
                },
                portable_playable: match game.portable {
                    Some(p) => if p > 0 { true } else { false },
                    None => false,
                },
            },
            own: Default::default(),
        };

        let date_result = chrono::NaiveDate::parse_from_str(game.next_valid_date.as_str(), "%Y-%m-%d");
        let date = match date_result {
            Ok(d) => d,
            Err(_) => {
                eprintln!("Could not parse date \"{}\" for game with ID {}, using today as next valid date", game.next_valid_date, game.id);
                chrono::offset::Local::now().naive_local().date()
            },
        };

        let choose_state = core::SGameChooseState {
            next_valid_proposal_date: date,
            retired: match game.play_more {
                Some(p) => if p > 0 { false } else { true },
                None => false,
            },
            passes: game.passes,
            ignore_passes: match game.eternal {
                Some(e) => if e > 0 { true } else { false },
                None => false,
            },
        };

        db_inner.games.push(core::SCollectionGame {
            internal_id: game.id,
            game_info,
            custom_info,
            choose_state,
        });
    }

    fn game_mut(db: &mut core::SDatabase, id: u32) -> Option<&mut core::SCollectionGame> {
        for cand_game in &mut db.games {
            if cand_game.internal_id == id {
                return Some(cand_game);
            }
        }
        None
    }

    for own in gc1_own {
        let mut game = match game_mut(&mut db_inner, own.game_id) {
            Some(g) => g,
            None => {
                continue;
            }
        };

        match own.storefront.as_str() {
            "free" => game.custom_info.own.free = true,
            "f2p" => game.custom_info.own.free = true,
            "steam" => game.custom_info.own.steam = true,
            " steam" => game.custom_info.own.steam = true,
            "stea," => game.custom_info.own.steam = true,
            "steam,humble" => game.custom_info.own.steam = true,
            "steam, rom" => {
                game.custom_info.own.steam = true;
                game.custom_info.own.emulator = true;
            },
            "bansteam" => {
                game.custom_info.own.steam = true;
                game.custom_info.own.ban_owned = true;
            },
            "gmg" => game.custom_info.own.gmg = true,
            "gog" => game.custom_info.own.gog = true,
            "humble" => game.custom_info.own.humble = true,
            "origin" => game.custom_info.own.origin = true,
            "banorigin" => {
                game.custom_info.own.origin = true;
                game.custom_info.own.ban_owned = true;
            },
            "egs" => game.custom_info.own.egs = true,
            "battlenet" => game.custom_info.own.battlenet = true,
            "itchio" => game.custom_info.own.itch = true,
            "square" => game.custom_info.own.standalone_launcher = true,
            "gc" => game.custom_info.own.gamecube = true,
            "emu" => game.custom_info.own.emulator = true,
            "rom" => game.custom_info.own.emulator = true,
            "mister" => game.custom_info.own.emulator = true,
            "gba" => game.custom_info.own.gba = true,
            "ds" => game.custom_info.own.ds = true,
            "3ds" => game.custom_info.own.n3ds = true,
            "ban3ds" => {
                game.custom_info.own.n3ds = true;
                game.custom_info.own.ban_owned = true;
            },
            "wii" => game.custom_info.own.wii = true,
            "wiiu" => game.custom_info.own.wiiu = true,
            "switch" => game.custom_info.own.switch = true,
            "switvh" => game.custom_info.own.switch = true,
            "eshop" => game.custom_info.own.switch = true,
            "switcheshop" => game.custom_info.own.switch = true,
            "ps1" => game.custom_info.own.ps1 = true,
            "ps2" => game.custom_info.own.ps2 = true,
            "ps3" => game.custom_info.own.ps3 = true,
            "ps4" => game.custom_info.own.ps4 = true,
            "ps5" => game.custom_info.own.ps5 = true,
            "psp" => game.custom_info.own.psp = true,
            "vita" => game.custom_info.own.vita = true,
            "psn" => game.custom_info.own.ps4 = true,
            "xbox" => game.custom_info.own.xbox = true,
            "ios" => game.custom_info.own.ios = true,
            "quest" => game.custom_info.own.oculus_quest = true,
            "indiegamestand->steam" => (),
            "desura" => (),
            "indieroyale" => (),
            "devpage" => (),
            "pc" => (),
            "beta" => (),
            "eso" => (),
            "uplay" => (),
            "gamepass" => (),
            "applearcade" => (),
            "freetrial" => (),
            "n" => (),
            "jeremylent" => (),
            _ => {
                eprintln!("Encountered unknown own platform \"{}\"", own.storefront.as_str());
                return;
            }
        }

        /*
        game_id: u32,
        storefront: String,
        */
    }

    let mut next_session_id = 0;
    for session in gc1_sessions {

        if session.started.is_none() {
            continue; // no way to provide reasonable data for these entries, drop them
        }

        let mut found_game = false;
        for game in &db_inner.games {
            if game.internal_id == session.game_id {
                found_game = true;
                break;
            }
        }

        if !found_game {
            continue; // some games were deleted from the DB but sessions remain, drop these sessions
        }

        let start_date_str = session.started.as_ref().expect("checked above").as_str();
        let date_result = chrono::NaiveDate::parse_from_str(start_date_str, "%Y-%m-%d");
        let date = match date_result {
            Ok(d) => d,
            Err(_) => {
                let datetime_result = chrono::NaiveDateTime::parse_from_str(start_date_str, "%Y-%m-%d %H:%M:%S");
                match datetime_result {
                    Ok(dt) => dt.date(),
                    Err(_) => {
                        eprintln!("Could not parse date \"{}\" for session of game with ID {}", start_date_str, session.game_id);
                        return;
                    }
                }
            },
        };

        let session_state = match session.outcome.as_str() {
            "stuck" => core::ESessionState::Finished {
                end_date: date.succ(), // we don't have this info, so it's day + 1 for now
                memorable: true,
            },
            "tansient" => core::ESessionState::Finished {
                end_date: date.succ(), // we don't have this info, so it's day + 1 for now
                memorable: false,
            },
            "transient" => core::ESessionState::Finished {
                end_date: date.succ(), // we don't have this info, so it's day + 1 for now
                memorable: false,
            },
            "" => core::ESessionState::Ongoing,
            _ => {
                eprintln!("Unknown session outcome \"{}\" encountered", session.outcome);
                return;
            }
        };

        let gc2_session = core::SSession {
            internal_id: next_session_id,
            game_internal_id: session.game_id,
            start_date: date,
            state: session_state,
        };

        db_inner.sessions.push(gc2_session);
        next_session_id = next_session_id + 1;
    }

    let db = core::EDatabase::V1(db_inner);

    let mut out_path = path.clone();
    out_path.push("database.json");

    if out_path.exists() {
        eprintln!("database.json already exists at {:?}, not exporting.", out_path);
        return;
    }

    let open_options = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .append(true)
        .open(out_path);

    let file = match open_options {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open database.json with: {:?}", e);
            return;
        }
    };
    let writer = std::io::BufWriter::new(file);

    match serde_json::to_writer_pretty(writer, &db) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Failed to serialize database.json with: {:?}", e);
            return;
        }
    };
}

async fn gc1_data_populate_igdb_info(state: SArghsGC1DataPopulateIGDBInfo) -> Result<(), String> {
    let mut cfg : SConfigFile = confy::load(CONFIG_NAME).unwrap();

    let mut map_path = std::path::PathBuf::new();
    map_path.push(cfg.gc1_dir.clone());
    map_path.push("gc1_to_gc2_game_info_map.json");

    let mut map_bak_dir = std::path::PathBuf::new();
    map_bak_dir.push(cfg.gc1_dir.clone());
    map_bak_dir.push("gc1_to_gc2_game_info_bak/");

    if state.restart || matches!(cfg.gc1_data_populate_igdb_info_state, EGC1PopulateIGDBInfoState::NotStarted) {
        cfg.gc1_data_populate_igdb_info_state = EGC1PopulateIGDBInfoState::InProgress;
        cfg.gc1_data_populate_igdb_next_idx = 0;
        if map_path.exists() {

            let bak_file_name = format!("gc1_to_gc2_game_info_map_{}.json", chrono::offset::Utc::now().timestamp());
            let mut map_bak_file = map_bak_dir.clone();
            map_bak_file.push(bak_file_name);
            if let Err(_) = std::fs::copy(map_path.clone(), map_bak_file.clone()) {
                return Err(String::from("Failed to backup map json before deletion."));
            }

            if let Err(_) = std::fs::remove_file(map_path.clone()) {
                return Err(String::from("Failed to delete map json."));
            }
        }
    }
    else if matches!(cfg.gc1_data_populate_igdb_info_state, EGC1PopulateIGDBInfoState::Finished) {
        println!("All games have been process with IGDB info, goodbye!");
    }

    let mut gc1_games_path = std::path::PathBuf::new();
    gc1_games_path.push(cfg.gc1_dir.clone());
    gc1_games_path.push("_game.csv");

    let mut games = Vec::new();

    let rdr_opt = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(gc1_games_path);
    if let Err(e) = rdr_opt {
        println!("Failed to load gc1 games CSV with error: {:?}", e);
        return Ok(());
    }
    let mut rdr = rdr_opt.expect("checked above");

    for result in rdr.deserialize() {
        if let Err(e) = &result {
            println!("Failed to deserialize gc1 game with error: {}", e);
            return Ok(());
        }
        let game: SGC1Game = result.expect("checked above");
        games.push(game);
    }

    if let EGC1PopulateIGDBInfoState::InProgress = cfg.gc1_data_populate_igdb_info_state {

        let start_idx = cfg.gc1_data_populate_igdb_next_idx;

        // -- load the existing map from disc
        let mut map : std::collections::HashMap<u32, SGC1GameToGameInfoMap> = {
            if map_path.exists() {
                let file = match std::fs::File::open(map_path.clone()) {
                    Ok(f) => f,
                    Err(_) => {
                        return Err(String::from("Failed to open map json."));
                    }
                };
                let reader = std::io::BufReader::new(file);

                match serde_json::from_reader(reader) {
                    Ok(g) => g,
                    Err(_) => {
                        return Err(String::from("Failed to deserialize map json."));
                    }
                }
            }
            else {
                std::collections::HashMap::new()
            }
        };

        // -- start querying IGDB
        let mut cur_idx : usize = start_idx as usize;
        let mut count = 0;

        let session = SReqwestTwitchAPIClient::new_session().await?;

        let stdin = std::io::stdin();
        let mut buffer = String::new();

        while cur_idx < games.len() && count < 15 {
            println!("\nCreating map for GC1 game \"{}\"", games[cur_idx].title);

            let game_infos = SReqwestTwitchAPIClient::search(&session, games[cur_idx].title.as_str(), false).await?;

            let mut chosen_game_info : Option<core::EGameInfo> = None;

            if game_infos.len() > 0 {
                println!("Found games on IGDB:");
                for (idx, gi) in game_infos.iter().enumerate() {
                    match gi.release_date() {
                        Some(date) => {
                            println!("{}: {} ({})", idx, gi.title(), date);
                        },
                        None => {
                            println!("{}: {}", idx, gi.title());
                        },
                    }
                }

                loop {
                    println!("Please enter {}-{} to select an entry, or \"keep\" to keep GC1 data.", 0, game_infos.len() - 1);
                    buffer.clear();
                    stdin.read_line(&mut buffer).unwrap();

                    if buffer.trim().eq("keep") {
                        // -- go to custom handler
                        println!("Keeping data from GC1.");
                        break;
                    }

                    if let Ok(idx) = buffer.trim().parse::<usize>() {
                        if idx < game_infos.len() {
                            println!("Copying IGDB data from \"{}\".", game_infos[idx].title());
                            chosen_game_info = Some(game_infos[idx].clone());
                            break;
                        }
                    }

                    println!("Could not understand input, try again.");
                }
            }
            else {
                println!("Found no matching games on IGDB. Keeping gc1 data.");
            }

            if chosen_game_info.is_none() {
                let title = games[cur_idx].title.clone();

                let mut release_date : Option<chrono::naive::NaiveDate> = None;
                if let Some(year) = games[cur_idx].release_year {
                    release_date = Some(chrono::naive::NaiveDate::from_ymd(year as i32, 1, 1));
                }

                chosen_game_info = Some(core::EGameInfo::new_custom(title, release_date));
            }

            assert!(chosen_game_info.is_some());

            let game_map = SGC1GameToGameInfoMap {
                gc1_id: games[cur_idx].id,
                game_info: chosen_game_info.expect("checked above"),
            };

            map.insert(game_map.gc1_id, game_map);

            cur_idx += 1;
            count += 1;

            // -- just hard sleep here to avoid using up our API request budget
            tokio::time::sleep(std::time::Duration::from_secs_f32(0.5)).await;
        }

        // -- save out the modified map
        {
            if map_path.exists() {
                let bak_file_name = format!("gc1_to_gc2_game_info_map_{}.json", chrono::offset::Utc::now().timestamp());
                let mut map_bak_file = map_bak_dir.clone();
                map_bak_file.push(bak_file_name);
                if let Err(e) = std::fs::copy(map_path.clone(), map_bak_file.clone()) {
                    println!("{:?}", map_bak_file);
                    println!("Internal error: {}", e);
                    return Err(String::from("Failed to backup map json before deletion."));
                }

                if let Err(_) = std::fs::remove_file(map_path.clone()) {
                    return Err(String::from("Failed to delete map file before saving."));
                }
            }

            let open_options = std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .append(true)
                .open(map_path);

            let file = match open_options {
                Ok(f) => f,
                Err(_) => {
                    return Err(String::from("Failed to open map file for write."));
                }
            };
            let writer = std::io::BufWriter::new(file);

            match serde_json::to_writer_pretty(writer, &map) {
                Ok(_) => {},
                Err(_) => {
                    return Err(String::from("Failed to serialize map file."));
                }
            };
        }

        if cur_idx == games.len() {
            cfg.gc1_data_populate_igdb_info_state = EGC1PopulateIGDBInfoState::Finished;
        }
        else {
            cfg.gc1_data_populate_igdb_info_state = EGC1PopulateIGDBInfoState::InProgress;
            cfg.gc1_data_populate_igdb_next_idx = cur_idx as u32;
        }

        if let Err(e) = confy::store(CONFIG_NAME, cfg) {
            println!("Failed to write config file with error: {}", e);
        }

        println!("Progress: {}/{}", cur_idx, games.len());
    }

    Ok(())
}

async fn convert_v0_db_to_v1() -> Result<(), String> {
    let cfg : SConfigFile = confy::load(CONFIG_NAME).unwrap();

    let mut db_path = std::path::PathBuf::new();
    db_path.push(cfg.gc2_dir.clone());
    db_path.push("database.json");

    let db : core::EDatabase = {
        if db_path.exists() {
            let file = match std::fs::File::open(db_path.clone()) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Failed to open {:?} with: {:?}", db_path, e);
                    return Ok(());
                }
            };
            let reader = std::io::BufReader::new(file);

            // Read the JSON contents of the file as an instance of `User`.
            match serde_json::from_reader(reader) {
                Ok(g) => g,
                Err(e) => {
                    eprintln!("Failed to deserialize {:?} with: {:?}", db_path, e);
                    return Ok(());
                }
            }
        }
        else {
            core::EDatabase::new()
        }
    };

    let mut v1db_inner = core::SDatabase::new();

    let session = SReqwestTwitchAPIClient::new_session().await?;

    if let core::EDatabase::V0(v0db) = db {
        for game in v0db.games {
            println!("Updating: {}", game.game_info.title);

            let new_game_info = {
                match game.game_info.igdb_id {
                    Some(igdb_id) => {
                        let igdb_game_info = SReqwestTwitchAPIClient::get_game_info(&session, igdb_id).await?;
                        // -- just hard sleep here to avoid using up our API request budget
                        tokio::time::sleep(std::time::Duration::from_secs_f32(0.5)).await;
                        igdb_game_info
                    },
                    None => {
                        core::EGameInfo::new_custom(game.game_info.title, game.game_info.release_date)
                    }
                }
            };

            let new_game = core::SCollectionGame {
                internal_id: game.internal_id,
                game_info: new_game_info,
                custom_info: game.custom_info,
                choose_state: game.choose_state,
            };

            v1db_inner.games.push(new_game);
        }

        v1db_inner.sessions = v0db.sessions;

        let mut new_db_path = std::path::PathBuf::new();
        new_db_path.push(cfg.gc2_dir.clone());
        new_db_path.push("database_v1.json");

        let open_options = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .append(true)
            .open(new_db_path);

        let file = match open_options {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to open database_v1.json with: {:?}", e);
                return Ok(());
            }
        };
        let writer = std::io::BufWriter::new(file);

        let new_db = core::EDatabase::V1(v1db_inner);

        match serde_json::to_writer_pretty(writer, &new_db) {
            Ok(_) => {},
            Err(e) => {
                eprintln!("Failed to serialize database.json with: {:?}", e);
                return Ok(());
            }
        };
    }
    else {
        eprintln!("Not a V0 database");
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let arghs: SArghs = argh::from_env();
    match arghs.subcommand {
        EArghsSubcommands::SetGC1Dir(dir) => {
            let mut cfg : SConfigFile = confy::load(CONFIG_NAME).unwrap();
            cfg.gc1_dir = dir.dir;
            confy::store(CONFIG_NAME, cfg).unwrap()
        }
        EArghsSubcommands::GC1DataPopulateIGDBInfo(state) => {
            gc1_data_populate_igdb_info(state).await.unwrap();
        }
        EArghsSubcommands::CreateGC2DBFromGC1Data(_) => {
            create_gc2_db_from_gc1_data();
        }
        EArghsSubcommands::ConvertV0DBToV1(_) => {
            convert_v0_db_to_v1().await.unwrap();
        }
    }
}
