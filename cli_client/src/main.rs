use argh::FromArgs;
use async_trait::async_trait;
use confy;
use futures::executor::block_on;
use gamechooser_core::*;
use reqwest;
use serde::{Serialize, Deserialize};
use serde::de::{DeserializeOwned};

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
    let config_store = SConfigStore{};

    let future = gamechooser_core::test_any_client::<SReqwestTwitchAPIClient, SConfigStore>(&config_store);
    block_on(future).unwrap();
}

fn main() {
    let arghs: SArghs = argh::from_env();
    match arghs.subcommand {
        EArghsSubcommands::Test(_) => {
            test();
        }
        EArghsSubcommands::SetTwitchClient(stc) => {
            let config_store = SConfigStore{};
            config_store.save_twitch_client(stc.client_id.as_str(), stc.client_secret.as_str());
        }
    }
}
