use console_error_panic_hook;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

use gamechooser_core as core;

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

trait TUnpackJSError {
    type OkType;

    fn js_error(self) -> Result<Self::OkType, String>;
}

fn js_error_to_string(js_error: JsValue) -> String {
    format!("{:?}", js_error)
}

impl<T> TUnpackJSError for Result<T, JsValue> {
    type OkType = T;

    fn js_error(self) -> Result<Self::OkType, String> {
        self.map_err(js_error_to_string)
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

async fn call_test() -> Result<String, String> {
    let window = web_sys::window().expect("no global `window` exists");

    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);

    let url = format!("http://localhost:8000/test");
    //let request = Request::new_with_str_and_init(&url, &opts).stomp_err(String::from("Failed to make request"))?;
    let request = Request::new_with_str_and_init(&url, &opts).js_error()?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.stomp_err(String::from("Fetch failed"))?;
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().stomp_err(String::from("resp was not a Response"))?;
    let text_promise = resp.text().stomp_err(String::from("Couldn't get text from response - was it bad?"))?;
    let text = JsFuture::from(text_promise).await.stomp_err(String::from("Couldn't get text from response"))?;

    let text_string = text.as_string().ok_or(String::from("text was not a string"))?;

    Ok(text_string)
}

#[wasm_bindgen]
pub async fn twitch_api_test() {
    weblog!("twitch_api_test started");

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    let elem = &document.get_element_by_id("twitch_api_test_output").unwrap();

    if let Ok(p) = elem.clone().dyn_into::<web_sys::HtmlParagraphElement>() {
        p.set_inner_text("API test started 333");

        let response_string = call_test().await;

        if let Err(e) = response_string {
            weblog!("test failed with error: {:?}", e);
            return;
        }

        p.set_inner_text(response_string.unwrap().as_str());
    }

    weblog!("twitch_api_test end reached");
}

#[wasm_bindgen]
pub async fn search_igdb() -> Result<String, String> {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    // -- do the request
    let games : Vec<core::SGame> = {
    //let text_string = {
        let name_search_string = &document.get_element_by_id("name_search_string").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();

        let mut opts = RequestInit::new();
        opts.method("POST");
        opts.mode(RequestMode::Cors);

        let url = format!("http://localhost:8000/search_igdb/{}", name_search_string.value().as_str());
        let request = Request::new_with_str_and_init(&url, &opts).js_error()?;

        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.stomp_err(String::from("Fetch failed"))?;
        assert!(resp_value.is_instance_of::<Response>());
        let resp: Response = resp_value.dyn_into().stomp_err(String::from("resp was not a Response"))?;

        /*
        let text_promise = resp.text().stomp_err(String::from("Couldn't get text from response - was it bad?"))?;
        let text = JsFuture::from(text_promise).await.stomp_err(String::from("Couldn't get text from response"))?;

        text.as_string().ok_or(String::from("text was not a string"))?
        */

        let json_promise = resp.json().js_error()?;
        let json = JsFuture::from(json_promise).await.js_error()?;

        json.into_serde().unwrap()
    };

    let mut text_string = String::new();
    for game in &games {
        text_string.push_str(game.title());
        text_string.push_str("\n");
    }

    let output_elem = &document.get_element_by_id("search_igdb_output").unwrap().dyn_into::<web_sys::HtmlParagraphElement>().unwrap();
    output_elem.set_inner_text(text_string.as_str());

    Ok(text_string)
}
