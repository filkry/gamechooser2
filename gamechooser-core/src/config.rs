use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SConfig {
    pub live_max_passes: u16,
}

impl Default for SConfig {
    fn default() -> Self {
        Self{
            live_max_passes: 2,
        }
    }
}