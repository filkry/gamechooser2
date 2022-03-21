//use reqwest;
use async_trait::async_trait;
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

#[async_trait(?Send)]
pub trait TTwitchAPIClient {
    type Session : Clone + Send;

    async fn init(params: STwitchOauthTokenRequest) -> Result<Self::Session, String>;
    async fn post_interp_json<T: DeserializeOwned>(session: Self::Session, request: STwitchAPIRequestBuilder) -> Result<T, String>;
    async fn post_text(session: Self::Session, request: STwitchAPIRequestBuilder) -> Result<String, String>;
    fn access_token(session: &Self::Session) -> &str;
}

/*
#[async_trait]
pub trait TTwitchAPIPostResponse {
    async fn json<T: DeserializeOwned>(self) -> Result<T, String>;
    async fn text(self) -> Result<String, String>;
}

#[async_trait]
pub trait TTwitchAPIPost {
    type Response : TTwitchAPIPostResponse;

    fn header_str(self, field_name: &str, value: &str) -> Self;
    fn header_string(self, field_name: &str, value: String) -> Self;
    fn body(self, value: &'static str) -> Self;
    //fn form<T: Serialize>(self, params: &T) -> Self;
    async fn send(self) -> Result<Self::Response, String>;
}

#[async_trait]
pub trait TTwitchAPIClient {
    type Post : TTwitchAPIPost;

    async fn init_access_token(&mut self, params: &STwitchOauthTokenRequest) -> Result<(), String>;
    fn post(&self, url: &str) -> Self::Post;
    fn access_token(&self) -> String;
}
*/

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

pub async fn test_any_client<T: TTwitchAPIClient, C: TConfigStore>(config_store: &C) -> Result<String, String> {
    let params = STwitchOauthTokenRequest{
        client_id: config_store.get_twitch_client_id().unwrap(),
        client_secret: config_store.get_twitch_client_secret().unwrap(),
        grant_type: "client_credentials",
    };
    let session = T::init(params.clone()).await?;

    let request = STwitchAPIRequestBuilder::new()
        .url("https://api.igdb.com/v4/search/")
        .header("Client-ID", params.client_id.as_str())
        .header("Authorization", format!("Bearer {}", T::access_token(&session)).as_str())
        .body("search \"Halo\"; fields game,name;");

    let searchres = T::post_text(session.clone(), request).await;

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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
