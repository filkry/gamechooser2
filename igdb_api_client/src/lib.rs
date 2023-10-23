use std::error::Error;
use std::result::Result;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use gamechooser_core as core;

#[derive(Default, Serialize, Deserialize)]
pub struct SConfigFile {
    twitch_client_id: String,
    twitch_client_secret: String,
}

#[derive(Debug, Serialize, Clone)]
struct STwitchOauthTokenRequest {
    pub client_id: String,
    pub client_secret: String,
    pub grant_type: &'static str,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
struct STwitchOauthTokenResponse {
    access_token: String,
    expires_in: u32,
    token_type: String,
}

#[derive(Clone)]
pub struct STwitchAPIRequestBuilder {
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

#[derive(Clone)]
pub struct SReqwestTwitchAPISession {
    client: reqwest::Client,
    twitch_client_id: String,
    token_info: Option<STwitchOauthTokenResponse>,
}

pub struct SReqwestTwitchAPIClient {}

impl SConfigFile {
    pub fn set_twitch_client(&mut self, id: &str, secret: &str) {
        self.twitch_client_id = id.to_string();
        self.twitch_client_secret = secret.to_string();
    }
}

impl STwitchAPIRequestBuilder {
    pub fn new() -> Self {
        Self {
            url: String::new(),
            headers: Vec::new(),
            body: None,
        }
    }

    pub fn url(mut self, url: &str) -> Self {
        self.url = url.to_string();
        self
    }

    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_string(), value.to_string()));
        self
    }

    pub fn body(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());
        self
    }
}

#[derive(Deserialize, Debug)]
struct SIGDBInfoResultReleaseDate {
    date: i64, // unix timestamp
    status: u32,
}

fn best_release_date(dates: Vec<SIGDBInfoResultReleaseDate>) -> core::EReleaseDate {
    let mut best_date = core::EReleaseDate::UnknownUnreleased;
    let mut earliest = i64::MAX;

    for date in dates {
        // 6 should be full release - for 100% confidence should use API endpoint to look up status
        if date.status == 6 && date.date < earliest {
            best_date = core::EReleaseDate::Known(chrono::naive::NaiveDateTime::from_timestamp(date.date, 0).date());
            earliest = date.date;
        }
    }

    best_date
}

impl SReqwestTwitchAPIClient {
    pub async fn new_session() -> Result<SReqwestTwitchAPISession, String> {
        let cfg: SConfigFile = confy::load("gamechooser2_igdb_api_client").unwrap();

        let params = STwitchOauthTokenRequest {
            client_id: cfg.twitch_client_id,
            client_secret: cfg.twitch_client_secret,
            grant_type: "client_credentials",
        };

        Self::init(params).await
    }

    fn prepare_request(
        session: &SReqwestTwitchAPISession,
        rb: STwitchAPIRequestBuilder,
    ) -> reqwest::RequestBuilder {
        let mut request = session.client.post(rb.url);

        if let Some(b) = rb.body {
            request = request.body(b);
        }

        for (hname, hvalue) in rb.headers {
            request = request.header(hname.as_str(), hvalue.as_str());
        }

        request
    }

    async fn init(params: STwitchOauthTokenRequest) -> Result<SReqwestTwitchAPISession, String> {
        let client = reqwest::Client::new();

        let res = client
            .post("https://id.twitch.tv/oauth2/token")
            .form(&params)
            .send()
            .await;

        match res {
            Ok(res_) => {
                let resp: STwitchOauthTokenResponse = res_.json().await.unwrap();
                println!("{:?}", resp);
                Ok(SReqwestTwitchAPISession {
                    client,
                    twitch_client_id: params.client_id,
                    token_info: Some(resp),
                })
            }
            Err(e_) => Err(e_.to_string()),
        }
    }

    async fn post_interp_json<T: DeserializeOwned>(
        session: SReqwestTwitchAPISession,
        rb: STwitchAPIRequestBuilder,
    ) -> Result<T, Box<dyn Error>> {
        let req = Self::prepare_request(&session, rb);
        let resp = req.send().await?;
        Ok(resp.json().await?)
    }

    #[allow(dead_code)]
    async fn post_text(
        session: SReqwestTwitchAPISession,
        rb: STwitchAPIRequestBuilder,
    ) -> Result<String, String> {
        let req = Self::prepare_request(&session, rb);
        let resp = req.send().await.unwrap();
        Ok(resp.text().await.unwrap())
    }

