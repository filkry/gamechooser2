use chrono;
use serde::{Serialize, Deserialize};

use crate::database_v3;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SGameInfoCustom {
    pub title: String,
    release_date: Option<chrono::naive::NaiveDate>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SGameInfoIGDB {
    pub id: u32,
    pub slug: String,
    pub cached_title: String,
    pub cached_release_date: Option<chrono::naive::NaiveDate>,
    pub cached_cover_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EGameInfo {
    Custom(SGameInfoCustom),
    IGDB(SGameInfoIGDB),
}

pub type SGameCustomInfo = database_v3::SGameCustomInfo;
pub type SGameChooseState = database_v3::SGameChooseState;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SCollectionGame {
    pub internal_id: u32, // $$$FRK(TODO): These internal IDs should have a type for type validation, but I'm lazy right now
    pub game_info: EGameInfo,
    pub custom_info: SGameCustomInfo,
    pub choose_state: SGameChooseState,
}

pub type SSession = database_v3::SSession;

#[derive(Debug, Serialize, Deserialize)]
pub struct SDatabase {
    pub games: Vec<SCollectionGame>,
    pub sessions: Vec<SSession>,
}

impl SGameInfoIGDB {
    pub fn cover_url(&self) -> Option<String> {
        if let Some(cover_id) = &self.cached_cover_id {
            return Some(format!("https://images.igdb.com/igdb/image/upload/t_cover_big/{}.jpg", cover_id));
        }

        None
    }
}

impl EGameInfo {
    pub fn new_igdb(igdb_id: u32, slug: &str, cover_id: Option<String>, title: &str, release_date: Option<chrono::naive::NaiveDate>) -> Self {
        Self::IGDB(
            SGameInfoIGDB{
                id: igdb_id,
                slug: String::from(slug),
                cached_title: String::from(title),
                cached_cover_id: cover_id,
                cached_release_date: release_date,
            }
        )
    }

    pub fn new_custom(title: String, release_date: Option<chrono::naive::NaiveDate>) -> Self {
        Self::Custom(
            SGameInfoCustom {
                title,
                release_date,
            }
        )
    }

    pub fn title(&self) -> &str {
        match self {
            Self::IGDB(inner) => inner.cached_title.as_str(),
            Self::Custom(inner) => inner.title.as_str(),
        }
    }

    pub fn release_date(&self) -> Option<chrono::naive::NaiveDate> {
        match self {
            Self::IGDB(inner) => inner.cached_release_date,
            Self::Custom(inner) => inner.release_date,
        }
    }

    pub fn released(&self) -> bool {
        let today = chrono::offset::Local::now().naive_local().date();
        match self.release_date() {
            Some(d) => d <= today,
            None => true,
        }
    }

    pub fn igdb_id(&self) -> Option<u32> {
        if let Self::IGDB(inner) = self {
            return Some(inner.id);
        }

        None
    }

    pub fn cover_url(&self) -> Option<String> {
        if let Self::IGDB(inner) = self {
            return inner.cover_url();
        }

        None
    }

    pub fn set_title(&mut self, title: &str) {
        match self {
            Self::Custom(inner) => inner.title = title.to_string(),
            Self::IGDB(inner) => inner.cached_title = title.to_string(),
        }
    }

    pub fn set_release_date(&mut self, date: chrono::naive::NaiveDate) {
        match self {
            Self::Custom(inner) => inner.release_date = Some(date),
            Self::IGDB(inner) => inner.cached_release_date = Some(date),
        }
    }

    pub fn set_release_date_str(&mut self, date_str: &str) -> Result<(), ()> {
        if let Ok(date) = chrono::naive::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            self.set_release_date(date);
            Ok(())
        }
        else {
            Err(())
        }
    }
}

impl SDatabase {
    pub fn new() -> Self {
        Self {
            games: Vec::new(),
            sessions: Vec::new(),
        }
    }
}


