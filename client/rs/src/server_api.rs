use serde_json;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::{JsCast, JsError, JsValue};
use web_sys::{Request, RequestInit, RequestMode, Response};

use gamechooser_core as core;
use super::web::{window, TToJsError};

pub(super) async fn search_igdb(title: &str) -> Result<Vec<core::SGameInfo>, JsError> {
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

async fn post_data_return_data<S: serde::Serialize, T: serde::de::DeserializeOwned>(route: &str, data: S) -> Result<T, JsError> {
    let window = window();

    let data_json = serde_json::to_string(&data).unwrap();
    let body = JsValue::from_str(&data_json);

    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);
    opts.body(Some(&body));

    let origin = window.location().origin().to_jserr()?;
    let url = format!("{}/{}/", origin.as_str(), route);
    let request = Request::new_with_str_and_init(&url, &opts).to_jserr()?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.to_jserr()?;
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().to_jserr()?;

    let json_promise = resp.json().to_jserr()?;
    let json = JsFuture::from(json_promise).await.to_jserr()?;

    match json.into_serde() {
        Ok(d) => Ok(d),
        Err(_) => Err(JsError::new("Failed to serialize json into expected type")),
    }
}

async fn post_return_data<T: serde::de::DeserializeOwned>(route: &str, url_data: Option<&str>) -> Result<T, JsError> {
    let window = window();

    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);

    let origin = window.location().origin().to_jserr()?;
    let url = {
        if let Some(d) = url_data {
            format!("{}/{}/{}", origin.as_str(), route, d)
        }
        else {
            format!("{}/{}/", origin.as_str(), route)
        }
    };
    let request = Request::new_with_str_and_init(&url, &opts).to_jserr()?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.to_jserr()?;
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().to_jserr()?;

    let json_promise = resp.json().to_jserr()?;
    let json = JsFuture::from(json_promise).await.to_jserr()?;

    match json.into_serde() {
        Ok(d) => Ok(d),
        Err(_) => Err(JsError::new("Failed to serialize json into expected type")),
    }
}

async fn post_data<T: serde::Serialize>(route: &str, data: T) -> Result<(), JsError> {
    let window = window();

    let data_json = serde_json::to_string(&data).unwrap();
    let body = JsValue::from_str(&data_json);

    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);
    opts.body(Some(&body));

    let origin = window.location().origin().to_jserr()?;
    let url = format!("{}/{}/", origin.as_str(), route);
    let request = Request::new_with_str_and_init(&url, &opts).to_jserr()?;

    match JsFuture::from(window.fetch_with_request(&request)).await.to_jserr() {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

async fn post(route: &str, url_data: Option<&str>) -> Result<(), JsError> {
    let window = window();

    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);

    let origin = window.location().origin().to_jserr()?;
    let url = {
        if let Some(d) = url_data {
            format!("{}/{}/{}", origin.as_str(), route, d)
        }
        else {
            format!("{}/{}/", origin.as_str(), route)
        }
    };
    let request = Request::new_with_str_and_init(&url, &opts).to_jserr()?;

    match JsFuture::from(window.fetch_with_request(&request)).await.to_jserr() {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub(super) async fn add_game(game: core::SAddCollectionGame) -> Result<(), JsError> {
    post_data("add_game", game).await?;
    Ok(())
}

pub(super) async fn edit_game(game: core::SCollectionGame) -> Result<(), JsError> {
    post_data("edit_game", game).await?;
    Ok(())
}

pub(super) async fn get_recent_collection_games() -> Result<Vec<core::SCollectionGame>, JsError> {
    post_return_data("get_recent_collection_games", None).await
}

pub(super) async fn search_collection(query: &str) -> Result<Vec<core::SCollectionGame>, JsError> {
    post_return_data("search_collection", Some(query)).await
}

pub(super) async fn get_active_sessions() -> Result<Vec<core::SSessionAndGameInfo>, JsError> {
    post_return_data("get_active_sessions", None).await
}

pub(super) async fn start_session(internal_id: u32) -> Result<(), JsError> {
    let data_str = format!("{}", internal_id);
    post("start_session", Some(data_str.as_str())).await
}

pub(super) async fn finish_session(internal_id: u32, memorable: bool) -> Result<(), JsError> {
    let data_str = format!("{}/{}", internal_id, memorable);
    post("finish_session", Some(data_str.as_str())).await
}

pub(super) async fn get_randomizer_games(filter: core::SRandomizerFilter) -> Result<Vec<core::SCollectionGame>, JsError> {
    post_data_return_data("get_randomizer_games", filter).await
}
