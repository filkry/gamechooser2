use js_sys::{Function};
use wasm_bindgen::prelude::*;
use web_sys::{
    HtmlAnchorElement,
    HtmlButtonElement,
    HtmlDivElement,
    //HtmlElement,
    HtmlImageElement,
    //HtmlInputElement,
    //HtmlLabelElement,
    //HtmlLiElement,
    HtmlParagraphElement,
    //HtmlSelectElement,
    HtmlSpanElement,
    //HtmlUListElement,
};

use gamechooser_core as core;
use super::web::{document, TToJsError, TErgonomicDocument};

enum EGame {
    GameInfo(core::EGameInfo),
    CollectionGame(core::SCollectionGame),
}

pub struct SGameCard {
    game: EGame, // copy

    pub main_div: HtmlDivElement,
    _columns_div: HtmlDivElement,
    info_column_div: HtmlDivElement,
    generated_info_div: HtmlDivElement,
    pub customizable_info_div: Option<HtmlDivElement>,

    show_release_date: bool,
    show_igdb_link: bool,
    show_via: bool,
    show_own_info: bool,
    show_tag_info: bool,
}

pub struct SCompactGameCard {
    game: EGame, // copy

    pub main_div: HtmlDivElement,
}

impl EGame {
    fn game_info(&self) -> &core::EGameInfo {
        match self {
            Self::GameInfo(gi) => &gi,
            Self::CollectionGame(cg) => &cg.game_info,
        }
    }

    fn custom_info(&self) -> Option<&core::SGameCustomInfo> {
        match self {
            Self::GameInfo(_) => None,
            Self::CollectionGame(cg) => Some(&cg.custom_info),
        }
    }
}

impl SGameCard {
    fn new_internal(game: EGame) -> Result<Self, JsError> {
        let document = document();

        let main_div = document.create_element_typed::<HtmlDivElement>().to_jserr()?;
        main_div.set_class_name("game_card");

        let header_div = document.create_element_typed::<HtmlDivElement>().to_jserr()?;
        header_div.set_class_name("game_card_header_div");
        main_div.append_child(&header_div).to_jserr()?;

        let title_elem = document.create_element("h3").to_jserr()?;
        title_elem.set_text_content(Some(game.game_info().title()));
        header_div.append_child(&title_elem).to_jserr()?;

        let columns_div = document.create_element_typed::<HtmlDivElement>().to_jserr()?;
        columns_div.set_class_name("game_card_columns");
        main_div.append_child(&columns_div).to_jserr()?;

        if let Some(url) = game.game_info().cover_url() {
            let cover_div = document.create_element_typed::<HtmlDivElement>().to_jserr()?;
            cover_div.set_class_name("game_card_cover_div");
            columns_div.append_child(&cover_div).to_jserr()?;

            let img_elem = document.create_element_typed::<HtmlImageElement>().to_jserr()?;
            img_elem.set_src(url.as_str());
            cover_div.append_child(&img_elem).to_jserr()?;
        }

        let info_column_div = document.create_element_typed::<HtmlDivElement>().to_jserr()?;
        info_column_div.set_class_name("game_card_info");
        columns_div.append_child(&info_column_div).to_jserr()?;

        let generated_info_div = document.create_element_typed::<HtmlDivElement>().to_jserr()?;
        info_column_div.append_child(&generated_info_div).to_jserr()?;

        if let EGame::CollectionGame(collection_game) = &game {
            let header_buttons_div = document.create_element_typed::<HtmlDivElement>().to_jserr()?;
            header_buttons_div.set_class_name("game_card_footer_buttons_div");
            main_div.append_child(&header_buttons_div).to_jserr()?;

            {
                let info_button_elem = document.create_element_typed::<HtmlButtonElement>().to_jserr()?;
                let onclick_body = format!("game_card_view_details({});", collection_game.internal_id);
                let onclick = Function::new_no_args(onclick_body.as_str());
                info_button_elem.set_onclick(Some(&onclick));
                info_button_elem.set_inner_text("ℹ");
                header_buttons_div.append_child(&info_button_elem).to_jserr()?;
            }

            {
                let edit_button_elem = document.create_element_typed::<HtmlButtonElement>().to_jserr()?;
                let onclick_body = format!("edit_cached_game({});", collection_game.internal_id);
                let onclick = Function::new_no_args(onclick_body.as_str());
                edit_button_elem.set_onclick(Some(&onclick));
                edit_button_elem.set_inner_text("✎");
                header_buttons_div.append_child(&edit_button_elem).to_jserr()?;
            }
        }

        Ok(Self {
            game,
            main_div,
            _columns_div: columns_div,
            info_column_div,
            generated_info_div,
            customizable_info_div: None,
            show_release_date: false,
            show_igdb_link: false,
            show_own_info: false,
            show_tag_info: false,
            show_via: false,
        })
    }

