mod game_card;
mod stats;
mod web;
mod server_api;

use std::collections::HashMap;
use std::sync::RwLock;

//use console_error_panic_hook;
use chrono;
use once_cell::sync::Lazy;
use js_sys::{Function};
use wasm_bindgen::prelude::*;
use web_sys::{
    Element,
    //HtmlAnchorElement,
    HtmlButtonElement,
    HtmlDivElement,
    HtmlElement,
    HtmlImageElement,
    HtmlInputElement,
    HtmlLabelElement,
    HtmlLiElement,
    HtmlParagraphElement,
    HtmlSelectElement,
    //HtmlSpanElement,
    HtmlUListElement,
};

use gamechooser_core as core;
use game_card::{SGameCard, SCompactGameCard};
use web::{document, TToJsError, TErgonomicDocument};

macro_rules! weblog {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

enum EGameEdit {
    None,
    Add(core::SAddCollectionGame),
    Edit(u32),
}

struct SGameRandomizerSession {
    shuffled_internal_ids: Vec<u32>,
    cur_idx: usize,
}

enum EGameRandomizer {
    Uninit,
    Choosing(SGameRandomizerSession),
}

struct SAppState {
    collection_game_cache: HashMap<u32, core::SCollectionGame>,

    session_screen_sessions: Vec<core::SSession>,

    last_search_igdb_results: Option<Vec<core::SSearchIGDBResult>>,

    details_screen_game: Option<u32>,

    game_edit: EGameEdit,

    game_randomizer: EGameRandomizer,
}

#[allow(dead_code)]
enum ETagQuery {
    TrueOrFalse,
    True,
    False,
}

static APP: Lazy<RwLock<SAppState>> = Lazy::new(|| RwLock::new(SAppState::new()));

impl SAppState {
    pub fn new() -> Self {
        Self {
            collection_game_cache: HashMap::new(),
            session_screen_sessions: Vec::new(),
            last_search_igdb_results: None,
            details_screen_game: None,
            game_edit: EGameEdit::None,
            game_randomizer: EGameRandomizer::Uninit,
        }
    }
}

impl Default for EGameEdit {
    fn default() -> Self {
        Self::None
    }
}

impl EGameEdit{
    fn header(&self) -> &str {
        match self {
            Self::None => "INVALID STATE",
            Self::Add(_) => "Add game",
            Self::Edit(_) => "Edit game",
        }
    }

    fn submit_button_text(&self) -> &str {
        match self {
            Self::None => "INVALID STATE",
            Self::Add(_) => "Submit add",
            Self::Edit(_) => "Submit edit",
        }
    }
}

impl Default for EGameRandomizer {
    fn default() -> Self {
        Self::Uninit
    }
}

// -- I think this is necessary for something
#[wasm_bindgen]
extern {
}

fn element(id: &str) -> Result<HtmlElement, JsError> {
    document().get_typed_element_by_id::<HtmlElement>(id).or(Err(JsError::new(format!("could not find {}", id).as_str())))
}

fn div(id: &str) -> Result<HtmlDivElement, JsError> {
    document().get_typed_element_by_id::<HtmlDivElement>(id).or(Err(JsError::new(format!("could not find {}", id).as_str())))
}

fn swap_section_div(tgt_id: &str) -> Result<(), JsError> {
    div("sessions_div")?.style().set_property("display", "none").to_jserr()?;
    div("collection_div")?.style().set_property("display", "none").to_jserr()?;
    div("full_collection_div")?.style().set_property("display", "none").to_jserr()?;
    div("add_div")?.style().set_property("display", "none").to_jserr()?;
    div("randomizer_div")?.style().set_property("display", "none").to_jserr()?;
    div("stats_div")?.style().set_property("display", "none").to_jserr()?;
    div("game_details_div")?.style().set_property("display", "none").to_jserr()?;
    div("game_edit_div")?.style().set_property("display", "none").to_jserr()?;
    div("result_div")?.style().set_property("display", "none").to_jserr()?;
    div("login_div")?.style().set_property("display", "none").to_jserr()?;

    div(tgt_id)?.style().set_property("display", "block").to_jserr()?;

    if tgt_id.eq("login_div") {
        div("main_nav_div")?.style().set_property("display", "none").to_jserr()?;
    }
    else {
        div("main_nav_div")?.style().set_property("display", "block").to_jserr()?;
    }

    Ok(())
}

#[wasm_bindgen]
pub async fn initial_load() -> Result<(), JsError> {
    let sl = SShowLoadingHelper::new();
    if server_api::check_logged_in().await {
        drop(sl);
        show_sessions().await?;
    }
    else {
        show_login().await?;
    }

    Ok(())
}

#[wasm_bindgen]
pub async fn show_login() -> Result<(), JsError> {
    swap_section_div("login_div")
}

#[wasm_bindgen]
pub async fn show_sessions() -> Result<(), JsError> {
    enter_sessions_screen().await
}

#[wasm_bindgen]
pub async fn show_collection() -> Result<(), JsError> {
    enter_collection_screen().await
}

#[wasm_bindgen]
pub async fn show_full_collection() -> Result<(), JsError> {
    enter_full_collection_screen().await
}

#[wasm_bindgen]
pub fn show_add() -> Result<(), JsError> {
    swap_section_div("add_div")
}

#[wasm_bindgen]
pub fn show_randomizer() -> Result<(), JsError> {
    div("randomizer_game_div")?.style().set_property("display", "none").to_jserr()?;
    swap_section_div("randomizer_div")
}

#[wasm_bindgen]
pub async fn show_stats() -> Result<(), JsError> {
    let sl = SShowLoadingHelper::new();
    let stats = match server_api::simple_stats().await {
        Ok(s) => s,
        Err(e) => {
            show_error(e)?;
            return Ok(());
        }
    };
    drop(sl);

    let stats_div = div("stats_container")?;
    stats_div.set_inner_text("");

    let document = document();

    let append_stat_header = |tag: &str, title: &str| -> Result<(), JsError> {
        let h3 = document.create_element(tag).to_jserr()?;
        h3.set_text_content(Some(title));
        stats_div.append_child(&h3).to_jserr()?;
        Ok(())
    };

    append_stat_header("h2", format!("Collection ({})", stats.total_collection_size).as_str())?;

    append_stat_header("h3", "Released")?;
    stats::create_binary_percentage_chart(&stats_div, stats.collection_released, stats.total_collection_size)?;

    append_stat_header("h3", "Selectable")?;
    let classes = [stats.collection_selectable, stats.collection_retired, stats.collection_passed_many_times, stats.collection_cooldown];
    let class_titles = ["selectable", "retired", "passed", "cooldown"];
    stats::create_class_percentage_chart(&stats_div, &classes, Some(&class_titles))?;

    append_stat_header("h3", "Owned")?;
    stats::create_binary_percentage_chart(&stats_div, stats.collection_owned, stats.total_collection_size)?;

    append_stat_header("h3", "Played before")?;
    stats::create_binary_percentage_chart(&stats_div, stats.collection_played_before, stats.total_collection_size)?;

    append_stat_header("h3", "Japanese practice tag")?;
    stats::create_binary_percentage_chart(&stats_div, stats.collection_japanese_practice_tag, stats.total_collection_size)?;

    append_stat_header("h3", "Portable playable tag")?;
    stats::create_binary_percentage_chart(&stats_div, stats.collection_portable_playable_tag, stats.total_collection_size)?;

    append_stat_header("h2", format!("Selectable ({})", stats.collection_selectable).as_str())?;

    append_stat_header("h3", "Owned")?;
    stats::create_binary_percentage_chart(&stats_div, stats.selectable_owned, stats.collection_selectable)?;

    append_stat_header("h3", "Played before")?;
    stats::create_binary_percentage_chart(&stats_div, stats.selectable_played_before, stats.collection_selectable)?;

    append_stat_header("h3", "Japanese practice tag")?;
    stats::create_binary_percentage_chart(&stats_div, stats.selectable_japanese_practice_tag, stats.collection_selectable)?;

    append_stat_header("h3", "Portable playable tag")?;
    stats::create_binary_percentage_chart(&stats_div, stats.selectable_portable_playable_tag, stats.collection_selectable)?;

    swap_section_div("stats_div")
}

fn show_loading(show: bool) -> Result<(), JsError> {
    if show {
        element("popup_overlay")?.style().set_property("display", "block").to_jserr()?;
        element("loading_message")?.style().set_property("display", "block").to_jserr()?;
    }
    else {
        element("popup_overlay")?.style().set_property("display", "none").to_jserr()?;
        element("loading_message")?.style().set_property("display", "none").to_jserr()?;
    }

    Ok(())
}

struct SShowLoadingHelper {
}

impl SShowLoadingHelper {
    fn new() -> Self {
        if let Err(_) = show_loading(true) {
            weblog!("show_loading failed");
        }
        Self {}
    }
}

impl Drop for SShowLoadingHelper {
    fn drop(&mut self) {
        if let Err(_) = show_loading(false) {
            weblog!("show_loading failed");
        }
    }
}

#[wasm_bindgen]
pub async fn login_screen_submit() -> Result<(), JsError> {
    let sec_input : HtmlInputElement = document().get_typed_element_by_id::<HtmlInputElement>("login_screen_secret").to_jserr()?;
    let sl = SShowLoadingHelper::new();
    if server_api::login(sec_input.value().as_str()).await.to_jserr().is_ok() {
        drop(sl);
        show_sessions().await?;
    }

    Ok(())
}

#[wasm_bindgen]
pub async fn add_screen_search_igdb() -> Result<(), JsError> {
    let document = document();

    // -- do the request
    let name_search_input = &document.get_typed_element_by_id::<HtmlInputElement>("add_screen_name_search_input").to_jserr()?;
    let sl = SShowLoadingHelper::new();
    let games : Vec<core::SSearchIGDBResult> = match server_api::search_igdb(
        name_search_input.value().as_str(),
        checkbox_value("add_screen_games_only")?,
    ).await {

        Ok(g) => g,
        Err(e) => {
            show_error(e)?;
            return Ok(());
        },
    };
    drop(sl);

    let output_elem = div("add_screen_search_igdb_output")?;
    output_elem.set_inner_html("");

    if games.len() == 0 {
        output_elem.set_inner_text("No results.");
        return Ok(())
    }

    for game in &games {
        let result_elem = document.create_element_typed::<HtmlDivElement>().to_jserr()?;
        output_elem.append_child(&result_elem).to_jserr()?;

        let mut game_div = SGameCard::new_from_game_info(&game.game_info)?;
        game_div
            .show_release_date()
            .show_igdb_link()
            .regen()?;

        result_elem.append_child(&game_div.main_div).to_jserr()?;

        let customizable_div = game_div.customizable_div()?;

        if game.in_collection {
            let warning_elem = document.create_element_typed::<HtmlDivElement>().to_jserr()?;
            warning_elem.set_class_name("add_game_duplicate");
            warning_elem.set_inner_text("⚠ Already in collection️");
            customizable_div.append_child(&warning_elem).to_jserr()?;
        }

        let button_elem = document.create_element_typed::<HtmlButtonElement>().to_jserr()?;
        let onclick_body = format!("add_screen_add_result({});", game.game_info.igdb_id().expect("IGDB results should have an igdb_id"));
        let onclick = Function::new_no_args(onclick_body.as_str());
        button_elem.set_onclick(Some(&onclick));
        button_elem.set_inner_text("Add");
        customizable_div.append_child(&button_elem).to_jserr()?;
    }

    // -- cache results for later use
    {
        let mut app = APP.try_write().expect("Should never actually have contention.");
        app.last_search_igdb_results = Some(games);
    }

    Ok(())
}

#[allow(dead_code)]
fn populate_inner_text(id: &str, value: &str) -> Result<(), JsError> {
    let elem = document().get_typed_element_by_id::<HtmlElement>(id).to_jserr()?;
    elem.set_inner_text(value);
    Ok(())
}

fn populate_text_input(id: &str, value: &str) -> Result<(), JsError> {
    let elem = document().get_typed_element_by_id::<HtmlInputElement>(id).to_jserr()?;
    elem.set_value(value);
    Ok(())
}

fn populate_date_input(id: &str, value: Option<chrono::naive::NaiveDate>) -> Result<(), JsError> {
    let date_elem = document().get_typed_element_by_id::<HtmlInputElement>(id).to_jserr()?;
    let date = match value {
        Some(d) => d,
        None => chrono::offset::Local::now().naive_local().date(),
    };
    let date_str = date.format("%Y-%m-%d").to_string();
    date_elem.set_value(date_str.as_str());
    Ok(())
}

fn populate_release_date_input(id: &str, value: core::EReleaseDate) -> Result<(), JsError> {
    let type_id = format!("{}_type", id);
    let date_id = format!("{}_date", id);

    let date_type_elem = document().get_typed_element_by_id::<HtmlSelectElement>(type_id.as_str()).to_jserr()?;
    let date_date_elem = document().get_typed_element_by_id::<HtmlInputElement>(date_id.as_str()).to_jserr()?;

    match value {
        core::EReleaseDate::UnknownUnreleased => {
            date_type_elem.set_value("unknown_unreleased");

            let d = chrono::offset::Local::now().naive_local().date();
            date_date_elem.style().set_property("display", "none").to_jserr()?;
            let date_str = d.format("%Y-%m-%d").to_string();
            date_date_elem.set_value(date_str.as_str());
        },
        core::EReleaseDate::UnknownReleased => {
            date_type_elem.set_value("unknown_released");

            let d = chrono::offset::Local::now().naive_local().date();
            date_date_elem.style().set_property("display", "none").to_jserr()?;
            let date_str = d.format("%Y-%m-%d").to_string();
            date_date_elem.set_value(date_str.as_str());
        },
        core::EReleaseDate::Known(d) => {
            date_type_elem.set_value("known");

            date_date_elem.style().set_property("display", "block").to_jserr()?;
            let date_str = d.format("%Y-%m-%d").to_string();
            date_date_elem.set_value(date_str.as_str());
        },
    };
    Ok(())
}

fn populate_checkox_input(id: &str, value: bool) -> Result<(), JsError> {
    let elem = document().get_typed_element_by_id::<HtmlInputElement>(id).to_jserr()?;
    elem.set_checked(value);
    Ok(())
}

fn populate_number_input(id: &str, value: f64) -> Result<(), JsError> {
    let elem = document().get_typed_element_by_id::<HtmlInputElement>(id).to_jserr()?;
    elem.set_value_as_number(value);
    Ok(())
}

fn populate_img(id: &str, src: Option<&str>) -> Result<(), JsError> {
    let elem = document().get_typed_element_by_id::<HtmlImageElement>(id).to_jserr()?;
    if let Some(url) = src {
        elem.set_src(url);
        elem.style().set_property("display", "block").to_jserr()?;
    }
    else {
        elem.style().set_property("display", "none").to_jserr()?;
    }

    Ok(())
}

async fn view_details(game: core::SCollectionGame) -> Result<(), JsError> {
    let mut card = SGameCard::new_from_collection_game(&game)?;
    card.show_all().regen()?;

    let card_div = div("game_details_card")?;
    card_div.set_inner_html("");
    card_div.append_child(&card.main_div).to_jserr()?;

    let _sl = SShowLoadingHelper::new();
    let sessions = match server_api::get_sessions(
        Some(game.internal_id),
        false,
        false,
        None,
    ).await {
        Ok(s) => s,
        Err(e) => {
            show_error(e)?;
            return Ok(());
        }
    };

    for _session in sessions {
        div("game_details_sessions")?.set_inner_text("At least one session exists, but session details list has not yet been implemented.");
    }

    {
        let mut app = APP.try_write().expect("Should never actually have contention.");
        let internal_id = game.internal_id;
        app.collection_game_cache.insert(internal_id, game);
        app.details_screen_game = Some(internal_id);
    }

    swap_section_div("game_details_div")?;

    Ok(())
}

fn edit_screen_populate_game_info(game_info: &core::EGameInfo) -> Result<(), JsError> {
    populate_text_input("game_edit_title", game_info.title())?;
    populate_release_date_input("game_edit_release_date", game_info.release_date())?;
    let cover_url = game_info.cover_url();
    populate_img("game_edit_cover_art", cover_url.as_ref().map(|u| u.as_str()))?;

    Ok(())
}

fn edit_screen_populate_custom_info(custom_info: &core::SGameCustomInfo) -> Result<(), JsError> {

    populate_text_input("game_edit_via", custom_info.via.as_str())?;

    let output_tag = div("game_edit_tags")?;
    let output_own = div("game_edit_own")?;

    output_tag.set_inner_html("");
    output_own.set_inner_html("");

    let output_tag_ul = document().create_element_typed::<HtmlUListElement>().to_jserr()?;
    output_tag_ul.set_class_name("checkbox_list");
    output_tag.append_child(&output_tag_ul).to_jserr()?;

    let output_own_ul = document().create_element_typed::<HtmlUListElement>().to_jserr()?;
    output_own_ul.set_class_name("checkbox_list");
    output_own.append_child(&output_own_ul).to_jserr()?;

    let mut stored_err = None;

    {
        let capture_err_tag = |val: bool, name: &str| {
            if stored_err.is_some() {
                return;
            }

            let cb_id = format!("game_edit_tag__{}", name);
            if let Err(e) = create_checkbox(val, cb_id.as_str(), name, &output_tag_ul, true) {
                stored_err = Some(e);
            }
        };

        custom_info.tags.each(capture_err_tag);
        if let Some(e) = stored_err {
            return Err(e);
        }
    }

    {
        let capture_err_own = |owned: bool, name: &str| {
            if stored_err.is_some() {
                return;
            }

            let cb_id = format!("game_edit_own__{}", name);
            if let Err(e) = create_checkbox(owned, cb_id.as_str(), name, &output_own_ul, true) {
                stored_err = Some(e);
            }
        };

        custom_info.own.each(capture_err_own);
        if let Some(e) = stored_err {
            return Err(e);
        }
    }

    Ok(())
}

fn edit_screen_populate_choose_state(choose_state: &core::SGameChooseState) -> Result<(), JsError> {
    populate_date_input("game_edit_next_valid_proposal_date", Some(choose_state.next_valid_proposal_date))?;
    populate_checkox_input("game_edit_retired", choose_state.retired)?;
    populate_number_input("game_edit_passes", choose_state.passes as f64)?;
    populate_checkox_input("game_edit_ignore_passes", choose_state.ignore_passes)?;

    Ok(())
}

fn edit_screen_update_text() -> Result<(), JsError> {

    let app = APP.try_read().expect("Should never actually have contention");

    let header_elem = document().get_typed_element_by_id::<HtmlElement>("game_edit_header").to_jserr()?;
    header_elem.set_inner_text(app.game_edit.header());
    let submit_elem = document().get_typed_element_by_id::<HtmlElement>("game_edit_submit").to_jserr()?;
    submit_elem.set_inner_text(app.game_edit.submit_button_text());

    Ok(())
}
fn edit_game(game: core::SCollectionGame) -> Result<(), JsError> {
    edit_screen_populate_game_info(&game.game_info)?;
    edit_screen_populate_custom_info(&game.custom_info)?;
    edit_screen_populate_choose_state(&game.choose_state)?;

    document().get_typed_element_by_id::<HtmlDivElement>("game_edit_choose_state").to_jserr()?
        .style().set_property("display", "block").to_jserr()?;

    {
        let mut app = APP.try_write().expect("Should never actually have contention.");
        app.game_edit = EGameEdit::Edit(game.internal_id);
    }

    edit_screen_update_text()?;

    swap_section_div("game_edit_div")?;

    Ok(())
}

fn add_game(game: core::SAddCollectionGame) -> Result<(), JsError> {
    edit_screen_populate_game_info(&game.game_info)?;
    edit_screen_populate_custom_info(&game.custom_info)?;

    document().get_typed_element_by_id::<HtmlDivElement>("game_edit_choose_state").to_jserr()?
        .style().set_property("display", "none").to_jserr()?;

    {
        let mut app = APP.try_write().expect("Should never actually have contention.");
        app.game_edit= EGameEdit::Add(game);
    }

    edit_screen_update_text()?;

    swap_section_div("game_edit_div")?;

    Ok(())
}

#[wasm_bindgen]
pub fn add_screen_add_result(igdb_id: u32) -> Result<(), JsError> {
    let game_opt = {
        let mut result = None;

        let app = APP.try_read().expect("Should never actually have contention");
        if let Some(games) = &app.last_search_igdb_results {
            for g in games {
                if let Some(inner_id) = g.game_info.igdb_id() {
                    if inner_id == igdb_id {
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
    add_game(core::SAddCollectionGame::new(game.game_info))?;

    Ok(())
}

#[wasm_bindgen]
pub fn add_screen_add_custom() -> Result<(), JsError> {
    let game_info = core::EGameInfo::new_custom(String::new(), core::EReleaseDate::UnknownUnreleased);

    add_game(core::SAddCollectionGame::new(game_info))?;

    Ok(())
}

fn update_game_info_from_edit_screen(game_info: &mut core::EGameInfo) -> Result<(), JsError> {
    game_info.set_title(document().get_typed_element_by_id::<HtmlInputElement>("game_edit_title").to_jserr()?.value().as_str());

    let date_str = document().get_typed_element_by_id::<HtmlInputElement>("game_edit_release_date_date").to_jserr()?.value();
    let type_value = document().get_typed_element_by_id::<HtmlSelectElement>("game_edit_release_date_type").to_jserr()?.value();

    match type_value.as_str() {
        "unknown_unreleased" => game_info.set_release_date(core::EReleaseDate::UnknownUnreleased),
        "unknown_released" => game_info.set_release_date(core::EReleaseDate::UnknownReleased),
        "known" => {
            if let Err(_) = game_info.set_release_date_known_str(date_str.as_str()) {
                return Err(JsError::new("Could not parse date from game_edit_release_date element."));
            }
        },
        _ => {
            return Err(JsError::new("Release date type select widget had invalid value"));
        }
    }

    Ok(())
}

fn checkbox_value(id: &str) -> Result<bool, JsError> {
    Ok(document().get_typed_element_by_id::<HtmlInputElement>(id).to_jserr()?.checked())
}

fn update_bool_from_checkbox(val: &mut bool, name: &str, id_prefix: &str) -> Result<(), JsError> {
    let id = format!("{}_{}", id_prefix, name);
    *val = checkbox_value(id.as_str())?;
    Ok(())
}

fn update_custom_info_from_edit_screen(custom_info: &mut core::SGameCustomInfo) -> Result<(), JsError> {
    custom_info.via = document().get_typed_element_by_id::<HtmlInputElement>("game_edit_via").to_jserr()?.value();

    let mut stored_err = None;
    {
        let capture_err = |tag: &mut bool, name: &str| {
            if stored_err.is_some() {
                return;
            }

            if let Err(e) = update_bool_from_checkbox(tag, name, "game_edit_tag_") {
                stored_err = Some(e);
            }
        };

        custom_info.tags.each_mut(capture_err);

        if let Some(e) = stored_err {
            return Err(e);
        }
    }

    {
        let capture_err = |owned: &mut bool, name: &str| {
            if stored_err.is_some() {
                return;
            }

            if let Err(e) = update_bool_from_checkbox(owned, name, "game_edit_own_") {
                stored_err = Some(e);
            }
        };

        custom_info.own.each_mut(capture_err);

        if let Some(e) = stored_err {
            return Err(e);
        }
    }

    Ok(())
}

fn update_choose_state_from_edit_screen(choose_state: &mut core::SGameChooseState) -> Result<(), JsError> {
    let choose_date_str = document().get_typed_element_by_id::<HtmlInputElement>("game_edit_next_valid_proposal_date").to_jserr()?.value();
    choose_state.next_valid_proposal_date = chrono::naive::NaiveDate::parse_from_str(choose_date_str.as_str(), "%Y-%m-%d")?;
    choose_state.retired = checkbox_value("game_edit_retired")?;
    choose_state.passes = document().get_typed_element_by_id::<HtmlInputElement>("game_edit_passes").to_jserr()?.value_as_number() as u16;
    choose_state.ignore_passes = checkbox_value("game_edit_ignore_passes")?;

    Ok(())
}

async fn edit_screen_submit_edit_helper(game: &mut core::SCollectionGame) -> Result<(), JsError> {
    update_game_info_from_edit_screen(&mut game.game_info)?;
    update_custom_info_from_edit_screen(&mut game.custom_info)?;
    update_choose_state_from_edit_screen(&mut game.choose_state)?;

    let _sl = SShowLoadingHelper::new();
    if let Err(e) = server_api::edit_game(game.clone()).await {
        show_error(e)?;
    }
    Ok(())
}

async fn edit_screen_submit_add_helper(mut game: core::SAddCollectionGame) -> Result<(), JsError> {
    update_game_info_from_edit_screen(&mut game.game_info)?;
    update_custom_info_from_edit_screen(&mut game.custom_info)?;

    let _sl = SShowLoadingHelper::new();
    if let Err(e) = server_api::add_game(game.clone()).await {
        show_error(e)?;
    }
    Ok(())
}

#[wasm_bindgen]
pub async fn release_date_type_changed(caller: Element) -> Result<(), JsError> {

    weblog!("Caller was {:?} (id \"{}\")", caller, caller.id());

    let date_id = format!("{}date", &caller.id()[0.. (caller.id().len() - 4)]); // slice off "type"

    let date_type_elem = document().get_typed_element_by_id::<HtmlSelectElement>(caller.id().as_str()).to_jserr()?;
    let date_date_elem = document().get_typed_element_by_id::<HtmlInputElement>(date_id.as_str()).to_jserr()?;

    match date_type_elem.value().as_str() {
        "unknown_unreleased" => {
            date_date_elem.style().set_property("display", "none").to_jserr()?;
        },
        "unknown_released" => {
            date_date_elem.style().set_property("display", "none").to_jserr()?;
        },
        "known" => {
            date_date_elem.style().set_property("display", "block").to_jserr()?;
        },
        _ => {},
    };

    Ok(())
}

#[wasm_bindgen]
pub async fn edit_screen_submit() -> Result<(), JsError> {
    let p = document().get_typed_element_by_id::<HtmlParagraphElement>("result_message").to_jserr()?;

    let edit = {
        let mut app = APP.try_write().expect("Should never actually have contention");
        std::mem::take(&mut app.game_edit)
    };

    match edit {
        EGameEdit::None => {
            p.set_inner_text("ERROR: edit screen had no valid game");
        },
        EGameEdit::Add(add_game) => {
            match edit_screen_submit_add_helper(add_game).await {
                Ok(_) => p.set_inner_text("Successfully added game"),
                Err(e) => {
                    p.set_inner_text("Failed to add game.");
                    return Err(e);
                },
            }
        },
        EGameEdit::Edit(internal_id) => {
            let mut app = APP.try_write().expect("Should never actually have contention");
            let game = cached_collection_game_by_id_mut(&mut app.collection_game_cache, internal_id).ok_or(JsError::new("Submitting edits to game that isn't in cache"))?;

            match edit_screen_submit_edit_helper(game).await {
                Ok(_) => p.set_inner_text("Successfully edited game"),
                Err(e) => {
                    p.set_inner_text("Failed to edit game.");
                    return Err(e);
                },
            }
        },
    }

    swap_section_div("result_div")?;
    Ok(())
}

fn populate_sessions_screen_list(sessions: Vec<core::SSessionAndCollectionGame>) -> Result<(), JsError> {
    let document = document();

    let output_elem = document.get_typed_element_by_id::<HtmlDivElement>("session_screen_session_list").to_jserr()?;
    output_elem.set_inner_html("");

    for session in &sessions {
        let session_div = document.create_element_typed::<HtmlDivElement>().to_jserr()?;
        output_elem.append_child(&session_div).to_jserr()?;

        // create the game card
        let mut game_card = SGameCard::new_from_collection_game(&session.collection_game)?;
        game_card.show_release_date().regen()?;

        session_div.append_child(&game_card.main_div).to_jserr()?;

        let card_custom = game_card.customizable_div()?;

        let header_elem = document.create_element("h4").to_jserr()?;
        header_elem.set_text_content(Some("Session"));
        card_custom.append_child(&header_elem).to_jserr()?;

        // populate session_div with session-specific info
        let start_date_elem = document.create_element_typed::<HtmlParagraphElement>().to_jserr()?;
        start_date_elem.set_inner_text(format!("Start date: {}", session.session.start_date).as_str());
        card_custom.append_child(&start_date_elem).to_jserr()?;

        match session.session.state {
            core::ESessionState::Ongoing => {
                let checkbox_list = document.create_element_typed::<HtmlUListElement>().to_jserr()?;
                checkbox_list.set_class_name("checkbox_list");
                card_custom.append_child(&checkbox_list).to_jserr()?;

                let memorable_elem_id = format!("session_screen_memorable_{}", session.session.internal_id);
                create_checkbox(false, memorable_elem_id.as_str(), "Memorable", &checkbox_list, true)?;

                let retire_elem_id = format!("session_screen_retire_{}", session.session.internal_id);
                create_checkbox(false, retire_elem_id.as_str(), "Retire", &checkbox_list, true)?;

                let ignore_passes_id = format!("session_screen_ignore_passes_{}", session.session.internal_id);
                create_checkbox(false, ignore_passes_id.as_str(), "Allow infinite passes", &checkbox_list, true)?;

                let button_elem = document.create_element_typed::<HtmlButtonElement>().to_jserr()?;
                let onclick_body = format!("session_screen_finish_session({});", session.session.internal_id);
                let onclick = Function::new_no_args(onclick_body.as_str());
                button_elem.set_onclick(Some(&onclick));
                button_elem.set_inner_text("Finish session");
                card_custom.append_child(&button_elem).to_jserr()?;
            },
            core::ESessionState::Finished{end_date, memorable} => {
                let end_date_elem = document.create_element_typed::<HtmlParagraphElement>().to_jserr()?;
                end_date_elem.set_inner_text(format!("End date: {}", end_date).as_str());
                card_custom.append_child(&end_date_elem).to_jserr()?;

                let memorable_elem = document.create_element_typed::<HtmlParagraphElement>().to_jserr()?;
                memorable_elem.set_inner_text(format!("Memorable: {}", memorable).as_str());
                card_custom.append_child(&memorable_elem).to_jserr()?;
            }
        }

    }

    // -- cache results for later use
    {
        let mut app = APP.try_write().expect("Should never actually have contention.");
        app.session_screen_sessions.clear();
        for session_and_game in sessions {
            app.session_screen_sessions.push(session_and_game.session);
            let internal_id = session_and_game.collection_game.internal_id;
            app.collection_game_cache.insert(internal_id, session_and_game.collection_game);
        }
    }

    Ok(())
}

fn create_checkbox(initial_val: bool, elem_id: &str, label_text: &str, output_elem: &HtmlElement, in_li: bool) -> Result<(), JsError> {
    let checkbox = document().create_element_typed::<HtmlInputElement>().to_jserr()?;
    checkbox.set_type("checkbox");
    checkbox.set_default_checked(initial_val);
    checkbox.set_id(elem_id);

    let label = document().create_element_typed::<HtmlLabelElement>().to_jserr()?;
    label.set_html_for(elem_id);
    label.set_inner_text(label_text);

    if in_li {
        let li = document().create_element_typed::<HtmlLiElement>().to_jserr()?;
        output_elem.append_child(&li).to_jserr()?;
        li.append_child(&checkbox).to_jserr()?;
        li.append_child(&label).to_jserr()?;
    }
    else {
        output_elem.append_child(&checkbox).to_jserr()?;
        output_elem.append_child(&label).to_jserr()?;
    }

    Ok(())
}

fn populate_collection_screen_game_list(games: Vec<core::SCollectionGame>) -> Result<(), JsError> {
    let doc = document();

    let output_elem = doc.get_typed_element_by_id::<HtmlDivElement>("collection_screen_game_list").to_jserr()?;
    output_elem.set_inner_html("");

    for game in &games {
        let collection_item_div = doc.create_element_typed::<HtmlDivElement>().to_jserr()?;
        output_elem.append_child(&collection_item_div).to_jserr()?;

        let mut game_card = SGameCard::new_from_collection_game(&game)?;

        game_card
            .show_release_date()
            .show_igdb_link()
            .show_own_info()
            .show_tag_info()
            .regen()?;

        collection_item_div.append_child(&game_card.main_div).to_jserr()?;

        let customizable_div = game_card.customizable_div()?;

        let start_sesion_button_elem = doc.create_element_typed::<HtmlButtonElement>().to_jserr()?;
        let onclick_body = format!("start_session({});", game.internal_id);
        let onclick = Function::new_no_args(onclick_body.as_str());
        start_sesion_button_elem.set_onclick(Some(&onclick));
        start_sesion_button_elem.set_inner_text("Start session");
        customizable_div.append_child(&start_sesion_button_elem).to_jserr()?;
    }

    // -- cache results for later use
    {
        let mut app = APP.try_write().expect("Should never actually have contention.");
        for game in games {
            app.collection_game_cache.insert(game.internal_id, game);
        }
    }

    Ok(())
}

fn populate_full_collection_screen_game_list(games: Vec<core::SCollectionGame>) -> Result<(), JsError> {
    let doc = document();

    let output_elem = doc.get_typed_element_by_id::<HtmlDivElement>("full_collection_screen_game_list").to_jserr()?;
    output_elem.set_inner_html("");

    let live_only = checkbox_value("live_games_only")?;

    for game in &games {

        if live_only && !game.choose_state.alive() {
            continue;
        }

        let game_card = SCompactGameCard::new_from_collection_game(&game)?;
        output_elem.append_child(&game_card.main_div).to_jserr()?;
    }

    // -- cache results for later use
    {
        let mut app = APP.try_write().expect("Should never actually have contention.");
        for game in games {
            app.collection_game_cache.insert(game.internal_id, game);
        }
    }

    Ok(())
}

fn show_result(msg: &str) -> Result<(), JsError> {
    let p = document().get_typed_element_by_id::<HtmlParagraphElement>("result_message").to_jserr()?;
    p.set_inner_text(msg);
    swap_section_div("result_div")?;
    Ok(())
}

fn show_error(e: String) -> Result<(), JsError> {
    element("popup_overlay")?.style().set_property("display", "block").to_jserr()?;
    element("error_message")?.style().set_property("display", "block").to_jserr()?;
    element("error_message_content")?.set_inner_text(e.as_str());
    Ok(())
}

async fn enter_sessions_screen() -> Result<(), JsError> {
    session_screen_apply_filter().await?;
    swap_section_div("sessions_div")?;

    Ok(())
}

#[wasm_bindgen]
pub async fn session_screen_apply_filter() -> Result<(), JsError> {
    let year = if checkbox_value("sessions_screen_filter_year_enable")? {
        let year_input = document().get_typed_element_by_id::<HtmlInputElement>("sessions_screen_filter_year").to_jserr()?;
        Some(year_input.value_as_number() as u32)
    }
    else {
        None
    };

    let sl = SShowLoadingHelper::new();
    let sessions = match server_api::get_sessions(
        None,
        checkbox_value("sessions_screen_filter_active")?,
        checkbox_value("sessions_screen_filter_memorable")?,
        year,
    ).await {
        Ok(s) => s,
        Err(e) => {
            show_error(e)?;
            return Ok(());
        }
    };

    drop(sl);
    populate_sessions_screen_list(sessions)?;

    Ok(())
}

async fn enter_collection_screen() -> Result<(), JsError> {
    let sl = SShowLoadingHelper::new();
    let games = match server_api::get_recent_collection_games().await {
        Ok(g) => g,
        Err(e) => {
            show_error(e)?;
            return Ok(());
        }
    };
    drop(sl);

    populate_collection_screen_game_list(games)?;

    swap_section_div("collection_div")?;

    Ok(())
}

#[wasm_bindgen]
pub async fn collection_screen_search() -> Result<(), JsError> {
    let document = document();

    // -- do the request
    let collection_search_input = &document.get_typed_element_by_id::<HtmlInputElement>("collection_search_input").to_jserr()?;
    let sl = SShowLoadingHelper::new();
    let games : Vec<core::SCollectionGame> = match server_api::search_collection(collection_search_input.value().as_str()).await {
        Ok(g) => g,
        Err(e) => {
            show_error(e)?;
            return Ok(());
        }
    };
    drop(sl);

    populate_collection_screen_game_list(games)?;

    Ok(())
}

#[wasm_bindgen]
pub async fn update_igdb_games() -> Result<(), JsError> {
    let sl = SShowLoadingHelper::new();
    match server_api::update_igdb_games().await {
        Ok(g) => g,
        Err(e) => {
            show_error(e)?;
            return Ok(());
        }
    };
    drop(sl);

    Ok(())
}

async fn enter_full_collection_screen() -> Result<(), JsError> {
    let sl = SShowLoadingHelper::new();
    let games = match server_api::get_full_collection().await {
        Ok(g) => g,
        Err(e) => {
            show_error(e)?;
            return Ok(());
        }
    };
    drop(sl);

    populate_full_collection_screen_game_list(games)?;

    swap_section_div("full_collection_div")?;

    Ok(())
}

fn cached_collection_game_by_id(app: &SAppState, internal_id: u32) -> Option<core::SCollectionGame> {
    let result = app.collection_game_cache.get(&internal_id).cloned();

    if let None = result {
        weblog!("Something requested a game not in the cache, dumping cache");
        for (k, v) in &app.collection_game_cache {
            weblog!("ID {}: \"{}\"", k, v.game_info.title());
        }
    }

    result
}

fn cached_collection_game_by_id_mut(cache: &mut HashMap<u32, core::SCollectionGame>, internal_id: u32) -> Option<&mut core::SCollectionGame> {
    if !cache.contains_key(&internal_id) {
        weblog!("Something requested a game not in the cache, dumping cache");
        for (k, v) in cache {
            weblog!("ID {}: \"{}\"", k, v.game_info.title());
        }

        return None;
    }

    cache.get_mut(&internal_id)
}

#[wasm_bindgen]
pub async fn edit_cached_game(internal_id: u32) -> Result<(), JsError> {
    let game = {
        let app = APP.try_read().expect("Should never actually have contention");
        cached_collection_game_by_id(&app, internal_id)
            .ok_or(JsError::new("Somehow editing a game that was not cached from the server."))?
    };

    edit_game(game)?;
    Ok(())
}

#[wasm_bindgen]
pub async fn game_card_view_details(internal_id: u32) -> Result<(), JsError> {
    let game = {
        let app = APP.try_read().map_err(|_| JsError::new("Should never actually have contention"))?;
        cached_collection_game_by_id(&app, internal_id)
            .ok_or(JsError::new(format!("Somehow viewing a game (internal id {}) that was not cached from the server.", internal_id).as_str()))?
    };

    view_details(game).await
}

#[wasm_bindgen]
pub async fn start_session(internal_id: u32) -> Result<(), JsError> {
    let app = APP.try_read().expect("Should never actually have contention");
    let game = cached_collection_game_by_id(&app, internal_id)
        .ok_or(JsError::new("Somehow starting session for a game that was not cached from the server."))?;

    let p = document().get_typed_element_by_id::<HtmlParagraphElement>("result_message").to_jserr()?;

    let sl = SShowLoadingHelper::new();
    match server_api::start_session(game.internal_id).await {
        Ok(_) => p.set_inner_text("Successfully started session."),
        Err(e) => p.set_inner_text(e.as_str()),
    }
    drop(sl);

    swap_section_div("result_div")?;

    Ok(())
}

#[wasm_bindgen]
pub async fn session_screen_finish_session(internal_id: u32) -> Result<(), JsError> {
    let session_opt = {
        let mut result = None;

        let app = APP.try_read().expect("Should never actually have contention");

        for s in &app.session_screen_sessions {
            if s.internal_id == internal_id {
                result = Some(s.clone());
                break;
            }
            else {
                weblog!("Trying to finish session for internal_id {} but it's not in the list!", internal_id);
            }
        }

        result
    };
    session_opt.ok_or(JsError::new("Somehow finishing session not in the list."))?;

    let memorable_checkbox_id = format!("session_screen_memorable_{}", internal_id);
    let memorable = checkbox_value(memorable_checkbox_id.as_str())?;

    let retire_checkbox_id = format!("session_screen_retire_{}", internal_id);
    let retire = checkbox_value(retire_checkbox_id.as_str())?;

    let ignore_passes_id = format!("session_screen_ignore_passes_{}", internal_id);
    let set_ignore_passes = checkbox_value(ignore_passes_id.as_str())?;

    let p = document().get_typed_element_by_id::<HtmlParagraphElement>("result_message").to_jserr()?;
    let sl = SShowLoadingHelper::new();
    match server_api::finish_session(internal_id, memorable, retire, set_ignore_passes).await {
        Ok(_) => p.set_inner_text("Successfully finished session."),
        Err(_) => p.set_inner_text("Failed to finish session."),
    }
    drop(sl);

    swap_section_div("result_div")?;

    Ok(())
}

async fn commit_randomizer_choose_states(session: &SGameRandomizerSession, app: &SAppState) -> Result<(), JsError> {
    let sl = SShowLoadingHelper::new();

    // -- build list of collection games to send to server
    let mut collection_games = Vec::with_capacity(session.shuffled_internal_ids.len());
    for internal_id in session.shuffled_internal_ids.iter().copied() {
        let game = cached_collection_game_by_id(app, internal_id).ok_or(JsError::new("randomizer game not in cache"))?;
        collection_games.push(game);
    }

    if let Err(e) = server_api::update_choose_state(&collection_games).await {
        show_error(e)?;
        return Ok(());
    }
    drop(sl);

    Ok(())
}

async fn populate_randomizer_choose_screen() -> Result<(), JsError> {
    let mut app = APP.try_write().expect("Should never actually have contention.");
    let mut done = false;

    if let EGameRandomizer::Choosing(session) = &app.game_randomizer {
        if session.cur_idx >= session.shuffled_internal_ids.len() {
            // -- out of games
            commit_randomizer_choose_states(&session, &app).await?;

            done = true;
            show_result("End of randomizer candidates! You having nothing to play!")?;
        }
        else {
            let internal_id = session.shuffled_internal_ids[session.cur_idx];
            let game = cached_collection_game_by_id(&app, internal_id).ok_or(JsError::new("randomizer game not in cache"))?;

            let card_div = div("randomizer_game_card")?;
            card_div.set_inner_text("");

            let mut game_card = SGameCard::new_from_collection_game(&game)?;
            game_card
                .show_igdb_link()
                .show_via()
                .show_own_info()
                .regen()?;

            card_div.append_child(&game_card.main_div).to_jserr()?;

            let customizable_div = game_card.customizable_div()?;

            let doc = document();

            {
                let button_elem = doc.create_element_typed::<HtmlButtonElement>().to_jserr()?;
                let onclick = Function::new_no_args("randomizer_pick_current_game()");
                button_elem.set_onclick(Some(&onclick));
                button_elem.set_inner_text("Start this game!");
                customizable_div.append_child(&button_elem).to_jserr()?;
            }

            {
                let button_elem = doc.create_element_typed::<HtmlButtonElement>().to_jserr()?;
                let onclick = Function::new_no_args("randomizer_pass_current_game()");
                button_elem.set_onclick(Some(&onclick));
                button_elem.set_inner_text("Pass this game");
                customizable_div.append_child(&button_elem).to_jserr()?;
            }

            {
                let button_elem = doc.create_element_typed::<HtmlButtonElement>().to_jserr()?;
                let onclick = Function::new_no_args("randomizer_push_current_game()");
                button_elem.set_onclick(Some(&onclick));
                button_elem.set_inner_text("Push to later");
                customizable_div.append_child(&button_elem).to_jserr()?;
            }

            {
                let button_elem = doc.create_element_typed::<HtmlButtonElement>().to_jserr()?;
                let onclick = Function::new_no_args("randomizer_retire_current_game()");
                button_elem.set_onclick(Some(&onclick));
                button_elem.set_inner_text("Retire");
                customizable_div.append_child(&button_elem).to_jserr()?;
            }

            div("randomizer_game_div")?.style().set_property("display", "block").to_jserr()?;
        }
    }
    else {
        return Err(JsError::new("populate_randomizer_choose_screen was called without any data."));
    }

    if done {
        app.game_randomizer = EGameRandomizer::Uninit;
    }

    Ok(())
}

#[wasm_bindgen]
pub async fn randomizer_screen_start() -> Result<(), JsError> {
    let couch = if checkbox_value("randomizer_screen_couch")? {
        Some(true)
    }
    else {
        None
    };
    let portable = if checkbox_value("randomizer_screen_portable")? {
        Some(true)
    }
    else {
        None
    };

    let jp_practice = match document()
        .get_typed_element_by_id::<HtmlSelectElement>("randomizer_screen_jp_practice")
        .to_jserr()?
        .value()
        .as_str()
    {
        "any" => None,
        "require_true" => Some(true),
        "require_false" => Some(false),
        _ => {
            show_error(String::from("Invalid value from randomizer_screen_jp_practice select."))?;
            None
        },
    };

    let max_passes = 2;

    let filter = core::SRandomizerFilter {
        tags: core::SGameTagsFilter{
            couch_playable: couch,
            portable_playable: portable,
            japanese_practice: jp_practice,
        },
        allow_unowned: checkbox_value("randomizer_screen_allow_unowned")?,
        only_firsts: checkbox_value("randomizer_screen_only_firsts")?,
        max_passes,
    };

    let sl = SShowLoadingHelper::new();
    let randomizer_list = match server_api::get_randomizer_games(filter).await {
        Ok(l) => l,
        Err(e) => {
            show_error(e)?;
            return Ok(());
        }
    };
    drop(sl);

    weblog!("Valid game count: {:?}", randomizer_list.games.len());

    {
        let mut app = APP.try_write().expect("Should never actually have contention.");

        let mut internal_id_list = Vec::with_capacity(randomizer_list.games.len());
        for game in randomizer_list.games {
            let internal_id = game.internal_id;
            app.collection_game_cache.insert(internal_id, game);
            internal_id_list.push(internal_id);
        }

        app.game_randomizer = EGameRandomizer::Choosing(SGameRandomizerSession{
            shuffled_internal_ids: internal_id_list,
            cur_idx: 0,
        });
    }

    populate_randomizer_choose_screen().await?;

    Ok(())
}

#[wasm_bindgen]
pub async fn randomizer_pick_current_game() -> Result<(), JsError> {
    let randomizer = {
        let mut app = APP.try_write().expect("Should never actually have contention.");
        std::mem::take(&mut app.game_randomizer)
    };

    if let EGameRandomizer::Choosing(session) = randomizer {
        // -- start the session
        {
            let game_internal_id = session.shuffled_internal_ids[session.cur_idx];
            let sl = SShowLoadingHelper::new();
            if let Err(e) = server_api::start_session(game_internal_id).await {
                show_error(e)?;
                return Ok(());
            }
            drop(sl);
        }

        // -- update all the choose date on games
        let app = APP.try_read().expect("Should never actually have contention.");
        commit_randomizer_choose_states(&session, &app).await?;
    }
    else {
        return Err(JsError::new("populate_randomizer_choose_screen was called without any data."));
    }

    show_sessions().await?;

    Ok(())
}

#[wasm_bindgen]
pub async fn randomizer_pass_current_game() -> Result<(), JsError> {
    let mut app = APP.try_write().expect("Should never actually have contention.");

    // -- wraps the use of &mut app so the borrow checker knows we're looking at separate fields
    fn mut_wrapper(app: &mut SAppState) -> Result<(), JsError> {
        if let EGameRandomizer::Choosing(session) = &mut app.game_randomizer {
            let internal_id = session.shuffled_internal_ids[session.cur_idx];
            let game = cached_collection_game_by_id_mut(&mut app.collection_game_cache, internal_id).expect("Randomizer game not in cache.");
            game.choose_state.pass();

            session.cur_idx = session.cur_idx + 1;
        }
        else {
            show_error(String::from("randomizer_pass_current_game was called without any data."))?;
        }

        Ok(())
    }

    mut_wrapper(&mut app)?;

    drop(app);

    populate_randomizer_choose_screen().await?;

    Ok(())
}

#[wasm_bindgen]
pub async fn randomizer_push_current_game() -> Result<(), JsError> {
    let mut app = APP.try_write().expect("Should never actually have contention.");

    // -- wraps the use of &mut app so the borrow checker knows we're looking at separate fields
    fn mut_wrapper(app: &mut SAppState) -> Result<(), JsError> {
        if let EGameRandomizer::Choosing(session) = &mut app.game_randomizer {
            let internal_id = session.shuffled_internal_ids[session.cur_idx];
            let game = cached_collection_game_by_id_mut(&mut app.collection_game_cache, internal_id).expect("Randomizer game not in cache.");
            game.choose_state.push(45);

            session.cur_idx = session.cur_idx + 1;
        }
        else {
            show_error(String::from("randomizer_push_current_game was called without any data."))?;
        }

        Ok(())
    }

    mut_wrapper(&mut app)?;

    drop(app);

    populate_randomizer_choose_screen().await?;

    Ok(())
}

#[wasm_bindgen]
pub async fn randomizer_retire_current_game() -> Result<(), JsError> {
    let mut app = APP.try_write().expect("Should never actually have contention.");

    // -- wraps the use of &mut app so the borrow checker knows we're looking at separate fields
    fn mut_wrapper(app: &mut SAppState) -> Result<(), JsError> {
        if let EGameRandomizer::Choosing(session) = &mut app.game_randomizer {
            let internal_id = session.shuffled_internal_ids[session.cur_idx];
            let game = cached_collection_game_by_id_mut(&mut app.collection_game_cache, internal_id).expect("Randomizer game not in cache.");
            game.choose_state.retire();

            session.cur_idx = session.cur_idx + 1;
        }
        else {
            show_error(String::from("randomizer_retire_current_game was called without any data."))?;
        }

        Ok(())
    }

    mut_wrapper(&mut app)?;

    drop(app);

    populate_randomizer_choose_screen().await?;

    Ok(())
}

#[wasm_bindgen]
pub async fn game_details_edit() -> Result<(), JsError> {
    let internal_id = {
        let mut app = APP.try_write().expect("Should never actually have contention.");
        if app.details_screen_game.is_none() {
            show_error(String::from("No game on details screen to edit."))?;
            return Ok(());
        }
        std::mem::take(&mut app.details_screen_game)
    };

    edit_cached_game(internal_id.expect("taken above")).await?;

    Ok(())
}

#[wasm_bindgen]
pub async fn game_details_reset() -> Result<(), JsError> {
    let internal_id = {
        let mut app = APP.try_write().expect("Should never actually have contention.");
        if app.details_screen_game.is_none() {
            show_error(String::from("No game on details screen to edit."))?;
            return Ok(());
        }
        std::mem::take(&mut app.details_screen_game)
    };

    let app = APP.try_read().expect("Should never actually have contention.");
    let cached_game = cached_collection_game_by_id(&app, internal_id.expect("checked above"));

    let sl = SShowLoadingHelper::new();
    match server_api::reset_choose_state(&cached_game.expect("game was not cached")).await {
        Ok(_) => show_result("Successfully reset game.")?,
        Err(e) => {
            let msg = format!("Failed to reset message, got error: {}", e);
            show_result(msg.as_str())?
        }
    }
    drop(sl);

    Ok(())
}
