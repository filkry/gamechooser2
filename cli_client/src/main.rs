use argh::FromArgs;
use async_trait::async_trait;
use confy;
use futures::executor::block_on;
use gamechooser_core::*;
use reqwest;
use serde::{Serialize, Deserialize};
use serde::de::{DeserializeOwned};

#[derive(Clone)]
struct SReqestTwitchAPISession {
    client: reqwest::blocking::Client,
    token_info: Option<gamechooser_core::STwitchOauthTokenResponse>,
}

struct SReqwestTwitchAPIClient {
}

#[derive(Default, Serialize, Deserialize)]
struct SConfigFile {
    twitch_client_id: Option<String>,
    twitch_client_secret: Option<String>,
}

struct SConfigStore {
}

impl SReqwestTwitchAPIClient {
    fn prepare_request(session: &SReqestTwitchAPISession, rb: gamechooser_core::STwitchAPIRequestBuilder) -> reqwest::blocking::RequestBuilder {
        let mut request = session.client.post(rb.url);

        if let Some(b) = rb.body {
            request = request.body(b);
        }

        for (hname, hvalue) in rb.headers {
            request = request.header(hname.as_str(), hvalue.as_str());
        }

        request
    }
}

#[async_trait(?Send)]
impl gamechooser_core::TTwitchAPIClient for SReqwestTwitchAPIClient {
    type Session = SReqestTwitchAPISession;

    async fn init(params: gamechooser_core::STwitchOauthTokenRequest) -> Result<Self::Session, String> {
        let client = reqwest::blocking::Client::new();

        let res = client.post("https://id.twitch.tv/oauth2/token")
            .form(&params)
            .send();

        match res {
            Ok(res_) => {
                let resp : STwitchOauthTokenResponse = res_.json().unwrap();
                println!("{:?}", resp);
                Ok(Self::Session{
                    client,
                    token_info: Some(resp),
                })
            },
            Err(e_) => Err(e_.to_string()),
        }

    }

    async fn post_interp_json<T: DeserializeOwned>(session: Self::Session, rb: gamechooser_core::STwitchAPIRequestBuilder) -> Result<T, String> {
        let req = Self::prepare_request(&session, rb);
        let resp = req.send().unwrap();
        Ok(resp.json().unwrap())
    }

    async fn post_text(session: Self::Session, rb: gamechooser_core::STwitchAPIRequestBuilder) -> Result<String, String> {
        let req = Self::prepare_request(&session, rb);
        let resp = req.send().unwrap();
        Ok(resp.text().unwrap())
    }

    fn access_token(session: &Self::Session) -> &str {
        session.token_info.as_ref().unwrap().access_token.as_str()
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