    pub fn new_from_game_info(game_info: &core::EGameInfo) -> Result<Self, JsError> {
        Self::new_internal(EGame::GameInfo(game_info.clone()))
    }

    pub fn new_from_collection_game(collection_game: &core::SCollectionGame) -> Result<Self, JsError> {
        Self::new_internal(EGame::CollectionGame(collection_game.clone()))
    }

    pub fn show_all(&mut self) -> &mut Self {
        self.show_release_date = true;
        self.show_igdb_link = true;
        self.show_own_info = true;
        self.show_tag_info = true;
        self.show_via = true;
        self
    }

    pub fn show_release_date(&mut self) -> &mut Self{
        self.show_release_date = true;
        self
    }

    pub fn show_igdb_link(&mut self) -> &mut Self {
        self.show_igdb_link = true;
        self
    }

    pub fn show_own_info(&mut self) -> &mut Self {
        self.show_own_info = true;
        self
    }

    pub fn show_tag_info(&mut self) -> &mut Self {
        self.show_tag_info = true;
        self
    }

    pub fn show_via(&mut self) -> &mut Self {
        self.show_via = true;
        self
    }

    pub fn customizable_div(&mut self) -> Result<&HtmlDivElement, JsError> {
        if let None = self.customizable_info_div {
            let new_div = document().create_element_typed::<HtmlDivElement>().to_jserr()?;
            self.info_column_div.append_child(&new_div).to_jserr()?;
            new_div.set_class_name("game_card_customizable_div");
            self.customizable_info_div = Some(new_div);
        }

        Ok(self.customizable_info_div.as_ref().expect("created above"))
    }

    pub fn regen(&mut self) -> Result<(), JsError> {
        let document = document();

        self.generated_info_div.set_inner_html(""); // reset

        // game_info elements
        if self.show_release_date {
            if let Some(d) = self.game.game_info().release_date() {
                let release_date_p = document.create_element_typed::<HtmlParagraphElement>().to_jserr()?;
                release_date_p.set_inner_text(format!("Release date: {:?}", d).as_str());
                self.generated_info_div.append_child(&release_date_p).to_jserr()?;
            }
        }

        if self.show_igdb_link {
            if let core::EGameInfo::IGDB(igdb) = &self.game.game_info() {
                let a = create_igdb_link(&igdb)?;
                self.generated_info_div.append_child(&a).to_jserr()?;
            }
        }

        // custom_info elements
        if let Some(custom_info) = &self.game.custom_info() {
            if self.show_via {
                if custom_info.via.len() > 0 {
                    let via_text = format!("Via: {}", custom_info.via);

                    let via_p = document.create_element_typed::<HtmlParagraphElement>().to_jserr()?;
                    via_p.set_inner_text(via_text.as_str());
                    self.generated_info_div.append_child(&via_p).to_jserr()?;
                }
            }

            let checks_container = document.create_element_typed::<HtmlDivElement>().to_jserr()?;
            self.generated_info_div.append_child(&checks_container).to_jserr()?;

            if self.show_own_info {
                create_own_checks(&custom_info.own, &checks_container)?;
            }

            if self.show_tag_info {
                create_tag_checks(&custom_info.tags, &checks_container)?;
            }
        }

        Ok(())
    }

}

