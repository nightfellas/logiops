pub mod battery;
pub mod device_name;
pub mod dpi;
pub mod smartshift;

pub use battery::{BatteryInfo, ChargingStatus};
pub use device_name::DeviceType;
pub use dpi::DpiSettings;
pub use smartshift::{SmartShiftMode, SmartShiftSettings};

// Standard Logitech HID++ 2.0 / 4.0 Feature Codes
pub const FEATURE_ROOT: u16 = 0x0000;
pub const FEATURE_FEATURE_SET: u16 = 0x0001;
pub const FEATURE_DEVICE_NAME_TYPE: u16 = 0x0005;
pub const FEATURE_BATTERY_STATUS: u16 = 0x1000;
pub const FEATURE_UNIFIED_BATTERY: u16 = 0x1004;
pub const FEATURE_SMART_SHIFT: u16 = 0x2110;
pub const FEATURE_HIRES_SCROLLING: u16 = 0x2120;
pub const FEATURE_REPROG_CONTROLS_V4: u16 = 0x1B04;
pub const FEATURE_THUMB_WHEEL: u16 = 0x2150;
pub const FEATURE_ADJUSTABLE_DPI: u16 = 0x2201;

pub const CID_GESTURE_BUTTON: u16 = 0x00C3;
pub const CID_MODE_SHIFT: u16 = 0x00C4;
