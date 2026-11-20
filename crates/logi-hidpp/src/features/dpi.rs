use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DpiSettings {
    pub current: u16,
    pub min: u16,
    pub max: u16,
    pub step: u16,
}

impl Default for DpiSettings {
    fn default() -> Self {
        Self {
            current: 1600,
            min: 200,
            max: 8000,
            step: 50,
        }
    }
}
