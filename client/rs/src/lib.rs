mod web;
mod server_api;

use std::sync::RwLock;

//use console_error_panic_hook;
use chrono;
use once_cell::sync::Lazy;
use js_sys::{Function};
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast};
use web_sys::{
    HtmlButtonElement,
    HtmlDivElement,
    HtmlElement,
    HtmlImageElement,
    HtmlInputElement,
    HtmlLabelElement,
    HtmlParagraphElement,
    HtmlSpanElement,
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

    last_search_igdb_results: Option<Vec<core::SGameInfo>>,

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
            session_screen_sessions: None,
            last_search_igdb_results: None,
            collection_screen_games: None,
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
    let name_search_input = &document.get_typed_element_by_id::<HtmlInputElement>("add_screen_name_search_input").to_jserr()?;
    let games : Vec<core::SGameInfo> = match server_api::search_igdb(name_search_input.value().as_str()).await {
        Ok(g) => g,
        Err(e) => {
            show_error(e)?;
            return Ok(());
        },
    };

    let output_elem = div("add_screen_search_igdb_output")?;
    output_elem.set_inner_html("");
    for game in &games {
        let game_div = document.create_element_typed::<HtmlDivElement>().to_jserr()?;
        output_elem.append_child(&game_div).to_jserr()?;

        let title_elem = document.create_element("h3").to_jserr()?;
        title_elem.set_text_content(Some(game.title()));
        game_div.append_child(&title_elem).to_jserr()?;

        if let Some(url) = game.cover_url() {
            let img_elem = document.create_element_typed::<HtmlImageElement>().to_jserr()?;
            img_elem.set_src(url);
            game_div.append_child(&img_elem).to_jserr()?;
        }

        let button_elem = document.create_element_typed::<HtmlButtonElement>().to_jserr()?;
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

fn edit_screen_populate_game_info(game_info: &core::SGameInfo) -> Result<(), JsError> {
    populate_text_input("game_edit_title", game_info.title())?;
    populate_date_input("game_edit_release_date", game_info.release_date())?;
    populate_img("game_edit_cover_art", game_info.cover_url())?;

    Ok(())
}

fn edit_screen_populate_custom_info(custom_info: &core::SGameCustomInfo) -> Result<(), JsError> {
    populate_checkox_input("game_edit_tag_couch", custom_info.tags.couch_playable)?;
    populate_checkox_input("game_edit_tag_portable", custom_info.tags.portable_playable)?;

    populate_checkox_input("game_edit_own_steam", custom_info.own.steam)?;
    populate_checkox_input("game_edit_own_egs", custom_info.own.egs)?;
    populate_checkox_input("game_edit_own_emulator", custom_info.own.emulator)?;
    populate_checkox_input("game_edit_own_ds", custom_info.own.ds)?;
    populate_checkox_input("game_edit_own_n3ds", custom_info.own.n3ds)?;
    populate_checkox_input("game_edit_own_wii", custom_info.own.wii)?;
    populate_checkox_input("game_edit_own_wiiu", custom_info.own.wiiu)?;
    populate_checkox_input("game_edit_own_switch", custom_info.own.switch)?;
    populate_checkox_input("game_edit_own_ps4", custom_info.own.ps4)?;
    populate_checkox_input("game_edit_own_ps5", custom_info.own.ps5)?;

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
    add_game(core::SAddCollectionGame::new(game))?;

    Ok(())
}

fn update_game_info_from_edit_screen(game_info: &mut core::SGameInfo) -> Result<(), JsError> {
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

fn update_custom_info_from_edit_screen(custom_info: &mut core::SGameCustomInfo) -> Result<(), JsError> {
    custom_info.via = document().get_typed_element_by_id::<HtmlInputElement>("game_edit_via").to_jserr()?.value();

    custom_info.tags = core::SGameTags {
        couch_playable: checkbox_value("game_edit_tag_couch")?,
        portable_playable: checkbox_value("game_edit_tag_portable")?,
    };

    custom_info.own = core::SOwn {
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
        ..Default::default()
    };

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
        let session_div = document.create_element_typed::<HtmlDivElement>().to_jserr()?;
        output_elem.append_child(&session_div).to_jserr()?;

        let title_elem = document.create_element("h3").to_jserr()?;
        title_elem.set_text_content(Some(session.game_info.title()));
        session_div.append_child(&title_elem).to_jserr()?;

        if let Some(url) = session.game_info.cover_url() {
            let img_elem = document.create_element_typed::<HtmlImageElement>().to_jserr()?;
            img_elem.set_src(url);
            session_div.append_child(&img_elem).to_jserr()?;
        }

        let start_date_elem = document.create_element_typed::<HtmlParagraphElement>().to_jserr()?;
        start_date_elem.set_inner_text(format!("Start date: {}", session.session.start_date).as_str());
        session_div.append_child(&start_date_elem).to_jserr()?;

        match session.session.state {
            core::ESessionState::Ongoing => {
                let memorable_elem = document.create_element_typed::<HtmlInputElement>().to_jserr()?;
                memorable_elem.set_type("checkbox");
                let memorable_elem_id = format!("session_screen_memorable_{}", session.session.internal_id);
                memorable_elem.set_id(memorable_elem_id.as_str());
                session_div.append_child(&memorable_elem).to_jserr()?;

                let memorable_elem_label = document.create_element_typed::<HtmlLabelElement>().to_jserr()?;
                memorable_elem_label.set_html_for(memorable_elem_id.as_str());
                memorable_elem_label.set_inner_text("Memorable");
                session_div.append_child(&memorable_elem_label).to_jserr()?;

                let button_elem = document.create_element_typed::<HtmlButtonElement>().to_jserr()?;
                let onclick_body = format!("session_screen_finish_session({});", session.session.internal_id);
                let onclick = Function::new_no_args(onclick_body.as_str());
                button_elem.set_onclick(Some(&onclick));
                button_elem.set_inner_text("Finish session");
                session_div.append_child(&button_elem).to_jserr()?;
            },
            core::ESessionState::Finished{end_date, memorable} => {
                let end_date_elem = document.create_element_typed::<HtmlParagraphElement>().to_jserr()?;
                end_date_elem.set_inner_text(format!("End date: {}", end_date).as_str());
                session_div.append_child(&end_date_elem).to_jserr()?;

                let memorable_elem = document.create_element_typed::<HtmlParagraphElement>().to_jserr()?;
                memorable_elem.set_inner_text(format!("Memorable: {}", memorable).as_str());
                session_div.append_child(&memorable_elem).to_jserr()?;
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

fn populate_collection_screen_game_list(games: Vec<core::SCollectionGame>) -> Result<(), JsError> {
    let doc = document();

    let output_elem = doc.get_typed_element_by_id::<HtmlDivElement>("collection_screen_game_list").to_jserr()?;
    output_elem.set_inner_html("");

    for game in &games {
        let game_div = doc.create_element_typed::<HtmlDivElement>().to_jserr()?;
        output_elem.append_child(&game_div).to_jserr()?;

        let title_elem = doc.create_element("h3").to_jserr()?;
        title_elem.set_text_content(Some(game.game_info.title()));
        game_div.append_child(&title_elem).to_jserr()?;

        if let Some(url) = game.game_info.cover_url() {
            let img_elem = doc.create_element_typed::<HtmlImageElement>().to_jserr()?;
            img_elem.set_src(url);
            game_div.append_child(&img_elem).to_jserr()?;
        }

        if let Some(d) = game.game_info.release_date() {
            let release_date_p = doc.create_element_typed::<HtmlParagraphElement>().to_jserr()?;
            release_date_p.set_inner_text(format!("Release date: {:?}", d).as_str());
            game_div.append_child(&release_date_p).to_jserr()?;
        }

        if game.custom_info.tags.couch_playable {
            let couch_span = doc.create_element_typed::<HtmlSpanElement>().to_jserr()?;
            couch_span.set_inner_text("✓ couch");
            game_div.append_child(&couch_span).to_jserr()?;
        }
        if game.custom_info.tags.portable_playable {
            let portable_span = doc.create_element_typed::<HtmlSpanElement>().to_jserr()?;
            portable_span.set_inner_text("✓ portable");
            game_div.append_child(&portable_span).to_jserr()?;
        }

        let add_own_tag = |test: bool, name: &str| -> Result<(), JsError> {
            if test {
                let own_span = document().create_element_typed::<HtmlSpanElement>().to_jserr()?;
                own_span.set_inner_text(format!("✓ {}", name).as_str());
                game_div.append_child(&own_span).to_jserr()?;
            }
            Ok(())
        };
        add_own_tag(game.custom_info.own.free, "free")?;
        add_own_tag(game.custom_info.own.steam, "steam")?;
        add_own_tag(game.custom_info.own.gmg, "gmg")?;
        add_own_tag(game.custom_info.own.gog, "gog")?;
        add_own_tag(game.custom_info.own.humble, "humble")?;
        add_own_tag(game.custom_info.own.origin, "origin")?;
        add_own_tag(game.custom_info.own.egs, "egs")?;
        add_own_tag(game.custom_info.own.battlenet, "battle.net")?;
        add_own_tag(game.custom_info.own.itch, "itch.io")?;
        add_own_tag(game.custom_info.own.standalone_launcher, "standalone launcher")?;
        add_own_tag(game.custom_info.own.emulator, "emulator")?;
        add_own_tag(game.custom_info.own.gba, "gba")?;
        add_own_tag(game.custom_info.own.ds, "ds")?;
        add_own_tag(game.custom_info.own.n3ds, "3ds")?;
        add_own_tag(game.custom_info.own.gamecube, "gamecube")?;
        add_own_tag(game.custom_info.own.wii, "wii")?;
        add_own_tag(game.custom_info.own.wiiu, "wiiu")?;
        add_own_tag(game.custom_info.own.switch, "switch")?;
        add_own_tag(game.custom_info.own.ps1, "ps1")?;
        add_own_tag(game.custom_info.own.ps2, "ps2")?;
        add_own_tag(game.custom_info.own.ps3, "ps3")?;
        add_own_tag(game.custom_info.own.ps4, "ps4")?;
        add_own_tag(game.custom_info.own.ps5, "ps5")?;
        add_own_tag(game.custom_info.own.psp, "psp")?;
        add_own_tag(game.custom_info.own.vita, "vita")?;
        add_own_tag(game.custom_info.own.xbox, "xbox")?;
        add_own_tag(game.custom_info.own.ios, "ios")?;
        add_own_tag(game.custom_info.own.oculus_quest, "oculus quest")?;
        add_own_tag(game.custom_info.own.ban_owned, "ban owned")?;

        let edit_button_elem = doc.create_element_typed::<HtmlButtonElement>().to_jserr()?;
        let onclick_body = format!("collection_screen_edit_game({});", game.internal_id);
        let onclick = Function::new_no_args(onclick_body.as_str());
        edit_button_elem.set_onclick(Some(&onclick));
        edit_button_elem.set_inner_text("Edit");
        game_div.append_child(&edit_button_elem).to_jserr()?;

        let start_sesion_button_elem = doc.create_element_typed::<HtmlButtonElement>().to_jserr()?;
        let onclick_body = format!("collection_screen_start_session({});", game.internal_id);
        let onclick = Function::new_no_args(onclick_body.as_str());
        start_sesion_button_elem.set_onclick(Some(&onclick));
        start_sesion_button_elem.set_inner_text("Start session");
        game_div.append_child(&start_sesion_button_elem).to_jserr()?;
    }

    // -- cache results for later use
    {
        let mut app = APP.try_write().expect("Should never actually have contention.");
        app.collection_screen_games = Some(games);
    }

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

#[wasm_bindgen]
pub async fn collection_screen_edit_game(internal_id: u32) -> Result<(), JsError> {
    let game_opt = {
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
    };
    let game = game_opt.ok_or(JsError::new("Somehow adding an IGDB game that was not in search results."))?;

    edit_game(game)
}

#[wasm_bindgen]
pub async fn collection_screen_start_session(internal_id: u32) -> Result<(), JsError> {
    let game_opt = {
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
    };
    let game = game_opt.ok_or(JsError::new("Somehow starting session for game not in the list."))?;

    let p = document().get_typed_element_by_id::<HtmlParagraphElement>("result_message").to_jserr()?;

    match server_api::start_session(game.internal_id).await {
        Ok(_) => p.set_inner_text("Successfully started session."),
        Err(_) => p.set_inner_text("Failed to start session."),
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

fn populate_randomizer_choose_screen() -> Result<(), JsError> {
    let game = {
        let app = APP.try_read().expect("Should never actually have contention.");
        if let EGameRandomizer::Choosing(session) = &app.game_randomizer {
            let game_idx = session.randomizer_list.shuffled_indices[session.cur_idx];
            session.randomizer_list.games[game_idx].clone()
        }
        else {
            return Err(JsError::new("populate_randomizer_choose_screen was called without any data."));
        }
    };

    populate_inner_text("randomizer_game_title", game.game_info.title())?;
    populate_img("randomizer_game_cover", game.game_info.cover_url())?;

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

    let max_passes = 3;

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

    {
        let mut app = APP.try_write().expect("Should never actually have contention.");
        app.game_randomizer = EGameRandomizer::Choosing(SGameRandomizerSession{
            randomizer_list,
            cur_idx: 0,
        });
    }

    populate_randomizer_choose_screen()?;

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
            if let Err(e) = server_api::update_choose_state(session.randomizer_list.games).await {
                show_error(e)?;
                return Ok(());
            }
        }
    }
    else {
        return Err(JsError::new("populate_randomizer_choose_screen was called without any data."));
    }

    Ok(())
}
