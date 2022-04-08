use chrono;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SGameTags {
    couch_playable: Option<bool>,
    portable_playable: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SOwn {
    steam: bool,
    egs: bool,
    emulator: bool,

    ds: bool,
    n3ds: bool,
    wii: bool,
    wiiu: bool,
    switch: bool,

    ps4: bool,
    ps5: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SGame {
    internal_id: Option<u64>,
    title: String,
    release_date: Option<chrono::naive::NaiveDate>,
    igdb_id: Option<u64>,
    cover_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SCollectionGame {
    game: SGame,

    via: String,

    tags: SGameTags,
    own: SOwn,

    next_valid_proposal_date: chrono::naive::NaiveDate,
    retired: bool,
    passes: u16,
    ignore_passes: bool,
}

impl SGame {
    pub fn new_igdb(title: String, release_date: Option<chrono::naive::NaiveDate>, igdb_id: u64, cover_url: Option<String>) -> Self {
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

    pub fn cover_url(&self) -> Option<&str> {
        match &self.cover_url {
            Some(s) => Some(s.as_str()),
            None => None,
        }
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
