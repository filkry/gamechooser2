use chrono;
use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SGameTags {
    pub couch_playable: Option<bool>,
    pub portable_playable: Option<bool>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SOwn {
    pub steam: bool,
    pub egs: bool,
    pub emulator: bool,

    pub ds: bool,
    pub n3ds: bool,
    pub wii: bool,
    pub wiiu: bool,
    pub switch: bool,

    pub ps4: bool,
    pub ps5: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SGame {
    internal_id: Option<u32>,
    title: String,
    release_date: Option<chrono::naive::NaiveDate>,
    igdb_id: Option<u32>,
    cover_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SGameCustomInfo {
    pub via: String,

    pub tags: SGameTags,
    pub own: SOwn,
}

pub struct SGameChooseState {
    next_valid_proposal_date: chrono::naive::NaiveDate,
    retired: bool,
    passes: u16,
    ignore_passes: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SCollectionGame {
    pub game: SGame,
    pub info: SGameCustomInfo,
}

impl SGame {
    pub fn new_igdb(title: String, release_date: Option<chrono::naive::NaiveDate>, igdb_id: u32, cover_url: Option<String>) -> Self {
        Self {
            internal_id: None,
            title,
            release_date,
            igdb_id: Some(igdb_id),
            cover_url,
        }
    }

    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    pub fn release_date(&self) -> Option<chrono::naive::NaiveDate> {
        self.release_date.clone()
    }

    pub fn igdb_id(&self) -> &Option<u32> {
        &self.igdb_id
    }

    pub fn cover_url(&self) -> Option<&str> {
        match &self.cover_url {
            Some(s) => Some(s.as_str()),
            None => None,
        }
    }

    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    pub fn set_release_date(&mut self, date: chrono::naive::NaiveDate) {
        self.release_date = Some(date);
    }

    pub fn set_release_date_str(&mut self, date_str: &str) -> Result<(), ()> {
        if let Ok(date) = chrono::naive::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            self.release_date = Some(date);
            Ok(())
        }
        else {
            Err(())
        }
    }
}

impl SGameCustomInfo {
    pub fn new() -> Self {
        Self {
            via: String::new(),
            tags: Default::default(),
            own: Default::default(),
        }
    }
}

impl SCollectionGame {
    pub fn new(game: SGame) -> Self {
        Self {
            game,
            info: SGameCustomInfo::new(),
        }
    }

    pub fn game_mut(&mut self) -> &mut SGame {
        &mut self.game
    }

    pub fn info_mut(&mut self) -> &mut SGameCustomInfo {
        &mut self.info
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
