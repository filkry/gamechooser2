use async_trait::async_trait;
use console_error_panic_hook;
use serde::de::{DeserializeOwned};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

use gamechooser_core;
use gamechooser_core::TConfigStore;

macro_rules! weblog {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

enum ETagQuery {
    TrueOrFalse,
    True,
    False,
}

/*
struct SRandomGameQuery {
    max_passes: u16,
    pick: bool,
    allow_backlog: bool,
    allow_buy: bool,
    couch: ETagQuery,
    portable: ETagQuery,
    short: ETagQuery,
    long: ETagQuery,
}
*/

struct SFetchTwitchAPIPostResponse {
    inner: Response,
}

struct SFetchTwitchAPIPost {
    url: String,
    headers: Vec<(String, String)>,
    inner: RequestInit,
}

struct SFetchTwitchAPIClient {
    token_info: Option<gamechooser_core::STwitchOauthTokenResponse>,
}

impl ETagQuery {
    pub fn new_from_str(strval: &str) -> Self {
        match strval {
            "☐" => Self::TrueOrFalse,
            "☑" => Self::True,
            "☒" => Self::False,
            _ => Self::TrueOrFalse,
        }
    }

    pub fn new_from_element(element_opt: &Option<web_sys::Element>) -> Self {
        if let Some(element) = element_opt {
            if let Ok(span) = element.clone().dyn_into::<web_sys::HtmlSpanElement>() {
                if let Some(tc) = span.text_content() {
                    return ETagQuery::new_from_str(&tc);
                }
            }
        }

        Self::TrueOrFalse
    }
}

#[async_trait]
impl gamechooser_core::TTwitchAPIPostResponse for SFetchTwitchAPIPostResponse {
    async fn json<T: DeserializeOwned>(self) -> Result<T, String> {
        let json = JsFuture::from(self.inner.json().unwrap()).await.unwrap();
        let res : T = json.into_serde().unwrap();
        Ok(res)
    }

    async fn text(self) -> Result<String, String> {
        let text = JsFuture::from(self.inner.text().unwrap()).await.unwrap();
        Ok(text.as_string().unwrap())
    }
}

#[async_trait]
impl gamechooser_core::TTwitchAPIPost for SFetchTwitchAPIPost {
    type Response = SFetchTwitchAPIPostResponse;

    fn header_str(self, field_name: &str, value: &str) -> Self {
        self.headers.push((field_name.to_string(), value.to_string()));
        self
    }

    fn header_string(self, field_name: &str, value: String) -> Self {
        self.headers.push((field_name.to_string(), value));
        self
    }

    fn body(self, value: &'static str) -> Self {
        let value = JsValue::from_str(value);
        self.inner.body(Some(&value));
        self
    }

    async fn send(self) -> Result<Self::Response, String> {
        let request = Request::new_with_str_and_init(&self.url, &self.inner).unwrap();
        for (hname, hvalue) in self.headers {
            request.headers().append(hname.as_str(), hvalue.as_str());
        }

        let window = web_sys::window().unwrap();
        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.unwrap();

        assert!(resp_value.is_instance_of::<Response>());
        let resp: Response = resp_value.dyn_into().unwrap();

        Ok(SFetchTwitchAPIPostResponse{
            inner: resp,
        })
    }
}

#[async_trait]
impl gamechooser_core::TTwitchAPIClient for SFetchTwitchAPIClient {
    type Post = SFetchTwitchAPIPost;

    async fn init_access_token(&mut self, params: &gamechooser_core::STwitchOauthTokenRequest) -> Result<(), String> {
        let mut opts = RequestInit::new();

        opts.method("POST");
        opts.mode(RequestMode::Cors);
        let params_value = JsValue::from_serde(params).unwrap();
        opts.body(Some(&params_value));

        let url = "https://id.twitch.tv/oauth2/token";
        let request = Request::new_with_str_and_init(&url, &opts).unwrap();

        let window = web_sys::window().unwrap();
        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.unwrap();

        assert!(resp_value.is_instance_of::<Response>());
        let resp: Response = resp_value.dyn_into().unwrap();

        let json = JsFuture::from(resp.json().unwrap()).await.unwrap();
        let res : gamechooser_core::STwitchOauthTokenResponse = json.into_serde().unwrap();

        self.token_info = Some(res);

        Ok(())
    }

    fn post(&self, url: &str) -> Self::Post {
        let mut opts = RequestInit::new();
        opts.method("POST");
        opts.mode(RequestMode::Cors);

        SFetchTwitchAPIPost {
            url: url.to_string(),
            headers: Vec::new(),
            inner: opts,
        }
    }

    fn access_token(&self) -> String {
        self.token_info.as_ref().unwrap().access_token.clone()
    }
}

struct SConfigStore {
    local_storage: web_sys::Storage,
}

impl SConfigStore {
    pub fn new() -> Self {
        let window = web_sys::window().expect("no global `window` exists");
        Self{
            local_storage: window.local_storage().unwrap().unwrap(),
        }
    }
}

impl gamechooser_core::TConfigStore for SConfigStore {
    fn get_twitch_client_id(&self) -> Option<String> {
        self.local_storage.get_item("twitch_client_id").unwrap()
    }

    fn get_twitch_client_secret(&self) -> Option<String> {
        self.local_storage.get_item("twitch_client_secret").unwrap()
    }

    fn save_twitch_client(&self, client_id: &str, client_secret: &str) {
        self.local_storage.set_item("twitch_client_id", client_id).unwrap();
        self.local_storage.set_item("twitch_client_secret", client_secret).unwrap();
    }
}

#[wasm_bindgen]
extern {
}

#[wasm_bindgen]
pub fn cycle_tag_tri_box(element: &web_sys::HtmlSpanElement) {
    match element.text_content() {
        Some(tc) => {
            match tc.as_str() {
                "☐" => element.set_text_content(Some("☑")),
                "☑" => element.set_text_content(Some("☒")),
                "☒" => element.set_text_content(Some("☐")),
                _ => element.set_text_content(Some("☐")),
            }
        },
        None => element.set_text_content(Some("☐")),
    }
}

#[wasm_bindgen]
pub async fn twitch_api_test() {
    weblog!("twitch_api_test started");

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    let elem = &document.get_element_by_id("twitch_api_test_output").unwrap();

    if let Ok(p) = elem.clone().dyn_into::<web_sys::HtmlParagraphElement>() {
        p.set_inner_text("API test started 333");
    }

    weblog!("twitch_api_test end reached");
}

#[wasm_bindgen]
pub fn store_twitch_api_client() {
    weblog!("store_twitch_api_client started");

    console_error_panic_hook::set_once();

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    let client_id_input = &document.get_element_by_id("twitch_api_client_id").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
    let client_secret_input = &document.get_element_by_id("twitch_api_client_secret").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();

    let cfg = SConfigStore::new();
    cfg.save_twitch_client(client_id_input.value().as_str(), client_secret_input.value().as_str());

    weblog!("store_twitch_api_client reached end");
}
