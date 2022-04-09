use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::{JsCast, JsError};
use web_sys::{Request, RequestInit, RequestMode, Response};

use gamechooser_core as core;
use super::web::{window, TToJsError};

pub(super) async fn search_igdb(title: &str) -> Result<Vec<core::SGame>, JsError> {
    let window = window();

    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);

    let origin = window.location().origin().to_jserr()?;
    let url = format!("{}/search_igdb/{}", origin.as_str(), title);
    let request = Request::new_with_str_and_init(&url, &opts).to_jserr()?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.to_jserr()?;
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().to_jserr()?;

    let json_promise = resp.json().to_jserr()?;
    let json = JsFuture::from(json_promise).await.to_jserr()?;

    json.into_serde().or(Err(JsError::new("Failed to serialize json into expected type.")))
}