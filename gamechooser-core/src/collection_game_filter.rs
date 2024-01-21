use serde::{Serialize, Deserialize};

use crate::{EHowLongToBeat, SCollectionGame, SConfig, SGameTags};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SGameTagsFilter {
    pub couch_playable: Option<bool>,
    pub portable_playable: Option<bool>,
    pub japanese_practice: Option<bool>,
    pub retro: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SCollectionGameFilter {
    pub tags: SGameTagsFilter,

    pub require_released: Option<bool>,
    pub required_alive_state: Option<bool>,
    pub require_is_after_valid_date: bool,

    pub required_ownership_state: Option<bool>,
    pub require_zero_sessions: bool,

    pub max_hltb_hours: Option<u16>,
}

// opinionated defaults based on my usual interests
impl Default for SGameTagsFilter {
    fn default() -> Self {
        Self{
            couch_playable: None,
            portable_playable: None,
            japanese_practice: Some(false),
            retro: Some(false),
        }
    }
}

impl Default for SCollectionGameFilter {
    fn default() -> Self {
        Self {
            tags: SGameTagsFilter::default(),
            require_released: Some(true),
            required_alive_state: Some(true),
            require_is_after_valid_date: true,
            required_ownership_state: None,
            require_zero_sessions: true,
            max_hltb_hours: None,
        }
    }
}

impl SGameTagsFilter {

    pub fn tags_pass(&self, game_tags: &SGameTags) -> bool {
        let mut result = true;

        if let Some(couch) = self.couch_playable {
            result = result && couch == game_tags.couch_playable;
        }
        if let Some(portable) = self.portable_playable {
            result = result && portable == game_tags.portable_playable;
        }
        if let Some(jp) = self.japanese_practice {
            result = result && jp == game_tags.japanese_practice;
        }
        if let Some(retro) = self.retro {
            result = result && retro == game_tags.retro;
        }

        return result;
    }
}

impl SCollectionGameFilter {
    // $$$FRK(TODO): having to do the has_any_sessions check outside is kinda busto
    pub fn game_passes(&self, cfg: &SConfig, game: &SCollectionGame, has_any_sessions: bool) -> bool {
        let mut result = true;

        // test tags
        result = result && self.tags.tags_pass(&game.custom_info.tags);

        if let Some(req_released) = self.require_released {
            result = result && req_released == game.game_info.released();
        }

        // test alive state
        if let Some(req_alive) = self.required_alive_state {
            let mut is_alive = true;
            is_alive = is_alive && (game.choose_state.ignore_passes || game.choose_state.passes <= cfg.live_max_passes);
            is_alive = is_alive && !game.choose_state.retired;

            result = result && req_alive == is_alive;
        }

        // test after valid date
        if self.require_is_after_valid_date {
            let today = chrono::offset::Local::now().naive_local().date();
            result = result && game.choose_state.next_valid_proposal_date <= today;
        }

        // test ownership state
        if let Some(owned) = self.required_ownership_state {
            result = result && owned == game.custom_info.own.owned();
        }

        // test has any sessions
        // NOTE/HACK: some games I've played before were added but have no sessions, usually they
        // have the ignore_passes flag so we use that one too
        if self.require_zero_sessions {
            let has_any_sessions_proxy = has_any_sessions || game.choose_state.ignore_passes;
            result = result && !has_any_sessions_proxy;
        }

        // test max time to beat
        if let Some(max_hours) = self.max_hltb_hours {
            result = result && match game.how_long_to_beat {
                EHowLongToBeat::Unknown => false,
                EHowLongToBeat::Manual(game_hours) => game_hours <= max_hours,
            };
        }

        result
    }
}