impl SCompactGameCard {
    fn new_internal(game: EGame) -> Result<Self, JsError> {
        let document = document();

        let main_div = document.create_element_typed::<HtmlDivElement>().to_jserr()?;
        main_div.set_class_name("compact_game_card");

        let columns_div = document.create_element_typed::<HtmlDivElement>().to_jserr()?;
        columns_div.set_class_name("compact_game_card_columns");
        main_div.append_child(&columns_div).to_jserr()?;

        let cover_div = document.create_element_typed::<HtmlDivElement>().to_jserr()?;
        cover_div.set_class_name("compact_game_card_cover_div");
        columns_div.append_child(&cover_div).to_jserr()?;

        if let Some(url) = game.game_info().cover_url() {
            let img_elem = document.create_element_typed::<HtmlImageElement>().to_jserr()?;
            img_elem.set_src(url.as_str());
            cover_div.append_child(&img_elem).to_jserr()?;
        }
        else {

            let img_elem = document.create_element_typed::<HtmlImageElement>().to_jserr()?;
            img_elem.set_src("controller_image.png");
            cover_div.append_child(&img_elem).to_jserr()?;
        }

        let info_column_div = document.create_element_typed::<HtmlDivElement>().to_jserr()?;
        info_column_div.set_class_name("game_card_info");
        columns_div.append_child(&info_column_div).to_jserr()?;

        let title_elem = document.create_element("h3").to_jserr()?;
        title_elem.set_text_content(Some(game.game_info().title()));
        info_column_div.append_child(&title_elem).to_jserr()?;

        let generated_info_div = document.create_element_typed::<HtmlDivElement>().to_jserr()?;
        info_column_div.append_child(&generated_info_div).to_jserr()?;

        // custom_info elements
        if let Some(custom_info) = &game.custom_info() {
            let checks_container = document.create_element_typed::<HtmlDivElement>().to_jserr()?;
            generated_info_div.append_child(&checks_container).to_jserr()?;

            create_tag_checks(&custom_info.tags, &checks_container)?;
        }

        if let EGame::CollectionGame(collection_game) = &game {
            let floating_buttons_div = document.create_element_typed::<HtmlDivElement>().to_jserr()?;
            floating_buttons_div.set_class_name("compact_game_card_floating_buttons_div");
            main_div.append_child(&floating_buttons_div).to_jserr()?;

            {
                let info_button_elem = document.create_element_typed::<HtmlButtonElement>().to_jserr()?;
                let onclick_body = format!("game_card_view_details({});", collection_game.internal_id);
                let onclick = Function::new_no_args(onclick_body.as_str());
                info_button_elem.set_onclick(Some(&onclick));
                info_button_elem.set_inner_text("ℹ");
                floating_buttons_div.append_child(&info_button_elem).to_jserr()?;
            }

            {
                let edit_button_elem = document.create_element_typed::<HtmlButtonElement>().to_jserr()?;
                let onclick_body = format!("edit_cached_game({});", collection_game.internal_id);
                let onclick = Function::new_no_args(onclick_body.as_str());
                edit_button_elem.set_onclick(Some(&onclick));
                edit_button_elem.set_inner_text("✎");
                floating_buttons_div.append_child(&edit_button_elem).to_jserr()?;
            }
        }

        Ok(Self {
            game,
            main_div,
        })
    }

    pub fn new_from_collection_game(collection_game: &core::SCollectionGame) -> Result<Self, JsError> {
        Self::new_internal(EGame::CollectionGame(collection_game.clone()))
    }
}

// HELPER FUNCTIONS

fn create_igdb_link(igdb: &core::SGameInfoIGDB) -> Result<HtmlAnchorElement, JsError> {
    let a = document().create_element_typed::<HtmlAnchorElement>().to_jserr()?;
    let page_url = format!("https://www.igdb.com/games/{}", igdb.slug);
    a.set_inner_text("IGDB page ⧉");
    a.set_href(page_url.as_str());
    a.set_target("_blank");
    a.set_rel("noopener noreferrer");
    Ok(a)
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
