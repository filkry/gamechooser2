use argh::FromArgs;
use confy;
use gamechooser_core::*;
use reqwest;
use serde::{Serialize, Deserialize};
use serde::de::{DeserializeOwned};

struct SReqwestTwitchAPIPostResponse {
    inner: reqwest::blocking::Response,
}

struct SReqwestTwitchAPIPost {
    inner: reqwest::blocking::RequestBuilder,
}

struct SReqwestTwitchAPIClient {
    client: reqwest::blocking::Client,
    token_info: Option<gamechooser_core::STwitchOauthTokenResponse>,
}

#[derive(Default, Serialize, Deserialize)]
struct SConfigFile {
    twitch_client_id: Option<String>,
    twitch_client_secret: Option<String>,
}

struct SConfigStore {
}

impl gamechooser_core::TTwitchAPIPostResponse for SReqwestTwitchAPIPostResponse {
    fn json<T: DeserializeOwned>(self) -> Result<T, String> {
        Ok(self.inner.json().unwrap())
    }

    fn text(self) -> Result<String, String> {
        Ok(self.inner.text().unwrap())
    }
}

impl gamechooser_core::TTwitchAPIPost for SReqwestTwitchAPIPost {
    type Response = SReqwestTwitchAPIPostResponse;

    fn header_str(self, field_name: &str, value: &str) -> Self {
        Self {
            inner: self.inner.header(field_name, value),
        }
    }

    fn header_string(self, field_name: &str, value: String) -> Self {
        Self {
            inner: self.inner.header(field_name, value),
        }
    }

    fn body(self, value: &'static str) -> Self {
        Self {
            inner: self.inner.body(value),
        }
    }

    fn send(self) -> Result<Self::Response, String> {
        match self.inner.send() {
            Ok(res) => Ok(SReqwestTwitchAPIPostResponse {
                inner: res,
            }),
            Err(e_) =>  Err(e_.to_string()),
        }
    }
}

impl gamechooser_core::TTwitchAPIClient for SReqwestTwitchAPIClient {
    type Post = SReqwestTwitchAPIPost;

    fn init_access_token(&mut self, params: &STwitchOauthTokenRequest) -> Result<(), String> {
        let res = self.client.post("https://id.twitch.tv/oauth2/token")
            .form(params)
            .send();

        match res {
            Ok(res_) => {
                let resp : STwitchOauthTokenResponse = res_.json().unwrap();
                println!("{:?}", resp);
                self.token_info = Some(resp);
                Ok(())
            },
            Err(e_) => Err(e_.to_string()),
        }
    }

    fn post(&self, url: &str) -> Self::Post {
        SReqwestTwitchAPIPost {
            inner: self.client.post(url),
        }
    }

    fn access_token(&self) -> String {
        self.token_info.as_ref().unwrap().access_token.clone()
    }
}

impl gamechooser_core::TConfigStore for SConfigStore {
    fn get_twitch_client_id(&self) -> Option<String> {
        let cfg : SConfigFile = confy::load("gamechooser2_cli_client").unwrap();
        cfg.twitch_client_id
    }

    fn get_twitch_client_secret(&self) -> Option<String> {
        let cfg : SConfigFile = confy::load("gamechooser2_cli_client").unwrap();
        cfg.twitch_client_secret
    }

    fn save_twitch_client(&self, client_id: &str, client_secret: &str) {
        let mut cfg : SConfigFile = confy::load("gamechooser2_cli_client").unwrap();
        cfg.twitch_client_id = Some(client_id.to_string());
        cfg.twitch_client_secret = Some(client_secret.to_string());
        confy::store("gamechooser2_cli_client", cfg).unwrap();
    }
}

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
    let mut client = SReqwestTwitchAPIClient {
        client: reqwest::blocking::Client::new(),
        token_info: None,
    };

    let config_store = SConfigStore{};

    gamechooser_core::test_any_client(&mut client, &config_store).unwrap();
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
