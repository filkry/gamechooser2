use wasm_bindgen::{JsCast, JsValue, JsError};
use web_sys::{HtmlDivElement, HtmlImageElement, HtmlInputElement, HtmlParagraphElement, HtmlSpanElement};

pub trait TDOMElement {
    fn tag() -> &'static str;
}

pub trait TErgonomicDocument {
    fn create_element_typed<T: JsCast + TDOMElement>(&self) -> Result<T, JsError>;
    fn get_typed_element_by_id<T: JsCast>(&self, id: &str) -> Result<T, JsError>;
}

pub trait TToJsError {
    type OkType;

    fn to_jserr(self) -> Result<Self::OkType, JsError>;
}

// -- Implementations start here

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
}

impl TErgonomicDocument for web_sys::Document {
    fn create_element_typed<T: JsCast + TDOMElement>(&self) -> Result<T, JsError> {
        let elem = document().create_element(T::tag()).or(Err(JsError::new("Failed to create element")))?;

        match elem.dyn_into::<T>() {
            Ok(res) => Ok(res),
            Err(_) => Err(JsError::new("Created an element, but it somehow had the wrong type."))
        }
    }

    fn get_typed_element_by_id<T: JsCast>(&self, id: &str) -> Result<T, JsError> {
        let elem = document().get_element_by_id(id).ok_or(JsError::new("Element did not exist"))?;

        match elem.dyn_into::<T>() {
            Ok(res) => Ok(res),
            Err(_) => Err(JsError::new("get_typed_element_by_id found element, but it wasn't the desired type"))
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


