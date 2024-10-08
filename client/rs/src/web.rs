use wasm_bindgen::{JsCast, JsValue, JsError};
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
    SvgElement,
    SvgRectElement,
};

pub trait TDOMElement {
    fn tag() -> &'static str;
}

// -- $$$FRK(TODO): the error for this is simple, doesn't need to be a string
pub trait TErgonomicDocument {
    fn create_element_typed<T: JsCast + TDOMElement>(&self) -> Result<T, String>;
    fn get_typed_element_by_id<T: JsCast>(&self, id: &str) -> Result<T, String>;
}

pub trait TToJsError {
    type OkType;

    fn to_jserr(self) -> Result<Self::OkType, JsError>;
    fn to_str_err(self) -> Result<Self::OkType, String>;
}

// -- Implementations start here

impl TDOMElement for HtmlAnchorElement {
    fn tag() -> &'static str {
        "a"
    }
}
impl TDOMElement for HtmlButtonElement {
    fn tag() -> &'static str {
        "button"
    }
}
impl TDOMElement for HtmlDivElement {
    fn tag() -> &'static str {
        "div"
    }
}
impl TDOMElement for HtmlImageElement {
    fn tag() -> &'static str {
        "img"
    }
}
impl TDOMElement for HtmlInputElement {
    fn tag() -> &'static str {
        "input"
    }
}
impl TDOMElement for HtmlLabelElement {
    fn tag() -> &'static str {
        "label"
    }
}
impl TDOMElement for HtmlLiElement {
    fn tag() -> &'static str {
        "li"
    }
}
impl TDOMElement for HtmlParagraphElement {
    fn tag() -> &'static str {
        "p"
    }
}
impl TDOMElement for HtmlSpanElement {
    fn tag() -> &'static str {
        "span"
    }
}
impl TDOMElement for HtmlUListElement {
    fn tag() -> &'static str {
        "ul"
    }
}
impl TDOMElement for SvgElement {
    fn tag() -> &'static str {
        "svg"
    }
}
impl TDOMElement for SvgRectElement {
    fn tag() -> &'static str {
        "rect"
    }
}

impl<T> TToJsError for Result<T, JsValue> {
    type OkType = T;

    fn to_jserr(self) -> Result<Self::OkType, JsError> {
        match self {
            Ok(o) => Ok(o),
            Err(e) => {
                let temp = format!("{:?}", e);
                Err(JsError::new(temp.as_str()))
            }
        }
    }

    fn to_str_err(self) -> Result<Self::OkType, String> {
        match self {
            Ok(o) => Ok(o),
            Err(e) => {
                let temp = format!("{:?}", e);
                Err(temp)
            }
        }
    }
}

impl<T> TToJsError for Result<T, String> {
    type OkType = T;

    fn to_jserr(self) -> Result<Self::OkType, JsError> {
        match self {
            Ok(o) => Ok(o),
            Err(e) => {
                Err(JsError::new(e.as_str()))
            }
        }
    }

    fn to_str_err(self) -> Result<Self::OkType, String> {
        self
    }
}

impl TErgonomicDocument for web_sys::Document {
    fn create_element_typed<T: JsCast + TDOMElement>(&self) -> Result<T, String> {
        let elem = document().create_element(T::tag()).or(Err(String::from("Failed to create element")))?;

        match elem.dyn_into::<T>() {
            Ok(res) => Ok(res),
            Err(_) => Err(String::from("Created an element, but it somehow had the wrong type."))
        }
    }

    fn get_typed_element_by_id<T: JsCast>(&self, id: &str) -> Result<T, String> {
        let elem = document().get_element_by_id(id).ok_or(String::from("Element did not exist"))?;

        match elem.dyn_into::<T>() {
            Ok(res) => Ok(res),
            Err(_) => Err(String::from("get_typed_element_by_id found element, but it wasn't the desired type"))
        }
    }
}

pub fn window() -> web_sys::Window {
    let window = web_sys::window().expect("no global `window` exists");
    window
}

pub fn document() -> web_sys::Document {
    let window = window();
    let document = window.document().expect("should have a document on window");

    document
}

pub fn create_checkbox(initial_val: bool, elem_id: &str, label_text: &str, output_elem: &HtmlElement, in_li: bool) -> Result<(), JsError> {
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


