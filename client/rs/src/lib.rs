mod web;

//use console_error_panic_hook;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};
use web_sys::{HtmlDivElement, HtmlImageElement, HtmlInputElement, HtmlParagraphElement, HtmlSpanElement};

use gamechooser_core as core;
use web::{document, window, TToJsError, TErgonomicDocument};

macro_rules! weblog {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[allow(dead_code)]
enum ETagQuery {
    TrueOrFalse,
    True,
    False,
}

#[allow(dead_code)]
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

// -- I think this is necessary for something
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

async fn call_test() -> Result<String, JsError> {
    let window = web_sys::window().expect("no global `window` exists");

    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);

    let url = format!("http://localhost:8000/test");
    //let request = Request::new_with_str_and_init(&url, &opts).stomp_err(String::from("Failed to make request"))?;
    let request = Request::new_with_str_and_init(&url, &opts).to_jserr()?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.to_jserr()?;
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().to_jserr()?;
    let text_promise = resp.text().to_jserr()?;
    let text = JsFuture::from(text_promise).await.to_jserr()?;

    let text_string = text.as_string().ok_or(JsError::new("text was not a string"))?;

    Ok(text_string)
}

#[wasm_bindgen]
pub async fn twitch_api_test() -> Result<(), JsError> {
    weblog!("twitch_api_test started");

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    let elem = &document.get_element_by_id("twitch_api_test_output").unwrap();

    if let Ok(p) = elem.clone().dyn_into::<web_sys::HtmlParagraphElement>() {
        p.set_inner_text("API test started 333");
        let response_string = call_test().await?;
        p.set_inner_text(response_string.as_str());
    }

    weblog!("twitch_api_test end reached");

    Ok(())
}

#[wasm_bindgen]
pub async fn search_igdb() -> Result<(), JsError> {
    let window = window();
    let document = document();

    // -- do the request
    let games : Vec<core::SGame> = {
    //let text_string = {
        let name_search_input = &document.get_typed_element_by_id::<HtmlInputElement>("name_search_string")?;

        let mut opts = RequestInit::new();
        opts.method("POST");
        opts.mode(RequestMode::Cors);

        let url = format!("http://localhost:8000/search_igdb/{}", name_search_input.value().as_str());
        let request = Request::new_with_str_and_init(&url, &opts).or(Err(JsError::new("Failed to create request")))?;

        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.or(Err(JsError::new("Fetch failed")))?;
        assert!(resp_value.is_instance_of::<Response>());
        let resp: Response = resp_value.dyn_into().or(Err(JsError::new("resp_value is not a Response")))?;

        let json_promise = resp.json().or(Err(JsError::new("Could not get json promise from Response")))?;
        let json = JsFuture::from(json_promise).await.or(Err(JsError::new("Could not convert response to json")))?;

        json.into_serde().unwrap()
    };

    let output_elem = document.get_typed_element_by_id::<HtmlDivElement>("search_igdb_output")?;
    output_elem.set_inner_html("");
    for game in &games {
        let game_div = document.create_element_typed::<HtmlDivElement>()?;
        output_elem.append_child(&game_div).to_jserr()?;

        let title_elem = document.create_element("h3").to_jserr()?;
        title_elem.set_text_content(Some(game.title()));
        game_div.append_child(&title_elem).to_jserr()?;

        if let Some(url) = game.cover_url() {
            let img_elem = document.create_element_typed::<HtmlImageElement>()?;
            img_elem.set_src(url);
            game_div.append_child(&img_elem).to_jserr()?;
        }
    }

    Ok(())
}
