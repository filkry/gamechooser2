use js_sys::{Function};
use wasm_bindgen::prelude::*;
use web_sys::{
    //HtmlAnchorElement,
    HtmlButtonElement,
    HtmlDivElement,
    //HtmlElement,
    //HtmlImageElement,
    //HtmlInputElement,
    //HtmlLabelElement,
    //HtmlLiElement,
    HtmlParagraphElement,
    //HtmlSelectElement,
    //HtmlSpanElement,
    HtmlUListElement,
};

use gamechooser_core as core;
use super::web::{document, TToJsError, TErgonomicDocument, create_checkbox};

pub struct SSessionCard {
    _session: core::SSession, // copy

    pub main_div: HtmlDivElement,
}

impl SSessionCard {
    pub fn new(session: core::SSession, editable: bool, title: Option<&str>) -> Result<Self, JsError> {
        let document = document();

        let main_div = document.create_element_typed::<HtmlDivElement>().to_jserr()?;
        main_div.set_class_name("session_card");

        let header_div = document.create_element_typed::<HtmlDivElement>().to_jserr()?;
        header_div.set_class_name("session_card_header_div");
        main_div.append_child(&header_div).to_jserr()?;

        let title_elem = document.create_element("h4").to_jserr()?;
        if title.is_some() {
            title_elem.set_text_content(title);
        }
        else {
            title_elem.set_text_content(Some("Session"));
        }
        header_div.append_child(&title_elem).to_jserr()?;

        // populate session_div with session-specific info
        let start_date_elem = document.create_element_typed::<HtmlParagraphElement>().to_jserr()?;
        start_date_elem.set_inner_text(format!("Start date: {}", session.start_date).as_str());
        main_div.append_child(&start_date_elem).to_jserr()?;

        match session.state {
            core::ESessionState::Ongoing => {
                if editable {
                    let checkbox_list = document.create_element_typed::<HtmlUListElement>().to_jserr()?;
                    checkbox_list.set_class_name("checkbox_list");
                    main_div.append_child(&checkbox_list).to_jserr()?;

                    let memorable_elem_id = format!("session_screen_memorable_{}", session.internal_id);
                    create_checkbox(false, memorable_elem_id.as_str(), "Memorable", &checkbox_list, true)?;

                    let retire_elem_id = format!("session_screen_retire_{}", session.internal_id);
                    create_checkbox(false, retire_elem_id.as_str(), "Retire", &checkbox_list, true)?;

                    let ignore_passes_id = format!("session_screen_ignore_passes_{}", session.internal_id);
                    create_checkbox(false, ignore_passes_id.as_str(), "Allow infinite passes", &checkbox_list, true)?;

                    let button_elem = document.create_element_typed::<HtmlButtonElement>().to_jserr()?;
                    let onclick_body = format!("session_screen_finish_session({});", session.internal_id);
                    let onclick = Function::new_no_args(onclick_body.as_str());
                    button_elem.set_onclick(Some(&onclick));
                    button_elem.set_inner_text("Finish session");
                    main_div.append_child(&button_elem).to_jserr()?;
                }
            },
            core::ESessionState::Finished{end_date, memorable} => {
                let end_date_elem = document.create_element_typed::<HtmlParagraphElement>().to_jserr()?;
                end_date_elem.set_inner_text(format!("End date: {}", end_date).as_str());
                main_div.append_child(&end_date_elem).to_jserr()?;

                let memorable_elem = document.create_element_typed::<HtmlParagraphElement>().to_jserr()?;
                memorable_elem.set_inner_text(format!("Memorable: {}", memorable).as_str());
                main_div.append_child(&memorable_elem).to_jserr()?;
            }
        }

        Ok(Self {
            _session: session,
            main_div,
        })
    }

    #[allow(dead_code)]
    pub fn regen(&mut self) -> Result<(), JsError> {
        let _document = document();

        Ok(())
    }

}
