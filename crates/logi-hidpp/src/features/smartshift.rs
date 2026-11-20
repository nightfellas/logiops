use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SmartShiftMode {
    AlwaysRatchet,
    AlwaysFreeSpin,
    AutoDisengage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SmartShiftSettings {
    pub enabled: bool,
    pub threshold: u8,
    pub mode: SmartShiftMode,
}

impl Default for SmartShiftSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold: 32,
            mode: SmartShiftMode::AutoDisengage,
        }
    }
}
