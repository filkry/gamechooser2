use chrono;
use serde::{Deserialize, Serialize};

/* How to version bump

1. Duplicate this file with new version suffix.
2. Change the pub imported module in lib.rs
3. Change any types in this file that require updating
4. write a from_vx function in SDatabase
5. in the previous version, replace all unchanged types (compiler will help find these when you
   copy them directly in from_vx) with types imported from the new version
*/

use crate::database_v2;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SGameTags {
    pub couch_playable: bool,
    pub portable_playable: bool,
    pub japanese_practice: bool,

    #[serde(default)]
    pub retro: bool,

    #[serde(default)]
    pub pick_up_and_play: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SOwn {
    pub free: bool,
    pub steam: bool,
    pub gmg: bool,
    pub gog: bool,
    pub humble: bool,
    pub origin: bool,
    pub egs: bool,
    pub battlenet: bool,
    pub itch: bool,
    pub standalone_launcher: bool,

    pub emulator: bool,

    pub gba: bool,
    pub ds: bool,
    pub n3ds: bool,
    pub gamecube: bool,
    pub wii: bool,
    pub wiiu: bool,
    pub switch: bool,

    pub ps1: bool,
    pub ps2: bool,
    pub ps3: bool,
    pub ps4: bool,
    pub ps5: bool,
    pub psp: bool,
    pub vita: bool,

    pub xbox: bool,

    pub ios: bool,

    pub oculus_quest: bool,

    pub ban_owned: bool,
}

impl SGameInfoIGDB {
    pub fn cover_url(&self) -> Option<String> {
        if let Some(cover_id) = &self.cached_cover_id {
            return Some(format!(
                "https://images.igdb.com/igdb/image/upload/t_cover_big/{}.jpg",
                cover_id
            ));
        }

        None
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EReleaseDate {
    UnknownUnreleased,
    UnknownReleased,
    Known(chrono::naive::NaiveDate),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SGameInfoCustom {
    title: String,
    release_date: EReleaseDate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SGameInfoIGDB {
    pub id: u32,
    pub slug: String,
    pub cached_title: String,
    pub cached_release_date: EReleaseDate,
    pub cached_cover_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EGameInfo {
    Custom(SGameInfoCustom),
    IGDB(SGameInfoIGDB),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EHowLongToBeat {
    Unknown,
    Manual(u16),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SGameCustomInfo {
    pub via: String,

    pub tags: SGameTags,
    pub own: SOwn,

    #[serde(default)]
    pub phantom_session: bool,

    #[serde(default)]
    pub archived: bool,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SGameChooseState {
    pub next_valid_proposal_date: chrono::naive::NaiveDate,
    pub retired: bool,
    pub passes: u16,
    pub ignore_passes: bool,

    #[serde(default)]
    pub pushes: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SCollectionGame {
    pub internal_id: u32, // $$$FRK(TODO): These internal IDs should have a type for type validation, but I'm lazy right now
    pub game_info: EGameInfo,

    #[serde(default)]
    pub how_long_to_beat: EHowLongToBeat,

    pub custom_info: SGameCustomInfo,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct SDatabase {
    pub games: Vec<SCollectionGame>,
    pub sessions: Vec<SSession>,
}

impl SGameTags {
    pub fn each<F>(&self, mut f: F)
    where
        F: std::ops::FnMut(bool, &str),
    {
        f(self.couch_playable, "couch");
        f(self.portable_playable, "portable");
        f(self.japanese_practice, "jp practice");
        f(self.retro, "retro");
        f(self.pick_up_and_play, "pick up and play");
    }

    pub fn each_mut<F>(&mut self, mut f: F)
    where
        F: std::ops::FnMut(&mut bool, &str),
    {
        f(&mut self.couch_playable, "couch");
        f(&mut self.portable_playable, "portable");
        f(&mut self.japanese_practice, "jp practice");
        f(&mut self.retro, "retro");
        f(&mut self.pick_up_and_play, "pick up and play");
    }
}

impl SOwn {
    pub fn owned(&self) -> bool {
        let mut owned = false;
        let check = |o: bool, _: &str| {
            owned = owned || o;
        };
        self.each(check);

        owned
    }

    pub fn each<F>(&self, mut f: F)
    where
        F: std::ops::FnMut(bool, &str),
    {
        f(self.free, "free");
        f(self.steam, "steam");
        f(self.gmg, "gmg");
        f(self.gog, "gog");
        f(self.humble, "humble");
        f(self.origin, "origin");
        f(self.egs, "egs");
        f(self.battlenet, "battle.net");
        f(self.itch, "itch.io");
        f(self.standalone_launcher, "standalone launcher");
        f(self.emulator, "emulator");
        f(self.gba, "gba");
        f(self.ds, "ds");
        f(self.n3ds, "3ds");
        f(self.gamecube, "gamecube");
        f(self.wii, "wii");
        f(self.wiiu, "wiiu");
        f(self.switch, "switch");
        f(self.ps1, "ps1");
        f(self.ps2, "ps2");
        f(self.ps3, "ps3");
        f(self.ps4, "ps4");
        f(self.ps5, "ps5");
        f(self.psp, "psp");
        f(self.vita, "vita");
        f(self.xbox, "xbox");
        f(self.ios, "ios");
        f(self.oculus_quest, "oculus quest");
        f(self.ban_owned, "ban owns");
    }

    pub fn each_mut<F>(&mut self, mut f: F)
    where
        F: std::ops::FnMut(&mut bool, &str),
    {
        f(&mut self.free, "free");
        f(&mut self.steam, "steam");
        f(&mut self.gmg, "gmg");
        f(&mut self.gog, "gog");
        f(&mut self.humble, "humble");
        f(&mut self.origin, "origin");
        f(&mut self.egs, "egs");
        f(&mut self.battlenet, "battle.net");
        f(&mut self.itch, "itch.io");
        f(&mut self.standalone_launcher, "standalone launcher");
        f(&mut self.emulator, "emulator");
        f(&mut self.gba, "gba");
        f(&mut self.ds, "ds");
        f(&mut self.n3ds, "3ds");
        f(&mut self.gamecube, "gamecube");
        f(&mut self.wii, "wii");
        f(&mut self.wiiu, "wiiu");
        f(&mut self.switch, "switch");
        f(&mut self.ps1, "ps1");
        f(&mut self.ps2, "ps2");
        f(&mut self.ps3, "ps3");
        f(&mut self.ps4, "ps4");
        f(&mut self.ps5, "ps5");
        f(&mut self.psp, "psp");
        f(&mut self.vita, "vita");
        f(&mut self.xbox, "xbox");
        f(&mut self.ios, "ios");
        f(&mut self.oculus_quest, "oculus quest");
        f(&mut self.ban_owned, "ban owns");
    }
}

impl EGameInfo {
    pub fn new_igdb(
        igdb_id: u32,
        slug: &str,
        cover_id: Option<String>,
        title: &str,
        release_date: EReleaseDate,
    ) -> Self {
        Self::IGDB(SGameInfoIGDB {
            id: igdb_id,
            slug: String::from(slug),
            cached_title: String::from(title),
            cached_cover_id: cover_id,
            cached_release_date: release_date,
        })
    }

    pub fn new_custom(title: String, release_date: EReleaseDate) -> Self {
        Self::Custom(SGameInfoCustom {
            title,
            release_date,
        })
    }

    pub fn title(&self) -> &str {
        match self {
            Self::IGDB(inner) => inner.cached_title.as_str(),
            Self::Custom(inner) => inner.title.as_str(),
        }
    }

    pub fn release_date(&self) -> EReleaseDate {
        match self {
            Self::IGDB(inner) => inner.cached_release_date.clone(),
            Self::Custom(inner) => inner.release_date.clone(),
        }
    }

    pub fn released(&self) -> bool {
        let today = chrono::offset::Local::now().naive_local().date();
        match self.release_date() {
            EReleaseDate::UnknownUnreleased => false,
            EReleaseDate::UnknownReleased => true,
            EReleaseDate::Known(d) => d <= today,
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

    pub fn set_release_date(&mut self, date: EReleaseDate) {
        match self {
            Self::Custom(inner) => inner.release_date = date,
            Self::IGDB(inner) => inner.cached_release_date = date,
        }
    }

    pub fn set_release_date_known(&mut self, date: chrono::naive::NaiveDate) {
        match self {
            Self::Custom(inner) => inner.release_date = EReleaseDate::Known(date),
            Self::IGDB(inner) => inner.cached_release_date = EReleaseDate::Known(date),
        }
    }

    pub fn set_release_date_known_str(&mut self, date_str: &str) -> Result<(), ()> {
        if let Ok(date) = chrono::naive::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            self.set_release_date_known(date);
            Ok(())
        } else {
            Err(())
        }
    }
}

impl EHowLongToBeat {
    pub fn hours_to_beat(&self) -> Option<u16> {
        match self {
            Self::Unknown => None,
            Self::Manual(hours) => Some(hours.clone()),
        }
    }
}


impl Default for EHowLongToBeat {
    fn default() -> Self {
        EHowLongToBeat::Unknown
    }
}

impl SGameCustomInfo {
    pub fn new() -> Self {
        Self {
            via: String::new(),
            tags: Default::default(),
            own: Default::default(),
            archived: false,
            phantom_session: false,
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
            pushes: 0,
        }
    }
}

impl SGameChooseState {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn pass(&mut self) {
        self.passes = self.passes + 1;
        self.update_next_valid_date();
    }

    pub fn push(&mut self) {
        self.pushes = self.pushes + 1;
        self.update_next_valid_date();
    }

    pub fn update_next_valid_date(&mut self) {
        let today = chrono::offset::Local::now().naive_local().date();
        let delay_count = self.passes
            + std::cmp::min(self.pushes, 8) // max 1 year from pushes
            + 1; // always at least one so we can't get 0 delay
        let pass_days = delay_count * 30;
        self.next_valid_proposal_date = today
            .checked_add_signed(chrono::Duration::days(pass_days as i64))
            .unwrap();
    }

    pub fn retire(&mut self) {
        self.retired = true;
    }

    pub fn set_ignore_passes(&mut self) {
        self.ignore_passes = true;
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

impl SDatabase {
    pub fn new() -> Self {
        Self {
            games: Vec::new(),
            sessions: Vec::new(),
        }
    }

    pub fn from_v2(v2: database_v2::SDatabase) -> Self {
        let mut new_games = Vec::with_capacity(v2.games.len());
        for game in v2.games {
            let new_date = match game.game_info.release_date() {
                None => EReleaseDate::UnknownUnreleased,
                Some(d) => EReleaseDate::Known(d),
            };

            let new_game_info = match game.game_info {
                database_v2::EGameInfo::Custom(c) => EGameInfo::Custom(SGameInfoCustom {
                    title: c.title,
                    release_date: new_date,
                }),
                database_v2::EGameInfo::IGDB(igdb) => EGameInfo::IGDB(SGameInfoIGDB {
                    id: igdb.id,
                    slug: igdb.slug,
                    cached_title: igdb.cached_title,
                    cached_release_date: new_date,
                    cached_cover_id: igdb.cached_cover_id,
                }),
            };

            new_games.push(SCollectionGame {
                internal_id: game.internal_id,
                how_long_to_beat: EHowLongToBeat::Unknown,
                game_info: new_game_info,
                custom_info: game.custom_info,
                choose_state: game.choose_state,
            });
        }

        Self {
            games: new_games,
            sessions: v2.sessions,
        }
    }
}
