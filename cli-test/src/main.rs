use gamechooser_core::*;
use reqwest;
//use serde::{Serialize, Deserialize};
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

fn main() {

    let mut client = SReqwestTwitchAPIClient {
        client: reqwest::blocking::Client::new(),
        token_info: None,
    };

    gamechooser_core::test_any_client(&mut client).unwrap();
}
