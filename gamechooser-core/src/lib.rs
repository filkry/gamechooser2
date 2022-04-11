use chrono;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SGameTags {
    pub couch_playable: bool,
    pub portable_playable: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SGameTagsFilter {
    pub couch_playable: Option<bool>,
    pub portable_playable: Option<bool>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
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
pub struct SGameInfo {
    title: String,
    release_date: Option<chrono::naive::NaiveDate>,
    igdb_id: Option<u32>,
    cover_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SGameCustomInfo {
    pub via: String,

    pub tags: SGameTags,
    pub own: SOwn,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SGameChooseState {
    pub next_valid_proposal_date: chrono::naive::NaiveDate,
    pub retired: bool,
    pub passes: u16,
    pub ignore_passes: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SAddCollectionGame {
    pub game_info: SGameInfo,
    pub custom_info: SGameCustomInfo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SCollectionGame {
    pub internal_id: u32, // $$$FRK(TODO): These internal IDs should have a type for type validation, but I'm lazy right now
    pub game_info: SGameInfo,
    pub custom_info: SGameCustomInfo,

    #[serde(default)]
    pub choose_state: SGameChooseState,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ESessionState {
    Ongoing,
    Finished {
        end_date: chrono::naive::NaiveDate,
        memorable: bool,
    },
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SSession {
    pub internal_id: u32,
    pub game_internal_id: u32,
    pub start_date: chrono::naive::NaiveDate,
    pub state: ESessionState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SSessionAndGameInfo {
    pub session: SSession,
    pub game_info: SGameInfo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SRandomizerFilter {
    pub tags: SGameTagsFilter,
    pub allow_unowned: bool,
    pub max_passes: u16,
}

impl SOwn {
    fn owned(&self) -> bool {
        self.steam || self.egs || self.emulator || self.ds || self.n3ds || self.wii || self.wiiu || self.switch || self.ps4 || self.ps5
    }
}

impl SGameInfo {
    pub fn new_igdb(title: String, release_date: Option<chrono::naive::NaiveDate>, igdb_id: u32, cover_url: Option<String>) -> Self {
        Self {
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

impl SAddCollectionGame {
    pub fn new(game_info: SGameInfo) -> Self {
        Self {
            game_info,
            custom_info: SGameCustomInfo::new(),
        }
    }
}

impl SCollectionGame {
    pub fn new(add: SAddCollectionGame, id: u32) -> Self {
        Self {
            internal_id: id,
            game_info: add.game_info,
            custom_info: add.custom_info,
            choose_state: Default::default(),
        }
    }
}

impl Default for SGameChooseState {
    fn default() -> Self {
        Self {
            next_valid_proposal_date: chrono::offset::Local::now().naive_local().date(),
            retired: false,
            passes: 0,
            ignore_passes: false,
        }
    }
}

impl SSession {
    pub fn new(id: u32, game_internal_id: u32) -> Self {
        Self {
            internal_id: id,
            game_internal_id,
            start_date: chrono::offset::Local::now().naive_local().date(),
            state: ESessionState::Ongoing,
        }
    }

    pub fn finish(&mut self, memorable: bool) {
        self.state = ESessionState::Finished {
            end_date: chrono::offset::Local::now().naive_local().date(),
            memorable,
        }
    }
}

impl SRandomizerFilter {
    pub fn game_passes(&self, game: &SCollectionGame) -> bool {
        let mut result = true;

        if let Some(couch) = self.tags.couch_playable {
            result = result && couch == game.custom_info.tags.couch_playable;
        }
        if let Some(portable) = self.tags.portable_playable {
            result = result && portable == game.custom_info.tags.portable_playable;
        }

        result = result && (self.allow_unowned || game.custom_info.own.owned());
        result = result && (game.choose_state.passes <= self.max_passes);

        result
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
