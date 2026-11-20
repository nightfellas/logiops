pub mod device;
pub mod discovery;
pub mod error;
pub mod features;
pub mod report;
pub mod transport;

pub use device::HidppDevice;
pub use error::HidppError;
pub use features::{
    BatteryInfo, ChargingStatus, DeviceType, DpiSettings, SmartShiftMode, SmartShiftSettings,
};
pub use report::{HidppReport, REPORT_ID_LONG, REPORT_ID_SHORT, REPORT_ID_VERY_LONG};
pub use transport::{HidrawTransport, MockTransport, Transport};
