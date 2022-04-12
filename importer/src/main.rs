use argh::FromArgs;
use csv;
use confy;
use tokio;
use serde::{Serialize, Deserialize};

use igdb_api_client::SReqwestTwitchAPIClient;

const CONFIG_NAME : &str = "gamechooser2_importer";

#[derive(Serialize, Deserialize)]
enum EGC1PopulateIGDBInfoState {
    NotStarted,
    InProgress(u32),
    Finished,
}

#[derive(Default, Serialize, Deserialize)]
struct SConfigFile {
    gc1_dir: String,
    gc1_data_populate_igdb_info_state: EGC1PopulateIGDBInfoState,
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

    if state.restart || matches!(cfg.gc1_data_populate_igdb_info_state, EGC1PopulateIGDBInfoState::NotStarted) {
        cfg.gc1_data_populate_igdb_info_state = EGC1PopulateIGDBInfoState::InProgress(0);
    }

    let mut gc1_games_path = std::path::PathBuf::new();
    gc1_games_path.push(cfg.gc1_dir);
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

    let mut names : Vec<&str> = Vec::new();
    for i in 0..10 {
        names.push(games[i].title.as_str());
    }

    let session = SReqwestTwitchAPIClient::new_session().await?;
    let results = SReqwestTwitchAPIClient::multi_search(&session, &names).await?;

    println!("{:?}", results);

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
