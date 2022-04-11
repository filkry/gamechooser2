use argh::FromArgs;
use confy;
//use gamechooser_core::*;

use igdb_api_client::SConfigFile;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "test")]
#[argh(description = "Run whatever test code is currently in test()")]
struct SArghsTest {

}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "set_twitch_client")]
#[argh(description = "Set twitch client ID/Secret for using IGDB")]
struct SArghsSetTwitchClient {
    #[argh(positional)]
    client_id: String,

    #[argh(positional)]
    client_secret: String,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum EArghsSubcommands {
    Test(SArghsTest),
    SetTwitchClient(SArghsSetTwitchClient),
}

#[derive(FromArgs)]
#[argh(description = "Args")]
struct SArghs {
    #[argh(subcommand)]
    subcommand: EArghsSubcommands,
}

fn test() {
}

fn main() {
    let arghs: SArghs = argh::from_env();
    match arghs.subcommand {
        EArghsSubcommands::Test(_) => {
            test();
        }
        EArghsSubcommands::SetTwitchClient(stc) => {
            let mut cfg : SConfigFile = confy::load("gamechooser2_igdb_api_client").unwrap();
            cfg.set_twitch_client(stc.client_id.as_str(), stc.client_secret.as_str());
            confy::store("gamechooser2_igdb_api_client", cfg).unwrap()
        }
    }
}
