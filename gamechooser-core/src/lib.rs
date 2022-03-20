//use reqwest;
use serde::{Serialize, Deserialize};
use serde::de::{DeserializeOwned};
use std::result::{Result};

#[derive(Debug, Serialize)]
pub struct STwitchOauthTokenRequest {
    client_id: String,
    client_secret: String,
    grant_type: &'static str,
}

#[derive(Debug, Deserialize)]
pub struct STwitchOauthTokenResponse {
    pub access_token: String,
    pub expires_in: u32,
    pub token_type: String,
}

pub trait TTwitchAPIPostResponse {
    fn json<T: DeserializeOwned>(self) -> Result<T, String>;
    fn text(self) -> Result<String, String>;
}

pub trait TTwitchAPIPost {
    type Response : TTwitchAPIPostResponse;

    fn header_str(self, field_name: &str, value: &str) -> Self;
    fn header_string(self, field_name: &str, value: String) -> Self;
    fn body(self, value: &'static str) -> Self;
    //fn form<T: Serialize>(self, params: &T) -> Self;
    fn send(self) -> Result<Self::Response, String>;
}

pub trait TTwitchAPIClient {
    type Post : TTwitchAPIPost;

    fn init_access_token(&mut self, params: &STwitchOauthTokenRequest) -> Result<(), String>;
    fn post(&self, url: &str) -> Self::Post;
    fn access_token(&self) -> String;
}

pub trait TConfigStore {
    fn get_twitch_client_id(&self) -> Option<String>;
    fn get_twitch_client_secret(&self) -> Option<String>;
    fn save_twitch_client(&self, client_id: &str, client_secret: &str);
}

pub fn test_any_client<T: TTwitchAPIClient, C: TConfigStore>(client: &mut T, config_store: &C) -> Result<String, &'static str> {
    let params = STwitchOauthTokenRequest{
        client_id: config_store.get_twitch_client_id().unwrap(),
        client_secret: config_store.get_twitch_client_secret().unwrap(),
        grant_type: "client_credentials",
    };
    client.init_access_token(&params).unwrap();

    let searchres = client.post("https://api.igdb.com/v4/search/")
        .header_str("Client-ID", params.client_id.as_str())
        .header_string("Authorization", format!("Bearer {}", client.access_token()))
        .body("search \"Halo\"; fields game,name;")
        .send();

    //println!("{:?}", searchres);
    let searchresp = match searchres {
        Ok(searchres_) => {
            let searchresp : String = searchres_.text().unwrap();
            println!("{:?}", searchresp);
            searchresp
        },
        Err(e_) => {
            println!("Err status: {:?}", e_);
            return Err("FAIL");
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
