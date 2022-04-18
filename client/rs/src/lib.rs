mod web;
mod server_api;

use std::sync::RwLock;

//use console_error_panic_hook;
use chrono;
use once_cell::sync::Lazy;
use js_sys::{Function};
use wasm_bindgen::prelude::*;
use web_sys::{
    HtmlAnchorElement,
    HtmlButtonElement,
    HtmlDivElement,
    HtmlElement,
    HtmlImageElement,
    HtmlInputElement,
    HtmlLabelElement,
    HtmlLiElement,
    HtmlParagraphElement,
    HtmlSpanElement,
    HtmlUListElement,
};

use gamechooser_core as core;
use web::{document, TToJsError, TErgonomicDocument};

macro_rules! weblog {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

enum EGameEdit {
    None,
    Add(core::SAddCollectionGame),
    Edit(core::SCollectionGame),
}

struct SGameRandomizerSession {
    randomizer_list: core::SRandomizerList,
    cur_idx: usize,
}

enum EGameRandomizer {
    Uninit,
    Choosing(SGameRandomizerSession),
}

struct SAppState {
    session_screen_sessions: Option<Vec<core::SSessionAndGameInfo>>,

    collection_screen_games: Option<Vec<core::SCollectionGame>>,

    last_search_igdb_results: Option<Vec<core::EGameInfo>>,

    details_screen_game: Option<core::SCollectionGame>,

    game_edit: EGameEdit,

    game_randomizer: EGameRandomizer,
}

#[allow(dead_code)]
enum ETagQuery {
    TrueOrFalse,
    True,
    False,
}

struct SGameInListDiv {
    pub main_div: HtmlDivElement,
    pub info_div: HtmlDivElement,
}

static APP: Lazy<RwLock<SAppState>> = Lazy::new(|| RwLock::new(SAppState::new()));

impl SAppState {
    pub fn new() -> Self {
        Self {
            session_screen_sessions: None,
            last_search_igdb_results: None,
            collection_screen_games: None,
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

impl SGameInListDiv {
    fn new(game_info: &core::EGameInfo, add_class: Option<&str>) -> Result<Self, JsError> {
        let document = document();

        let game_div = document.create_element_typed::<HtmlDivElement>().to_jserr()?;
        game_div.set_class_name("game_in_list");
        if let Some(ac) = add_class {
            game_div.set_class_name(format!("game_in_list {}", ac).as_str());
        }

        let title_elem = document.create_element("h3").to_jserr()?;
        title_elem.set_text_content(Some(game_info.title()));
        game_div.append_child(&title_elem).to_jserr()?;

        let columns_div = document.create_element_typed::<HtmlDivElement>().to_jserr()?;
        columns_div.set_class_name("game_in_list_columns");
        game_div.append_child(&columns_div).to_jserr()?;

        if let Some(url) = game_info.cover_url() {
            let img_elem = document.create_element_typed::<HtmlImageElement>().to_jserr()?;
            img_elem.set_src(url.as_str());
            columns_div.append_child(&img_elem).to_jserr()?;
        }

        let info_div = document.create_element_typed::<HtmlDivElement>().to_jserr()?;
        info_div.set_class_name("game_in_list_info");
        columns_div.append_child(&info_div).to_jserr()?;

        Ok(Self {
            main_div: game_div,
            info_div,
        })
    }
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
    if server_api::check_logged_in().await {
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
pub fn show_add() -> Result<(), JsError> {
    swap_section_div("add_div")
}

#[wasm_bindgen]
pub fn show_randomizer() -> Result<(), JsError> {
    div("randomizer_game_div")?.style().set_property("display", "none").to_jserr()?;
    swap_section_div("randomizer_div")
}

#[wasm_bindgen]
pub fn show_stats() -> Result<(), JsError> {
    swap_section_div("stats_div")
}

#[wasm_bindgen]
pub async fn login_screen_submit() -> Result<(), JsError> {
    let sec_input : HtmlInputElement = document().get_typed_element_by_id::<HtmlInputElement>("login_screen_secret").to_jserr()?;
    if server_api::login(sec_input.value().as_str()).await.to_jserr().is_ok() {
        show_sessions().await?;
    }

    Ok(())
}

#[wasm_bindgen]
pub async fn add_screen_search_igdb() -> Result<(), JsError> {
    let document = document();

    // -- do the request
    let name_search_input = &document.get_typed_element_by_id::<HtmlInputElement>("add_screen_name_search_input").to_jserr()?;
    let games : Vec<core::EGameInfo> = match server_api::search_igdb(
        name_search_input.value().as_str(),
        checkbox_value("add_screen_games_only")?,
    ).await {

        Ok(g) => g,
        Err(e) => {
            show_error(e)?;
            return Ok(());
        },
    };

    let output_elem = div("add_screen_search_igdb_output")?;
    output_elem.set_inner_html("");

    if games.len() == 0 {
        output_elem.set_inner_text("No results.");
        return Ok(())
    }

    for game in &games {
        let game_div = SGameInListDiv::new(game, None)?;
        output_elem.append_child(&game_div.main_div).to_jserr()?;

        if let Some(d) = game.release_date() {
            let release_date_p = document.create_element_typed::<HtmlParagraphElement>().to_jserr()?;
            release_date_p.set_inner_text(format!("Release date: {:?}", d).as_str());
            game_div.info_div.append_child(&release_date_p).to_jserr()?;
        }

        let button_elem = document.create_element_typed::<HtmlButtonElement>().to_jserr()?;
        let onclick_body = format!("add_screen_add_result({});", game.igdb_id().expect("IGDB results should have an igdb_id"));
        let onclick = Function::new_no_args(onclick_body.as_str());
        button_elem.set_onclick(Some(&onclick));
        button_elem.set_inner_text("Add");
        game_div.info_div.append_child(&button_elem).to_jserr()?;
    }

    // -- cache results for later use
    {
        let mut app = APP.try_write().expect("Should never actually have contention.");
        app.last_search_igdb_results = Some(games);
    }

    Ok(())
}

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

fn details_screen_populate_game_info(game_info: &core::EGameInfo) -> Result<(), JsError> {
    populate_inner_text("game_details_title", game_info.title())?;
    if let Some(d) = game_info.release_date() {
        let release_date_str = format!("Release date: {:?}", d);
        populate_inner_text("game_details_release_date", release_date_str.as_str())?;
    }
    let cover_url = game_info.cover_url();
    populate_img("game_details_cover_art", cover_url.as_ref().map(|u| u.as_str()))?;

    Ok(())
}

fn details_screen_populate_custom_info(custom_info: &core::SGameCustomInfo) -> Result<(), JsError> {
    if custom_info.via.len() > 0 {
        let via_str = format!("Via: {}", custom_info.via);
        populate_inner_text("game_details_via", via_str.as_str())?;
    }

    let tags_and_own = div("game_details_tags_and_own")?;
    tags_and_own.set_inner_html("");
    create_tag_checks(&custom_info.tags, &tags_and_own)?;
    create_own_checks(&custom_info.own, &tags_and_own)?;

    Ok(())
}

fn view_details(game: core::SCollectionGame) -> Result<(), JsError> {
    details_screen_populate_game_info(&game.game_info)?;
    details_screen_populate_custom_info(&game.custom_info)?;

    {
        let mut app = APP.try_write().expect("Should never actually have contention.");
        app.details_screen_game = Some(game);
    }

    swap_section_div("game_details_div")?;

    Ok(())
}

fn edit_screen_populate_game_info(game_info: &core::EGameInfo) -> Result<(), JsError> {
    populate_text_input("game_edit_title", game_info.title())?;
    populate_date_input("game_edit_release_date", game_info.release_date())?;
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

            if let Err(e) = create_checkbox(val, name, "game_edit_tag_", &output_tag_ul, true) {
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

            if let Err(e) = create_checkbox(owned, name, "game_edit_own_", &output_own_ul, true) {
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
        app.game_edit= EGameEdit::Edit(game);
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
                if let Some(inner_id) = g.igdb_id() {
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
    add_game(core::SAddCollectionGame::new(game))?;

    Ok(())
}

fn update_game_info_from_edit_screen(game_info: &mut core::EGameInfo) -> Result<(), JsError> {
    game_info.set_title(document().get_typed_element_by_id::<HtmlInputElement>("game_edit_title").to_jserr()?.value().as_str());

    let date_str = document().get_typed_element_by_id::<HtmlInputElement>("game_edit_release_date").to_jserr()?.value();
    if let Err(_) = game_info.set_release_date_str(date_str.as_str()) {
        return Err(JsError::new("Could not parse date from game_edit_release_date element."));
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

async fn edit_screen_submit_edit_helper(mut game: core::SCollectionGame) -> Result<(), JsError> {
    update_game_info_from_edit_screen(&mut game.game_info)?;
    update_custom_info_from_edit_screen(&mut game.custom_info)?;
    update_choose_state_from_edit_screen(&mut game.choose_state)?;

    if let Err(e) = server_api::edit_game(game.clone()).await {
        show_error(e)?;
    }
    Ok(())
}

async fn edit_screen_submit_add_helper(mut game: core::SAddCollectionGame) -> Result<(), JsError> {
    update_game_info_from_edit_screen(&mut game.game_info)?;
    update_custom_info_from_edit_screen(&mut game.custom_info)?;

    if let Err(e) = server_api::add_game(game.clone()).await {
        show_error(e)?;
    }
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
        EGameEdit::Edit(game) => {
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

fn populate_sessions_screen_list(sessions: Vec<core::SSessionAndGameInfo>) -> Result<(), JsError> {
    let document = document();

    let output_elem = document.get_typed_element_by_id::<HtmlDivElement>("session_screen_session_list").to_jserr()?;
    output_elem.set_inner_html("");

    for session in &sessions {
        let session_div = SGameInListDiv::new(&session.game_info, None)?;
        output_elem.append_child(&session_div.main_div).to_jserr()?;

        let start_date_elem = document.create_element_typed::<HtmlParagraphElement>().to_jserr()?;
        start_date_elem.set_inner_text(format!("Start date: {}", session.session.start_date).as_str());
        session_div.info_div.append_child(&start_date_elem).to_jserr()?;

        match session.session.state {
            core::ESessionState::Ongoing => {
                let checkbox_list = document.create_element_typed::<HtmlUListElement>().to_jserr()?;
                checkbox_list.set_class_name("checkbox_list");
                session_div.info_div.append_child(&checkbox_list).to_jserr()?;

                let memorable_li = document.create_element_typed::<HtmlLiElement>().to_jserr()?;
                checkbox_list.append_child(&memorable_li).to_jserr()?;

                let memorable_elem = document.create_element_typed::<HtmlInputElement>().to_jserr()?;
                memorable_elem.set_type("checkbox");
                let memorable_elem_id = format!("session_screen_memorable_{}", session.session.internal_id);
                memorable_elem.set_id(memorable_elem_id.as_str());
                memorable_li.append_child(&memorable_elem).to_jserr()?;

                let memorable_elem_label = document.create_element_typed::<HtmlLabelElement>().to_jserr()?;
                memorable_elem_label.set_html_for(memorable_elem_id.as_str());
                memorable_elem_label.set_inner_text("Memorable");
                memorable_li.append_child(&memorable_elem_label).to_jserr()?;

                let button_elem = document.create_element_typed::<HtmlButtonElement>().to_jserr()?;
                let onclick_body = format!("session_screen_finish_session({});", session.session.internal_id);
                let onclick = Function::new_no_args(onclick_body.as_str());
                button_elem.set_onclick(Some(&onclick));
                button_elem.set_inner_text("Finish session");
                session_div.info_div.append_child(&button_elem).to_jserr()?;
            },
            core::ESessionState::Finished{end_date, memorable} => {
                let end_date_elem = document.create_element_typed::<HtmlParagraphElement>().to_jserr()?;
                end_date_elem.set_inner_text(format!("End date: {}", end_date).as_str());
                session_div.info_div.append_child(&end_date_elem).to_jserr()?;

                let memorable_elem = document.create_element_typed::<HtmlParagraphElement>().to_jserr()?;
                memorable_elem.set_inner_text(format!("Memorable: {}", memorable).as_str());
                session_div.info_div.append_child(&memorable_elem).to_jserr()?;
            }
        }

    }

    // -- cache results for later use
    {
        let mut app = APP.try_write().expect("Should never actually have contention.");
        app.session_screen_sessions = Some(sessions);
    }

    Ok(())
}

fn create_checkbox(initial_val: bool, name: &str, id_prefix: &str, output_elem: &HtmlElement, in_li: bool) -> Result<(), JsError> {

    let checkbox = document().create_element_typed::<HtmlInputElement>().to_jserr()?;
    checkbox.set_type("checkbox");
    checkbox.set_default_checked(initial_val);
    let id = format!("{}_{}", id_prefix, name);
    checkbox.set_id(id.as_str());

    let label = document().create_element_typed::<HtmlLabelElement>().to_jserr()?;
    label.set_html_for(id.as_str());
    label.set_inner_text(name);

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

fn create_check_span(val: bool, name: &str, output_div: &HtmlDivElement) -> Result<(), JsError> {
    if val {
        let span = document().create_element_typed::<HtmlSpanElement>().to_jserr()?;
        span.set_inner_text(format!("✓ {}", name).as_str());
        span.set_class_name("game_check");
        output_div.append_child(&span).to_jserr()?;
    }

    Ok(())
}

fn create_tag_checks(tags: &core::SGameTags, output_div: &HtmlDivElement) -> Result<(), JsError> {
    let mut stored_err = None;

    let capture_err = |owned: bool, name: &str| {
        if stored_err.is_some() {
            return;
        }

        if let Err(e) = create_check_span(owned, name, &output_div) {
            stored_err = Some(e);
        }
    };

    tags.each(capture_err);

    // -- verify no js errors during the tag adding stage
    if let Some(e) = stored_err {
        return Err(e);
    }

    Ok(())
}

fn create_own_checks(own: &core::SOwn, output_div: &HtmlDivElement) -> Result<(), JsError> {
    let mut stored_err = None;

    let capture_err = |owned: bool, name: &str| {
        if stored_err.is_some() {
            return;
        }

        if let Err(e) = create_check_span(owned, name, &output_div) {
            stored_err = Some(e);
        }
    };

    own.each(capture_err);

    // -- verify no js errors during the tag adding stage
    if let Some(e) = stored_err {
        return Err(e);
    }

    Ok(())
}

fn populate_collection_screen_game_list(games: Vec<core::SCollectionGame>) -> Result<(), JsError> {
    let doc = document();

    let output_elem = doc.get_typed_element_by_id::<HtmlDivElement>("collection_screen_game_list").to_jserr()?;
    output_elem.set_inner_html("");

    for game in &games {
        let game_div = SGameInListDiv::new(&game.game_info, None)?;
        output_elem.append_child(&game_div.main_div).to_jserr()?;

        if let Some(d) = game.game_info.release_date() {
            let release_date_p = doc.create_element_typed::<HtmlParagraphElement>().to_jserr()?;
            release_date_p.set_inner_text(format!("Release date: {:?}", d).as_str());
            game_div.info_div.append_child(&release_date_p).to_jserr()?;
        }

        let checks_container = doc.create_element_typed::<HtmlDivElement>().to_jserr()?;
        game_div.info_div.append_child(&checks_container).to_jserr()?;

        create_tag_checks(&game.custom_info.tags, &checks_container)?;
        create_own_checks(&game.custom_info.own, &checks_container)?;

        let edit_button_elem = doc.create_element_typed::<HtmlButtonElement>().to_jserr()?;
        let onclick_body = format!("collection_screen_view_details({});", game.internal_id);
        let onclick = Function::new_no_args(onclick_body.as_str());
        edit_button_elem.set_onclick(Some(&onclick));
        edit_button_elem.set_inner_text("View details");
        game_div.info_div.append_child(&edit_button_elem).to_jserr()?;

        let start_sesion_button_elem = doc.create_element_typed::<HtmlButtonElement>().to_jserr()?;
        let onclick_body = format!("collection_screen_start_session({});", game.internal_id);
        let onclick = Function::new_no_args(onclick_body.as_str());
        start_sesion_button_elem.set_onclick(Some(&onclick));
        start_sesion_button_elem.set_inner_text("Start session");
        game_div.info_div.append_child(&start_sesion_button_elem).to_jserr()?;
    }

    // -- cache results for later use
    {
        let mut app = APP.try_write().expect("Should never actually have contention.");
        app.collection_screen_games = Some(games);
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
    let err_div = div("error_div")?;
    err_div.set_inner_text(e.as_str());
    err_div.style().set_property("display", "block").to_jserr()?;
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

    let sessions = match server_api::get_sessions(
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

    populate_sessions_screen_list(sessions)?;

    Ok(())
}

async fn enter_collection_screen() -> Result<(), JsError> {
    let games = match server_api::get_recent_collection_games().await {
        Ok(g) => g,
        Err(e) => {
            show_error(e)?;
            return Ok(());
        }
    };

    populate_collection_screen_game_list(games)?;

    swap_section_div("collection_div")?;

    Ok(())
}

#[wasm_bindgen]
pub async fn collection_screen_search() -> Result<(), JsError> {
    let document = document();

    // -- do the request
    let collection_search_input = &document.get_typed_element_by_id::<HtmlInputElement>("collection_search_input").to_jserr()?;
    let games : Vec<core::SCollectionGame> = match server_api::search_collection(collection_search_input.value().as_str()).await {
        Ok(g) => g,
        Err(e) => {
            show_error(e)?;
            return Ok(());
        }
    };

    populate_collection_screen_game_list(games)?;

    Ok(())
}

fn collection_screen_game_by_id(internal_id: u32) -> Option<core::SCollectionGame> {
    let mut result = None;

    let app = APP.try_read().expect("Should never actually have contention");
    if let Some(games) = &app.collection_screen_games {
        for g in games {
            if internal_id == g.internal_id {
                result = Some(g.clone());
                break;
            }
        }
    }

    result
}

#[wasm_bindgen]
pub async fn collection_screen_edit_game(internal_id: u32) -> Result<(), JsError> {
    let game = collection_screen_game_by_id(internal_id)
        .ok_or(JsError::new("Somehow editing a game that was not in collection screen."))?;

    edit_game(game)
}

#[wasm_bindgen]
pub async fn collection_screen_view_details(internal_id: u32) -> Result<(), JsError> {
    let game = collection_screen_game_by_id(internal_id)
        .ok_or(JsError::new("Somehow viewing a game that was not in collection screen."))?;

    view_details(game)
}

#[wasm_bindgen]
pub async fn collection_screen_start_session(internal_id: u32) -> Result<(), JsError> {
    let game = collection_screen_game_by_id(internal_id)
        .ok_or(JsError::new("Somehow starting session for a game that was not in collection screen."))?;

    let p = document().get_typed_element_by_id::<HtmlParagraphElement>("result_message").to_jserr()?;

    match server_api::start_session(game.internal_id).await {
        Ok(_) => p.set_inner_text("Successfully started session."),
        Err(e) => p.set_inner_text(e.as_str()),
    }

    swap_section_div("result_div")?;

    Ok(())
}

#[wasm_bindgen]
pub async fn session_screen_finish_session(internal_id: u32) -> Result<(), JsError> {
    let session_opt = {
        let mut result = None;

        let app = APP.try_read().expect("Should never actually have contention");
        if let Some(sessions) = &app.session_screen_sessions {
            for s in sessions {
                if s.session.internal_id == internal_id {
                    result = Some(s.clone());
                    break;
                }
                else {
                    weblog!("Trying to finish session for internal_id {} but it's not in the list!", internal_id);
                }
            }
        }

        result
    };
    session_opt.ok_or(JsError::new("Somehow finishing session not in the list."))?;

    let checkbox_id = format!("session_screen_memorable_{}", internal_id);
    let memorable = checkbox_value(checkbox_id.as_str())?;

    let p = document().get_typed_element_by_id::<HtmlParagraphElement>("result_message").to_jserr()?;
    match server_api::finish_session(internal_id, memorable).await {
        Ok(_) => p.set_inner_text("Successfully finished session."),
        Err(_) => p.set_inner_text("Failed to finish session."),
    }

    swap_section_div("result_div")?;

    Ok(())
}

async fn populate_randomizer_choose_screen() -> Result<(), JsError> {
    let mut app = APP.try_write().expect("Should never actually have contention.");
    let mut done = false;

    if let EGameRandomizer::Choosing(session) = &app.game_randomizer {
        if session.cur_idx >= session.randomizer_list.shuffled_indices.len() {
            // -- out of games
            if let Err(e) = server_api::update_choose_state(&session.randomizer_list.games).await {
                show_error(e)?;
                return Ok(());
            }

            done = true;
            show_result("End of randomizer candidates! You having nothing to play!")?;
        }
        else {
            let game_idx = session.randomizer_list.shuffled_indices[session.cur_idx];
            let game = &session.randomizer_list.games[game_idx];

            populate_inner_text("randomizer_game_title", game.game_info.title())?;
            let cover_url = game.game_info.cover_url();
            match cover_url {
                Some(u) => populate_img("randomizer_game_cover", Some(u.as_str()))?,
                None => populate_img("randomizer_game_cover", None)?,
            }

            if game.custom_info.via.len() > 0 {
                let via_text = format!("Via: {}", game.custom_info.via);
                populate_inner_text("randomizer_game_via", via_text.as_str())?;
                element("randomizer_game_via")?.style().set_property("display", "block").to_jserr()?;
            }
            else {
                element("randomizer_game_via")?.style().set_property("display", "none").to_jserr()?;
            }

            if let core::EGameInfo::IGDB(igdb) = &game.game_info {
                let a = document().get_typed_element_by_id::<HtmlAnchorElement>("randomizer_game_igdb_link").to_jserr()?;
                let page_url = format!("https://www.igdb.com/games/{}", igdb.slug);
                a.set_href(page_url.as_str());
                a.style().set_property("display", "block").to_jserr()?;
            }
            else {
                element("randomizer_game_igdb_link")?.style().set_property("display", "none").to_jserr()?;
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

    let max_passes = 2;

    let filter = core::SRandomizerFilter {
        tags: core::SGameTagsFilter{
            couch_playable: couch,
            portable_playable: portable,
        },
        allow_unowned: checkbox_value("randomizer_screen_allow_unowned")?,
        max_passes,
    };

    let randomizer_list = match server_api::get_randomizer_games(filter).await {
        Ok(l) => l,
        Err(e) => {
            show_error(e)?;
            return Ok(());
        }
    };

    weblog!("Valid game count: {:?}", randomizer_list.games.len());

    {
        let mut app = APP.try_write().expect("Should never actually have contention.");
        app.game_randomizer = EGameRandomizer::Choosing(SGameRandomizerSession{
            randomizer_list,
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
            let cur_game_idx = session.randomizer_list.shuffled_indices[session.cur_idx];
            let game_internal_id = session.randomizer_list.games[cur_game_idx].internal_id;
            if let Err(e) = server_api::start_session(game_internal_id).await {
                show_error(e)?;
                return Ok(());
            }
        }

        // -- update all the choose date on games
        {
            if let Err(e) = server_api::update_choose_state(&session.randomizer_list.games).await {
                show_error(e)?;
                return Ok(());
            }
        }
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

    if let EGameRandomizer::Choosing(session) = &mut app.game_randomizer {
        let cur_game_idx = session.randomizer_list.shuffled_indices[session.cur_idx];
        let game = &mut session.randomizer_list.games[cur_game_idx];
        game.choose_state.pass();

        session.cur_idx = session.cur_idx + 1;
    }
    else {
        show_error(String::from("randomizer_pass_current_game was called without any data."))?;
    }

    drop(app);

    populate_randomizer_choose_screen().await?;

    Ok(())
}

#[wasm_bindgen]
pub async fn randomizer_push_current_game() -> Result<(), JsError> {
    let mut app = APP.try_write().expect("Should never actually have contention.");

    if let EGameRandomizer::Choosing(session) = &mut app.game_randomizer {
        let cur_game_idx = session.randomizer_list.shuffled_indices[session.cur_idx];
        let game = &mut session.randomizer_list.games[cur_game_idx];
        game.choose_state.push();

        session.cur_idx = session.cur_idx + 1;
    }
    else {
        show_error(String::from("randomizer_push_current_game was called without any data."))?;
    }

    drop(app);

    populate_randomizer_choose_screen().await?;

    Ok(())
}

#[wasm_bindgen]
pub async fn randomizer_retire_current_game() -> Result<(), JsError> {
    let mut app = APP.try_write().expect("Should never actually have contention.");

    if let EGameRandomizer::Choosing(session) = &mut app.game_randomizer {
        let cur_game_idx = session.randomizer_list.shuffled_indices[session.cur_idx];
        let game = &mut session.randomizer_list.games[cur_game_idx];
        game.choose_state.retire();

        session.cur_idx = session.cur_idx + 1;
    }
    else {
        show_error(String::from("randomizer_retire_current_game was called without any data."))?;
    }

    drop(app);

    populate_randomizer_choose_screen().await?;

    Ok(())
}

#[wasm_bindgen]
pub async fn game_details_edit() -> Result<(), JsError> {
    let game = {
        let mut app = APP.try_write().expect("Should never actually have contention.");
        if app.details_screen_game.is_none() {
            show_error(String::from("No game on details screen to edit."))?;
            return Ok(());
        }
        std::mem::take(&mut app.details_screen_game)
    };

    edit_game(game.expect("checked above"))?;
    Ok(())
}

#[wasm_bindgen]
pub async fn game_details_reset() -> Result<(), JsError> {
    let game = {
        let mut app = APP.try_write().expect("Should never actually have contention.");
        if app.details_screen_game.is_none() {
            show_error(String::from("No game on details screen to edit."))?;
            return Ok(());
        }
        std::mem::take(&mut app.details_screen_game)
    };

    match server_api::reset_choose_state(&game.expect("checked above")).await {
        Ok(_) => show_result("Successfully reset game.")?,
        Err(e) => {
            let msg = format!("Failed to reset message, got error: {}", e);
            show_result(msg.as_str())?
        }
    }

    Ok(())
}
