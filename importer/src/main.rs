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
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
struct SGC1GameToGameInfoMap {
    gc1_id: u32,
    game_info: core::SGameInfo,
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
#[argh(subcommand)]
enum EArghsSubcommands {
    SetGC1Dir(SArghsSetGC1Dir),
    GC1DataPopulateIGDBInfo(SArghsGC1DataPopulateIGDBInfo),
}

#[derive(FromArgs)]
#[argh(description = "Args")]
struct SArghs {
    #[argh(subcommand)]
    subcommand: EArghsSubcommands,
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

            let game_infos = SReqwestTwitchAPIClient::search(&session, games[cur_idx].title.as_str()).await?;

            let mut chosen_game_info : Option<core::SGameInfo> = None;

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
                        // -- go to custorelease_date = Some(chrono::naive::NaiveDate::from_ymd(year as i32, 1, 1));
                }

                chosen_game_info = Some(core::SGameInfo::new_custom(title, release_date));
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
    }
}
