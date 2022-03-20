use console_error_panic_hook;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

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
pub fn twitch_api_test() {
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
