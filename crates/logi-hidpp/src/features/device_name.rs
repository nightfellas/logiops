use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    Keyboard,
    RemoteControl,
    Numpad,
    Mouse,
    Touchpad,
    Trackball,
    Headset,
    Other,
}

impl From<u8> for DeviceType {
    fn from(val: u8) -> Self {
        match val {
            0 => DeviceType::Keyboard,
            1 => DeviceType::RemoteControl,
            2 => DeviceType::Numpad,
            3 => DeviceType::Mouse,
            4 => DeviceType::Touchpad,
            5 => DeviceType::Trackball,
            8 => DeviceType::Headset,
            _ => DeviceType::Other,
        }
    }
}
