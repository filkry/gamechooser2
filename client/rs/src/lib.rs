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

/*
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
*/

struct SFetchTwitchAPIClient {

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

impl SFetchTwitchAPIClient {
    fn prepare_request(rb: gamechooser_core::STwitchAPIRequestBuilder) -> Request {
        let mut opts = RequestInit::new();
        opts.method("POST");
        opts.mode(RequestMode::Cors);

        if let Some(b) = rb.body {
            let body = JsValue::from_str(b.as_str());
            opts.body(Some(&body));
        }

        let request = Request::new_with_str_and_init(&rb.url, &opts).unwrap();

        for (hname, hvalue) in rb.headers {
            request.headers().append(hname.as_str(), hvalue.as_str()).unwrap();
        }

        request
    }
}

trait TStompErr {
    type OkType;

    fn stomp_err<N>(self, e: N) -> Result<Self::OkType, N>;
}

impl<T, E> TStompErr for Result<T, E> {
    type OkType = T;

    fn stomp_err<N>(self, e: N) -> Result<Self::OkType, N> {
        match self {
            Ok(t) => Ok(t),
            Err(_) => Err(e),
        }
    }
}

#[async_trait(?Send)]
impl gamechooser_core::TTwitchAPIClient for SFetchTwitchAPIClient {
    type Session = gamechooser_core::STwitchOauthTokenResponse;

    async fn init(params: gamechooser_core::STwitchOauthTokenRequest) -> Result<Self::Session, String> {
        let mut opts = RequestInit::new();

        opts.method("POST");
        opts.mode(RequestMode::Cors);
        /*
        let params_value = {
            let temp = JsValue::from_serde(&params);
            match temp {
                Ok(res) => res,
                Err(_) => return Err(String::from("Failed to create JsValue from params."))
            }
        };
        */

        //opts.body(Some(&params_value));

        //let url = "https://id.twitch.tv/oauth2/token";

        let url = format!("https://id.twitch.tv/oauth2/token?client_id={}&client_secret={}&grant_type=client_credentials", params.client_id, params.client_secret);

        let request = Request::new_with_str_and_init(&url, &opts).stomp_err(String::from("Failed to make request"))?;

        let window = web_sys::window().ok_or(String::from("Failed to find window"))?;
        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.stomp_err(String::from("Fetch failed"))?;

        assert!(resp_value.is_instance_of::<Response>());
        let resp: Response = resp_value.dyn_into().stomp_err(String::from("resp was not a Response"))?;

        let json_promise = resp.json().stomp_err(String::from("Couldn't get json from response - was it bad?"))?;

        let json = JsFuture::from(json_promise).await.stomp_err(String::from("Couldn't get json from response"))?;
        let res : gamechooser_core::STwitchOauthTokenResponse = json.into_serde().stomp_err(String::from("Could not resolve response into expected type"))?;

        Ok(res)
    }

    async fn post_interp_json<T: DeserializeOwned>(_session: Self::Session, rb: gamechooser_core::STwitchAPIRequestBuilder) -> Result<T, String> {
        let req = Self::prepare_request(rb);

        let window = web_sys::window().unwrap();
        let resp_value = JsFuture::from(window.fetch_with_request(&req)).await.unwrap();

        assert!(resp_value.is_instance_of::<Response>());
        let resp: Response = resp_value.dyn_into().unwrap();

        let json = JsFuture::from(resp.json().unwrap()).await.unwrap();
        let res : T = json.into_serde().unwrap();

        Ok(res)
    }

    async fn post_text(_session: Self::Session, rb: gamechooser_core::STwitchAPIRequestBuilder) -> Result<String, String> {
        let req = Self::prepare_request(rb);

        let window = web_sys::window().unwrap();
        let resp_value = JsFuture::from(window.fetch_with_request(&req)).await.unwrap();

        assert!(resp_value.is_instance_of::<Response>());
        let resp: Response = resp_value.dyn_into().unwrap();

        let text = JsFuture::from(resp.text().unwrap()).await.unwrap();
        Ok(text.as_string().unwrap())
    }

    fn access_token(session: &Self::Session) -> &str {
        &session.access_token
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

        let cfg = SConfigStore::new();
        let response_string = gamechooser_core::test_any_client::<SFetchTwitchAPIClient, SConfigStore>(&cfg).await;

        if let Err(e) = response_string {
            weblog!("test failed with error: {:?}", e);
            return;
        }

        p.set_inner_text(response_string.unwrap().as_str());
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
