mod web;
mod server_api;

use std::sync::RwLock;

//use console_error_panic_hook;
use once_cell::sync::Lazy;
use js_sys::{Date, Function};
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};
use web_sys::{HtmlButtonElement, HtmlDivElement, HtmlElement, HtmlImageElement, HtmlInputElement, HtmlParagraphElement, HtmlSpanElement};

use gamechooser_core as core;
use web::{document, TToJsError, TErgonomicDocument};

macro_rules! weblog {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[derive(Copy, Clone)]
enum EGameEditMode {
    Add,
    Edit,
}

struct SAppState {
    collection_recent_games: Option<Vec<core::SCollectionGame>>,

    last_search_igdb_results: Option<Vec<core::SGame>>,

    game_edit_game: Option<core::SCollectionGame>,
    game_edit_mode: EGameEditMode,
}

#[allow(dead_code)]
enum ETagQuery {
    TrueOrFalse,
    True,
    False,
}

static APP: Lazy<RwLock<SAppState>> = Lazy::new(|| RwLock::new(SAppState::new()));

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

impl SAppState {
    pub fn new() -> Self {
        Self {
            last_search_igdb_results: None,
            collection_recent_games: None,
            game_edit_mode: EGameEditMode::Add,
            game_edit_game: None,
        }
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
pub async fn search_igdb() -> Result<(), JsError> {
    let document = document();

    // -- do the request
    let name_search_input = &document.get_typed_element_by_id::<HtmlInputElement>("name_search_string")?;
    let games : Vec<core::SGame> = server_api::search_igdb(name_search_input.value().as_str()).await?;

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

fn div(id: &str) -> Result<HtmlDivElement, JsError> {
    document().get_typed_element_by_id::<HtmlDivElement>(id).or(Err(JsError::new(format!("could not find {}", id).as_str())))
}

fn swap_section_div(tgt_id: &str) -> Result<(), JsError> {
    div("sessions_div")?.style().set_property("display", "none").to_jserr()?;
    div("collection_div")?.style().set_property("display", "none").to_jserr()?;
    div("add_div")?.style().set_property("display", "none").to_jserr()?;
    div("randomizer_div")?.style().set_property("display", "none").to_jserr()?;
    div("stats_div")?.style().set_property("display", "none").to_jserr()?;
    div("game_edit_div")?.style().set_property("display", "none").to_jserr()?;
    div("result_div")?.style().set_property("display", "none").to_jserr()?;

    div(tgt_id)?.style().set_property("display", "block").to_jserr()?;

    Ok(())
}

#[wasm_bindgen]
pub fn show_sessions() -> Result<(), JsError> {
    swap_section_div("sessions_div")
}

#[wasm_bindgen]
pub async fn show_collection() -> Result<(), JsError> {
    enter_collection_screen().await
}

#[wasm_bindgen]
pub fn show_add() -> Result<(), JsError> {
    swap_section_div("add_div")
}

#[wasm_bindgen]
pub fn show_randomizer() -> Result<(), JsError> {
    swap_section_div("randomizer_div")
}

#[wasm_bindgen]
pub fn show_stats() -> Result<(), JsError> {
    swap_section_div("stats_div")
}

#[wasm_bindgen]
pub async fn add_screen_search_igdb() -> Result<(), JsError> {
    let document = document();

    // -- do the request
    let name_search_input = &document.get_typed_element_by_id::<HtmlInputElement>("add_screen_name_search_input")?;
    let games : Vec<core::SGame> = server_api::search_igdb(name_search_input.value().as_str()).await?;

    let output_elem = document.get_typed_element_by_id::<HtmlDivElement>("add_screen_search_igdb_output")?;
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

        let button_elem = document.create_element_typed::<HtmlButtonElement>()?;
        let onclick_body = format!("add_screen_add_result({});", game.igdb_id().expect("IGDB results should have an igdb_id"));
        let onclick = Function::new_no_args(onclick_body.as_str());
        button_elem.set_onclick(Some(&onclick));
        button_elem.set_inner_text("Add");
        game_div.append_child(&button_elem).to_jserr()?;
    }

    // -- cache results for later use
    {
        let mut app = APP.try_write().expect("Should never actually have contention.");
        app.last_search_igdb_results = Some(games);
    }

    Ok(())
}

#[wasm_bindgen]
pub fn add_screen_add_result(igdb_id: u32) -> Result<(), JsError> {
    let game_opt = {
        let mut result = None;

        let app = APP.try_read().expect("Should never actually have contention");
        if let Some(games) = &app.last_search_igdb_results {
            for g in games {
                if let Some(inner_id) = g.igdb_id() {
                    if *inner_id == igdb_id {
                        result = Some(g.clone());
                        break;
                    }
                }
            }
        }

        result
    };

    if game_opt.is_none() {
        return Err(JsError::new("Somehow adding an IGDB game that was not in search results."))
    }
    let game = game_opt.expect("checked above");

    let header_elem = document().get_typed_element_by_id::<HtmlElement>("game_edit_header")?;
    header_elem.set_inner_text("Add game");
    let submit_elem = document().get_typed_element_by_id::<HtmlElement>("game_edit_submit")?;
    submit_elem.set_inner_text("Add");

    let title_elem = document().get_typed_element_by_id::<HtmlInputElement>("game_edit_title")?;
    title_elem.set_value(game.title());

    let date_elem = document().get_typed_element_by_id::<HtmlInputElement>("game_edit_release_date")?;
    if let Some(date) = game.release_date() {
        let date_str = date.format("%Y-%m-%d").to_string();
        date_elem.set_value(date_str.as_str());
    }
    else {
        let date = Date::new_0();
        let date_str = format!("{:04}-{:02}-{:02}", date.get_full_year(), date.get_month(), date.get_date());
        date_elem.set_value(date_str.as_str());
    }

    let cover_elem = document().get_typed_element_by_id::<HtmlImageElement>("game_edit_cover_art")?;
    if let Some(url) = game.cover_url() {
        cover_elem.set_src(url);
        cover_elem.style().set_property("display", "block").to_jserr()?;
    }
    else {
        cover_elem.style().set_property("display", "none").to_jserr()?;
    }

    {
        let mut app = APP.try_write().expect("Should never actually have contention.");
        app.game_edit_mode = EGameEditMode::Add;
        app.game_edit_game = Some(core::SCollectionGame::new(game));
    }

    swap_section_div("game_edit_div")?;

    Ok(())
}

async fn edit_screen_submit_edit_helper() -> Result<(), JsError> {
    let mut game = {
        let mut app = APP.try_write().expect("Should never actually have contention.");
        app.game_edit_game.take()
    }.ok_or(JsError::new("Trying to submit, but edit_game that does not exist"))?;

    game.game_mut().set_title(document().get_typed_element_by_id::<HtmlInputElement>("game_edit_title")?.value().as_str());
    let date_str = document().get_typed_element_by_id::<HtmlInputElement>("game_edit_release_date")?.value();
    if let Err(_) = game.game_mut().set_release_date_str(date_str.as_str()) {
        return Err(JsError::new("Could not parse date from game_edit_release_date element."));
    }

    game.info_mut().via = document().get_typed_element_by_id::<HtmlInputElement>("game_edit_via")?.value();

    fn checkbox_value(id: &str) -> Result<bool, JsError> {
        Ok(document().get_typed_element_by_id::<HtmlInputElement>(id)?.checked())
    }

    game.info_mut().tags = core::SGameTags {
        couch_playable: Some(checkbox_value("game_edit_tag_couch")?),
        portable_playable: Some(checkbox_value("game_edit_tag_portable")?),
    };

    game.info_mut().own = core::SOwn {
        steam: checkbox_value("game_edit_own_steam")?,
        egs: checkbox_value("game_edit_own_egs")?,
        emulator: checkbox_value("game_edit_own_emulator")?,
        ds: checkbox_value("game_edit_own_ds")?,
        n3ds: checkbox_value("game_edit_own_n3ds")?,
        wii: checkbox_value("game_edit_own_wii")?,
        wiiu: checkbox_value("game_edit_own_wiiu")?,
        switch: checkbox_value("game_edit_own_switch")?,
        ps4: checkbox_value("game_edit_own_ps4")?,
        ps5: checkbox_value("game_edit_own_ps5")?,
    };

    let mode = APP.try_read().expect("Should never actually have contention.").game_edit_mode;
    match mode {
        EGameEditMode::Add => {
            server_api::add_game(game.clone()).await?;
        },
        EGameEditMode::Edit => {
            server_api::edit_game(game.clone()).await?;
        },
    }

    weblog!("Edit collection game: {:?}", game);

    Ok(())
}

#[wasm_bindgen]
pub async fn edit_screen_submit_edit() -> Result<(), JsError> {
    let p = document().get_typed_element_by_id::<HtmlParagraphElement>("result_message")?;
    match edit_screen_submit_edit_helper().await {
        Ok(_) => {
            p.set_inner_text("Successfully added game.");
        }
        Err(e) => {
            p.set_inner_text("Failed to add game, error in console.");
            return Err(e);
        }
    }
    swap_section_div("result_div")?;
    Ok(())
}

async fn enter_collection_screen() -> Result<(), JsError> {
    let document = web::document();
    let games = server_api::get_recent_collection_games().await?;

    let output_elem = document.get_typed_element_by_id::<HtmlDivElement>("collection_screen_game_list")?;
    output_elem.set_inner_html("");
    for game in &games {
        let game_div = document.create_element_typed::<HtmlDivElement>()?;
        output_elem.append_child(&game_div).to_jserr()?;

        let title_elem = document.create_element("h3").to_jserr()?;
        title_elem.set_text_content(Some(game.game.title()));
        game_div.append_child(&title_elem).to_jserr()?;

        if let Some(url) = game.game.cover_url() {
            let img_elem = document.create_element_typed::<HtmlImageElement>()?;
            img_elem.set_src(url);
            game_div.append_child(&img_elem).to_jserr()?;
        }

        let button_elem = document.create_element_typed::<HtmlButtonElement>()?;
        let onclick_body = format!("collection_screen_edit_game({});", game.game.internal_id().expect("Games in collection should have internal ID"));
        let onclick = Function::new_no_args(onclick_body.as_str());
        button_elem.set_onclick(Some(&onclick));
        button_elem.set_inner_text("Edit");
        game_div.append_child(&button_elem).to_jserr()?;
    }

    // -- cache results for later use
    {
        let mut app = APP.try_write().expect("Should never actually have contention.");
        app.collection_recent_games = Some(games);
    }

    swap_section_div("collection_div")?;

    Ok(())
}