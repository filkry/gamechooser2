use std::fmt::Write;

use wasm_bindgen::prelude::*;
use web_sys::{
    HtmlDivElement,
};

use super::web::{document, TToJsError, TErgonomicDocument};

pub fn create_class_percentage_chart(parent: &HtmlDivElement, classes: &[u32], _class_names: Option<&[&str]>) -> Result<(), JsError> {
    let colors = [
        (0xff, 0xa6, 0),
        (0xff, 0x63, 0x61),
        (0xbc, 0x50, 0x90),
        (0x58, 0x50, 0x8d),
        (0x00, 0x3f, 0x5c),
    ];

    let document = document();

    let container = document.create_element_typed::<HtmlDivElement>().to_jserr()?;
    container.set_class_name("percentage_chart_container");
    parent.append_child(&container).to_jserr()?;

    let mut inner_html = String::new();

    let mut total = 0;
    for class_val in classes {
        total = total + class_val;
    }

    inner_html.push_str("<svg class=\"percentage_chart\">");

    let mut cur_percent : f32 = 0.0;
    for i in 0..classes.len() {
        let class_percent = classes[i] as f32 / total as f32;

        write!(inner_html, "<rect x=\"{}%\", width=\"{}%\", height=\"100%\", style=\"fill:rgb({}, {}, {});\"/>",
            cur_percent * 100.0,
            class_percent * 100.0,
            colors[i].0,
            colors[i].1,
            colors[i].2,
        ).unwrap();

        cur_percent = cur_percent + class_percent;
    }

    inner_html.push_str("<rect width=\"100%\" height=\"100%\" style=\"fill:none;stroke:black;stroke-width:1;\"/>");

    inner_html.push_str("</svg>");

    container.set_inner_html(inner_html.as_str());

    Ok(())
}

pub fn create_binary_percentage_chart(parent: &HtmlDivElement, true_count: u32, total: u32) -> Result<(), JsError> {
    let classes = [true_count, total - true_count];
    create_class_percentage_chart(parent, &classes, None)
}