    fn access_token(session: &SReqwestTwitchAPISession) -> &str {
        session.token_info.as_ref().unwrap().access_token.as_str()
    }

    pub async fn get_game_info(
        session: &SReqwestTwitchAPISession,
        igdb_id: u32,
    ) -> Result<core::EGameInfo, String> {
        #[derive(Deserialize, Debug)]
        #[allow(dead_code)]
        struct SIGDBInfoResultCover {
            id: u32,
            image_id: String,
        }
        #[derive(Deserialize, Debug)]
        struct SIGDBInfoResult {
            id: u32,
            name: String,
            slug: String,
            release_dates: Vec<SIGDBInfoResultReleaseDate>,
            cover: Option<SIGDBInfoResultCover>,
        }

        let mut query_results: Vec<SIGDBInfoResult> = {
            let where_clause = format!("where id = {};", igdb_id);
            let body = format!(
                "{}fields name,slug,release_dates.*,cover.image_id;",
                where_clause
            );

            let request = STwitchAPIRequestBuilder::new()
                .url("https://api.igdb.com/v4/games/")
                .header("Client-ID", session.twitch_client_id.as_str())
                .header(
                    "Authorization",
                    format!("Bearer {}", SReqwestTwitchAPIClient::access_token(&session)).as_str(),
                )
                .header("Accept", "application/json")
                .body(body.as_str());

            match SReqwestTwitchAPIClient::post_interp_json::<Vec<SIGDBInfoResult>>(
                session.clone(),
                request,
            )
            .await
            {
                Ok(res) => Ok(res),
                Err(e) => Err(format!("Failed with error {:?}", e)),
            }
        }?;

        if query_results.len() < 1 {
            return Err(format!("Got no results for IGDB game with ID {}", igdb_id));
        }

        if query_results.len() > 1 {
            return Err(format!(
                "Got more than one result for IGDB game with ID {}\n {:?}",
                igdb_id, query_results
            ));
        }

        assert!(query_results.len() == 1);
        let query_result = query_results.pop().expect("assert");

        let result = core::EGameInfo::new_igdb(
            query_result.id,
            query_result.slug.as_str(),
            query_result.cover.map(|c| c.image_id),
            query_result.name.as_str(),
            best_release_date(query_result.release_dates),
        );

        Ok(result)
    }

    pub async fn search(
        session: &SReqwestTwitchAPISession,
        name: &str,
        games_only: bool,
    ) -> Result<Vec<core::EGameInfo>, String> {
        #[derive(Deserialize, Debug)]
        #[allow(dead_code)]
        struct SIGDBSearchResultCover {
            id: u32,
            image_id: String,
        }
        #[derive(Deserialize, Debug)]
        struct SIGDBSearchResult {
            id: u32,
            name: String,
            slug: String,
            release_dates: Vec<SIGDBInfoResultReleaseDate>,
            cover: Option<SIGDBSearchResultCover>,
        }

        let search_results: Vec<SIGDBSearchResult> = {
            let where_clause = if games_only {
                "where category = 0 & version_parent = null;"
            } else {
                "where version_parent = null;"
            };
            let body = format!(
                "search \"{}\"; {}fields name,slug,release_dates.*,cover.image_id;",
                name, where_clause
            );

            /*
            Should be equivalent to:
            curl -d "search \"halo\"; fields name,release_dates,cover.url;" -H 'Client-ID: <ID>' -H 'Authorization: Bearer <BEARER>' -H 'Accept: application/json' https://api.igdb.com/v4/games
            */

            let request = STwitchAPIRequestBuilder::new()
                .url("https://api.igdb.com/v4/games/")
                .header("Client-ID", session.twitch_client_id.as_str())
                .header(
                    "Authorization",
                    format!("Bearer {}", SReqwestTwitchAPIClient::access_token(&session)).as_str(),
                )
                .header("Accept", "application/json")
                .body(body.as_str());

            match SReqwestTwitchAPIClient::post_interp_json::<Vec<SIGDBSearchResult>>(
                session.clone(),
                request,
            )
            .await
            {
                Ok(res) => Ok(res),
                Err(e) => Err(format!("Failed with error {:?}", e)),
            }
        }?;

        let mut results = Vec::with_capacity(search_results.len());
        for search_res in search_results {
            results.push(core::EGameInfo::new_igdb(
                search_res.id,
                search_res.slug.as_str(),
                search_res.cover.map(|c| c.image_id),
                search_res.name.as_str(),
                best_release_date(search_res.release_dates),
            ));
        }

        Ok(results)
    }

