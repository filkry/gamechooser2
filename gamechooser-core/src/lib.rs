use chrono;
use chrono::{Datelike};
use serde::{Serialize, Deserialize};

mod database_v2;
mod database_v3;

// -- latest database version is exported via pub
pub use database_v3::*;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SGameTagsFilter {
    pub couch_playable: Option<bool>,
    pub portable_playable: Option<bool>,
    pub japanese_practice: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SSearchIGDBResult {
    pub game_info: EGameInfo,
    pub in_collection: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SAddCollectionGame {
    pub game_info: EGameInfo,
    pub custom_info: SGameCustomInfo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SSessionAndCollectionGame {
    pub session: SSession,
    pub collection_game: SCollectionGame,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SSessionFilter {
    pub game_id: Option<u32>,
    pub active_only: bool,
    pub memorable_only: bool,
    pub year: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SRandomizerFilter {
    pub tags: SGameTagsFilter,
    pub allow_unowned: bool,
    pub only_firsts: bool,
    pub max_passes: u16,
}

// -- $$$FRK(TODO): need to guarantee that internal_ids are always in order after loading from JSON, for this to be reliable
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SRandomizerList {
    pub games: Vec<SCollectionGame>,
    pub shuffled_indices: Vec<usize>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SSimpleStats {
    pub total_collection_size: u32,

    pub collection_released: u32,
    pub collection_selectable: u32,
    pub collection_retired: u32,
    pub collection_passed_many_times: u32,
    pub collection_cooldown: u32,

    pub collection_owned: u32,
    pub collection_played_before: u32,
    pub collection_couch_playable_tag: u32,
    pub collection_japanese_practice_tag: u32,
    pub collection_portable_playable_tag: u32,

    pub selectable_owned: u32,
    pub selectable_played_before: u32,
    pub selectable_couch_playable_tag: u32,
    pub selectable_japanese_practice_tag: u32,
    pub selectable_portable_playable_tag: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EDatabase {
    V2(database_v2::SDatabase),
    V3(SDatabase),
}

impl std::ops::Deref for EDatabase {
    type Target = SDatabase;

    fn deref(&self) -> &Self::Target {
        #[allow(irrefutable_let_patterns)]
        if let Self::V3(inner) = self {
            return inner;
        }
        panic!("Trying to deref on database that is not of current version.");
    }
}

impl std::ops::DerefMut for EDatabase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[allow(irrefutable_let_patterns)]
        if let Self::V3(inner) = self {
            return inner;
        }
        panic!("Trying to deref on database that is not of current version.");
    }
}

impl SAddCollectionGame {
    pub fn new(game_info: EGameInfo) -> Self {
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

impl SSessionFilter {
    pub fn session_passes(&self, session: &SSession) -> bool {
        if let Some(game_id_inner) = self.game_id {
            if session.game_internal_id != game_id_inner {
                return false;
            }
        }

        if self.active_only && !matches!(session.state, ESessionState::Ongoing) {
            return false;
        }

        if self.memorable_only {
            match session.state {
                ESessionState::Ongoing => {
                    return false;
                },
                ESessionState::Finished{end_date: _, memorable} => {
                    if !memorable {
                        return false;
                    }
                }
            }
        }

        if let Some(y) = self.year {
            let mut either = false;

            if session.start_date.year() as u32 == y {
                either = true;
            }

            if let ESessionState::Finished{end_date, memorable: _} = session.state {
                if end_date.year() as u32 == y {
                    either = true;
                }
            }

            if !either {
                return false;
            }
        }

        return true;
    }
}

impl Default for SRandomizerFilter {
    fn default() -> Self {
        Self {
            tags: SGameTagsFilter::default(),
            allow_unowned: true,
            only_firsts: true,
            max_passes: Self::max_passes(),
        }
    }
}

impl SRandomizerFilter {
    pub fn max_passes() -> u16 {
        2
    }

    // -- $$$FRK(TODO): having to do the has_any_sessions check outside is kinda busto
    pub fn game_passes(&self, game: &SCollectionGame, has_any_sessions: bool) -> bool {
        let mut result = true;

        let today = chrono::offset::Local::now().naive_local().date();

        result = result && game.game_info.released();

        if let Some(couch) = self.tags.couch_playable {
            result = result && couch == game.custom_info.tags.couch_playable;
        }
        if let Some(portable) = self.tags.portable_playable {
            result = result && portable == game.custom_info.tags.portable_playable;
        }
        if let Some(jp) = self.tags.japanese_practice {
            result = result && jp == game.custom_info.tags.japanese_practice;
        }

        result = result && (self.allow_unowned || game.custom_info.own.owned());
        // -- some games I've played before were added but have no sessions, usually they have the
        // -- ignore_passes flag so we use that one too
        let has_any_sessions_proxy = has_any_sessions || game.choose_state.ignore_passes;
        result = result && !(self.only_firsts && has_any_sessions_proxy);

        result = result && (game.choose_state.ignore_passes || game.choose_state.passes <= self.max_passes);
        result = result && !game.choose_state.retired;
        result = result && game.choose_state.next_valid_proposal_date <= today;

        result
    }
}

impl SGameChooseState {
    // -- returns whether the game could ever conceivably be selectable
    pub fn alive(&self) -> bool {
        !self.retired
            && (self.ignore_passes || self.passes <= SRandomizerFilter::max_passes())
    }
}

impl EDatabase {
    pub fn new() -> Self {
        Self::V3(SDatabase::new())
    }

    pub fn to_latest_version(self) -> Self {
        match self {
            EDatabase::V2(v2) => EDatabase::V3(SDatabase::from_v2(v2)),
            EDatabase::V3(_) => self,
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
