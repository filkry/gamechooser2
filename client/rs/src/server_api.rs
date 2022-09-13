use serde_json;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Request, RequestInit, RequestMode, Response};

use gamechooser_core as core;
use super::web::{window, TToJsError};

#[allow(unused_macros)]
macro_rules! weblog {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

async fn check_err(resp: &Response) -> Result<(), String> {
    if !resp.ok() {
        let text_promise = resp.text().to_str_err()?;
        let text = JsFuture::from(text_promise).await.to_str_err()?;

        let msg = format!(
            "Server responded with status {} and message \"{}\"",
            resp.status(),
            text.as_string().unwrap_or(String::from("<NO MESSAGE>")),
        );
        return Err(msg);
    }

    Ok(())
}

pub(super) async fn search_igdb(title: &str, games_only: bool) -> Result<Vec<core::EGameInfo>, String> {
    let window = window();

    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);

    let origin = window.location().origin().to_str_err()?;
    let url = format!("{}/search_igdb/{}/{}", origin.as_str(), title, games_only);
    let request = Request::new_with_str_and_init(&url, &opts).to_str_err()?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.to_str_err()?;
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().to_str_err()?;

    let json_promise = resp.json().to_str_err()?;
    let json = JsFuture::from(json_promise).await.to_str_err()?;

    json.into_serde().or(Err(String::from("Failed to serialize json into expected type.")))
}

async fn post_data_return_data<S: serde::Serialize, T: serde::de::DeserializeOwned>(route: &str, data: S) -> Result<T, String> {
    let window = window();

    let data_json = serde_json::to_string(&data).unwrap();
    let body = JsValue::from_str(&data_json);

    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);
    opts.body(Some(&body));

    let origin = window.location().origin().to_str_err()?;
    let url = format!("{}/{}/", origin.as_str(), route);
    let request = Request::new_with_str_and_init(&url, &opts).to_str_err()?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.to_str_err()?;
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().to_str_err()?;

    check_err(&resp).await?;

    let json_promise = resp.json().to_str_err()?;
    let json = JsFuture::from(json_promise).await.to_str_err()?;

    match json.into_serde() {
        Ok(d) => Ok(d),
        Err(_) => Err(String::from("Failed to serialize json into expected type")),
    }
}

async fn post_return_data<T: serde::de::DeserializeOwned>(route: &str, url_data: Option<&str>) -> Result<T, String> {
    let window = window();

    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);

    let origin = window.location().origin().to_str_err()?;
    let url = {
        if let Some(d) = url_data {
            format!("{}/{}/{}", origin.as_str(), route, d)
        }
        else {
            format!("{}/{}/", origin.as_str(), route)
        }
    };
    let request = Request::new_with_str_and_init(&url, &opts).to_str_err()?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.to_str_err()?;
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().to_str_err()?;

    check_err(&resp).await?;

    let json_promise = resp.json().to_str_err()?;
    let json = JsFuture::from(json_promise).await.to_str_err()?;

    match json.into_serde() {
        Ok(d) => Ok(d),
        Err(_) => Err(String::from("Failed to serialize json into expected type")),
    }
}

async fn post_data<T: serde::Serialize>(route: &str, data: T) -> Result<(), String> {
    let window = window();

    let data_json = serde_json::to_string(&data).unwrap();
    let body = JsValue::from_str(&data_json);

    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);
    opts.body(Some(&body));

    let origin = window.location().origin().to_str_err()?;
    let url = format!("{}/{}/", origin.as_str(), route);
    let request = Request::new_with_str_and_init(&url, &opts).to_str_err()?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.to_str_err()?;
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().to_str_err()?;

    check_err(&resp).await?;

    Ok(())
}

async fn post(route: &str, url_data: Option<&str>) -> Result<(), String> {
    let window = window();

    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);

    let origin = window.location().origin().to_str_err()?;
    let url = {
        if let Some(d) = url_data {
            format!("{}/{}/{}", origin.as_str(), route, d)
        }
        else {
            format!("{}/{}/", origin.as_str(), route)
        }
    };
    let request = Request::new_with_str_and_init(&url, &opts).to_str_err()?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.to_str_err()?;
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().to_str_err()?;

    check_err(&resp).await?;

    Ok(())
}

pub(super) async fn add_game(game: core::SAddCollectionGame) -> Result<(), String> {
    post_data("add_game", game).await?;
    Ok(())
}

pub(super) async fn edit_game(game: core::SCollectionGame) -> Result<(), String> {
    post_data("edit_game", game).await?;
    Ok(())
}

pub(super) async fn get_recent_collection_games() -> Result<Vec<core::SCollectionGame>, String> {
    post_return_data("get_recent_collection_games", None).await
}

pub(super) async fn update_igdb_games() -> Result<(), String> {
    post("update_igdb_games", None).await?;
    Ok(())
}

pub(super) async fn search_collection(query: &str) -> Result<Vec<core::SCollectionGame>, String> {
    post_return_data("search_collection", Some(query)).await
}

pub(super) async fn get_full_collection() -> Result<Vec<core::SCollectionGame>, String> {
    post_return_data("get_full_collection", None).await
}

pub(super) async fn get_sessions(
    game_id: Option<u32>,
    active_only: bool,
    memorable_only: bool,
    year: Option<u32>,
) -> Result<Vec<core::SSessionAndCollectionGame>, String> {

    let filter = core::SSessionFilter {
        game_id,
        active_only,
        memorable_only,
        year,
    };

    post_data_return_data("get_sessions", filter).await
}

pub(super) async fn start_session(internal_id: u32) -> Result<(), String> {
    let data_str = format!("{}", internal_id);
    post("start_session", Some(data_str.as_str())).await
}

pub(super) async fn finish_session(internal_id: u32, memorable: bool, retire: bool, set_ignore_passes: bool) -> Result<(), String> {
    let data_str = format!("{}/{}/{}/{}", internal_id, memorable, retire, set_ignore_passes);
    post("finish_session", Some(data_str.as_str())).await
}

pub(super) async fn get_randomizer_games(filter: core::SRandomizerFilter) -> Result<core::SRandomizerList, String> {
    post_data_return_data("get_randomizer_games", filter).await
}

pub(super) async fn update_choose_state(games: &Vec<core::SCollectionGame>) -> Result<(), String> {
    post_data("update_choose_state", games).await?;
    Ok(())
}

pub(super) async fn reset_choose_state(game: &core::SCollectionGame) -> Result<(), String> {
    let data_str = format!("{}", game.internal_id);
    post("reset_choose_state", Some(data_str.as_str())).await
}

pub(super) async fn simple_stats() -> Result<core::SSimpleStats, String> {
    post_return_data("simple_stats", None).await
}

pub(super) async fn check_logged_in() -> bool {
    post("check_logged_in", None).await.is_ok()
}

pub(super) async fn login(secret: &str) -> Result<(), String> {
    let secret_str = format!("{}", secret);
    post_return_data("login", Some(secret_str.as_str())).await
}
