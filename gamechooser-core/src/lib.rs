use chrono;
use chrono::{Datelike};
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SGameInfo0 {
    pub title: String,
    pub release_date: Option<chrono::naive::NaiveDate>,
    pub igdb_id: Option<u32>,
    pub cover_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SGameInfoCustom {
    title: String,
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
    pub game_info: EGameInfo,
    pub custom_info: SGameCustomInfo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SCollectionGame0 {
    pub internal_id: u32, // $$$FRK(TODO): These internal IDs should have a type for type validation, but I'm lazy right now
    pub game_info: SGameInfo0,
    pub custom_info: SGameCustomInfo,
    pub choose_state: SGameChooseState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SCollectionGame {
    pub internal_id: u32, // $$$FRK(TODO): These internal IDs should have a type for type validation, but I'm lazy right now
    pub game_info: EGameInfo,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SSessionAndGameInfo {
    pub session: SSession,
    pub game_info: EGameInfo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SSessionFilter {
    pub active_only: bool,
    pub memorable_only: bool,
    pub year: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SRandomizerFilter {
    pub tags: SGameTagsFilter,
    pub allow_unowned: bool,
    pub max_passes: u16,
}

// -- $$$FRK(TODO): need to guarantee that internal_ids are always in order after loading from JSON, for this to be reliable
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SRandomizerList {
    pub games: Vec<SCollectionGame>,
    pub shuffled_indices: Vec<usize>,
}

// -- newest version always omits a version number to keep updating code simple
#[derive(Debug, Serialize, Deserialize)]
pub struct SDatabase0 {
    pub games: Vec<SCollectionGame0>,
    pub sessions: Vec<SSession>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SDatabase {
    pub games: Vec<SCollectionGame>,
    pub sessions: Vec<SSession>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EDatabase {
    V0(SDatabase0),
    V1(SDatabase),
}

impl std::ops::Deref for EDatabase {
    type Target = SDatabase;

    fn deref(&self) -> &Self::Target {
        #[allow(irrefutable_let_patterns)]
        if let Self::V1(inner) = self {
            return inner;
        }
        panic!("Trying to deref on database that is not of current version.");
    }
}

impl std::ops::DerefMut for EDatabase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[allow(irrefutable_let_patterns)]
        if let Self::V1(inner) = self {
            return inner;
        }
        panic!("Trying to deref on database that is not of current version.");
    }
}

impl SGameTags {
    pub fn each<F>(&self, mut f: F) where
        F: std::ops::FnMut(bool, &str)
    {
        f(self.couch_playable, "couch playable");
        f(self.portable_playable, "portable playable");
    }

    pub fn each_mut<F>(&mut self, mut f: F) where
        F: std::ops::FnMut(&mut bool, &str)
    {
        f(&mut self.couch_playable, "couch playable");
        f(&mut self.portable_playable, "portable playable");
    }
}

impl SOwn {
    fn owned(&self) -> bool {
        let mut owned = false;
        let check = |o: bool, _: &str| {
            owned = owned || o;
        };
        self.each(check);

        owned
    }

    pub fn each<F>(&self, mut f: F) where
        F: std::ops::FnMut(bool, &str)
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

    pub fn each_mut<F>(&mut self, mut f: F) where
        F: std::ops::FnMut(&mut bool, &str)
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

impl SGameChooseState {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn pass(&mut self) {
        self.passes = self.passes + 1;
        self.push();
    }

    pub fn push(&mut self) {
        let today = chrono::offset::Local::now().naive_local().date();
        let pass_count = std::cmp::max(1, self.passes);
        self.next_valid_proposal_date = today.checked_add_signed(chrono::Duration::days((pass_count * 45) as i64)).unwrap();
    }

    pub fn retire(&mut self) {
        self.retired = true;
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

impl SSessionFilter {
    pub fn session_passes(&self, session: &SSession) -> bool {
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

        result = result && (game.choose_state.ignore_passes || game.choose_state.passes <= self.max_passes);
        result = result && !game.choose_state.retired;
        result = result && game.choose_state.next_valid_proposal_date <= chrono::offset::Local::now().naive_local().date();

        result
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

impl EDatabase {
    pub fn new() -> Self {
        Self::V1(SDatabase::new())
    }

    pub fn to_latest_version(self) -> Self {
        match self {
            EDatabase::V0(_) => panic!("V0 cannot be automatically converted to V1, requires using importer application to query IGDB for missing info."),
            EDatabase::V1(_) => self
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
