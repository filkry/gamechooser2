use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use gamechooser_core;


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
pub fn send_random_game_query() {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    let _couch = ETagQuery::new_from_element(&document.get_element_by_id("couch"));
    let _portable = ETagQuery::new_from_element(&document.get_element_by_id("portable"));
    let _shortok = ETagQuery::new_from_element(&document.get_element_by_id("shortok"));
    let _longok = ETagQuery::new_from_element(&document.get_element_by_id("longok"));
}

#[wasm_bindgen]
pub fn twitch_api_test() {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    let elem = &document.get_element_by_id("twitch_api_test_output").unwrap();

    if let Ok(p) = elem.clone().dyn_into::<web_sys::HtmlParagraphElement>() {
        p.set_inner_text("API test started");
    }

}