    // -- not pub because the API doesn't seem to work when you have 'search' in the queries
    #[allow(dead_code)]
    async fn multi_search(
        session: &SReqwestTwitchAPISession,
        names: &[&str],
    ) -> Result<Vec<Vec<core::EGameInfo>>, String> {
        if names.len() > 10 {
            return Err(String::from("Cannot multi-search for more than 10 games"));
        }

        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct SIGDBSearchResultCover {
            id: u32,
            image_id: String,
        }
        #[derive(Deserialize)]
        struct SIGDBSearchResult {
            id: u32,
            slug: String,
            name: String,
            release_dates: Vec<SIGDBInfoResultReleaseDate>,
            cover: Option<SIGDBSearchResultCover>,
        }

        #[derive(Deserialize)]
        struct SIGDBMultiSearchResults {
            r0: Option<Vec<SIGDBSearchResult>>,
            r1: Option<Vec<SIGDBSearchResult>>,
            r2: Option<Vec<SIGDBSearchResult>>,
            r3: Option<Vec<SIGDBSearchResult>>,
            r4: Option<Vec<SIGDBSearchResult>>,
            r5: Option<Vec<SIGDBSearchResult>>,
            r6: Option<Vec<SIGDBSearchResult>>,
            r7: Option<Vec<SIGDBSearchResult>>,
            r8: Option<Vec<SIGDBSearchResult>>,
            r9: Option<Vec<SIGDBSearchResult>>,
        }

        impl SIGDBMultiSearchResults {
            fn to_vec(self) -> Vec<Vec<SIGDBSearchResult>> {
                let mut result = Vec::with_capacity(10);

                if let Some(r) = self.r0 {
                    result.push(r);
                }
                if let Some(r) = self.r1 {
                    result.push(r);
                }
                if let Some(r) = self.r2 {
                    result.push(r);
                }
                if let Some(r) = self.r3 {
                    result.push(r);
                }
                if let Some(r) = self.r4 {
                    result.push(r);
                }
                if let Some(r) = self.r5 {
                    result.push(r);
                }
                if let Some(r) = self.r6 {
                    result.push(r);
                }
                if let Some(r) = self.r7 {
                    result.push(r);
                }
                if let Some(r) = self.r8 {
                    result.push(r);
                }
                if let Some(r) = self.r9 {
                    result.push(r);
                }

                result
            }
        }

        let mq_results: SIGDBMultiSearchResults = {
            let mut body = String::new();

            for (idx, name) in names.iter().enumerate() {
                let name_query = format!(
                    "
query games \"r{}\" {{
    search \"{}\";
    fields name,slug,release_dates.*,cover.image_id;
}};\n",
                    idx, name
                );

                body.push_str(name_query.as_str());
            }

            println!("{}", body);

            let request = STwitchAPIRequestBuilder::new()
                .url("https://api.igdb.com/v4/multiquery/")
                .header("Client-ID", session.twitch_client_id.as_str())
                .header(
                    "Authorization",
                    format!("Bearer {}", SReqwestTwitchAPIClient::access_token(&session)).as_str(),
                )
                .header("Accept", "application/json")
                .body(body.as_str());

            println!("\n\n\nTEXT\n\n\n");

            match SReqwestTwitchAPIClient::post_text(session.clone(), request.clone()).await {
                Ok(res) => {
                    println!("{:?}", res);
                }
                Err(e) => {
                    return Err(format!("Failed with error {:?}", e));
                }
            }

            println!("\n\n\nJSON\n\n\n");

            match SReqwestTwitchAPIClient::post_interp_json::<SIGDBMultiSearchResults>(
                session.clone(),
                request,
            )
            .await
            {
                Ok(res) => Ok(res),
                Err(e) => Err(format!("Failed with error {:?}", e)),
            }
        }?;

        fn extract_cover_url(cover: SIGDBSearchResultCover) -> String {
            cover.image_id
        }

        let mq_results_vec = mq_results.to_vec();

        let mut results = Vec::with_capacity(mq_results_vec.len());
        for query_result in mq_results_vec {
            let mut name_result = Vec::with_capacity(query_result.len());
            for igdb_game in query_result {
                name_result.push(core::EGameInfo::new_igdb(
                    igdb_game.id,
                    igdb_game.slug.as_str(),
                    igdb_game.cover.map(extract_cover_url),
                    igdb_game.name.as_str(),
                    best_release_date(igdb_game.release_dates),
                ));
            }
            results.push(name_result);
        }

        assert!(results.len() == names.len());

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
