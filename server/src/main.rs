#[macro_use] extern crate rocket;

//use reqwest;
use serde::{Serialize, Deserialize};
use serde::de::{DeserializeOwned};
use std::result::{Result};

#[derive(Debug, Serialize, Clone)]
pub struct STwitchOauthTokenRequest {
    pub client_id: String,
    pub client_secret: String,
    pub grant_type: &'static str,
}

#[derive(Clone, Debug, Deserialize)]
pub struct STwitchOauthTokenResponse {
    pub access_token: String,
    pub expires_in: u32,
    pub token_type: String,
}

pub struct STwitchAPIRequestBuilder {
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

impl STwitchAPIRequestBuilder {
    pub fn new() -> Self {
        Self {
            url: String::new(),
            headers: Vec::new(),
            body: None,
        }
    }

    pub fn url(mut self, url: &str) -> Self {
        self.url = url.to_string();
        self
    }

    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_string(), value.to_string()));
        self
    }

    pub fn body(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());
        self
    }
}

pub trait TConfigStore {
    fn get_twitch_client_id(&self) -> Option<String>;
    fn get_twitch_client_secret(&self) -> Option<String>;
    fn save_twitch_client(&self, client_id: &str, client_secret: &str);
}

#[derive(Clone)]
struct SReqwestTwitchAPISession {
    client: reqwest::Client,
    token_info: Option<STwitchOauthTokenResponse>,
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
    fn prepare_request(session: &SReqwestTwitchAPISession, rb: STwitchAPIRequestBuilder) -> reqwest::RequestBuilder {
        let mut request = session.client.post(rb.url);

        if let Some(b) = rb.body {
            request = request.body(b);
        }

        for (hname, hvalue) in rb.headers {
            request = request.header(hname.as_str(), hvalue.as_str());
        }

        request
    }

    async fn init(params: STwitchOauthTokenRequest) -> Result<SReqwestTwitchAPISession, String> {
        let client = reqwest::Client::new();

        let res = client.post("https://id.twitch.tv/oauth2/token")
            .form(&params)
            .send()
            .await;

        match res {
            Ok(res_) => {
                let resp : STwitchOauthTokenResponse = res_.json().await.unwrap();
                println!("{:?}", resp);
                Ok(SReqwestTwitchAPISession{
                    client,
                    token_info: Some(resp),
                })
            },
            Err(e_) => Err(e_.to_string()),
        }
    }

    async fn post_interp_json<T: DeserializeOwned>(session: SReqwestTwitchAPISession, rb: STwitchAPIRequestBuilder) -> Result<T, String> {
        let req = Self::prepare_request(&session, rb);
        let resp = req.send().await.unwrap();
        Ok(resp.json().await.unwrap())
    }

    async fn post_text(session: SReqwestTwitchAPISession, rb: STwitchAPIRequestBuilder) -> Result<String, String> {
        let req = Self::prepare_request(&session, rb);
        let resp = req.send().await.unwrap();
        Ok(resp.text().await.unwrap())
    }

    fn access_token(session: &SReqwestTwitchAPISession) -> &str {
        session.token_info.as_ref().unwrap().access_token.as_str()
    }
}

impl SConfigStore {
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

async fn test_client(config_store: &SConfigStore) -> Result<String, String> {
    let params = STwitchOauthTokenRequest{
        client_id: config_store.get_twitch_client_id().unwrap(),
        client_secret: config_store.get_twitch_client_secret().unwrap(),
        grant_type: "client_credentials",
    };

    let session = SReqwestTwitchAPIClient::init(params.clone()).await?;

    let request = STwitchAPIRequestBuilder::new()
        .url("https://api.igdb.com/v4/search/")
        .header("Client-ID", params.client_id.as_str())
        .header("Authorization", format!("Bearer {}", SReqwestTwitchAPIClient::access_token(&session)).as_str())
        .body("search \"Halo\"; fields game,name;");

    let searchres = SReqwestTwitchAPIClient::post_text(session.clone(), request).await;

    //println!("{:?}", searchres);
    let searchresp = match searchres {
        Ok(searchres_) => {
            println!("{:?}", searchres_);
            searchres_
        },
        Err(e_) => {
            println!("Err status: {:?}", e_);
            return Err(String::from("FAIL"));
        }
    };

    Ok(searchresp)
}

#[get("/")]
async fn index() -> Result<String, String> {
    test_client(&SConfigStore{}).await
}

#[post("/test")]
async fn test() -> Result<String, String> {
    test_client(&SConfigStore{}).await
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/static", rocket::fs::FileServer::from("../client/served_files"))
        .mount("/", routes![index, test])
}