use thiserror::Error;

#[derive(Error, Debug)]
pub enum HidppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Transport error: {0}")]
    Transport(String),

    #[error("Device is offline or in deep RF sleep")]
    DeviceOffline,

    #[error("Device timed out waiting for report response")]
    Timeout,

    #[error("Invalid report length: expected at least {expected}, got {actual}")]
    InvalidReportLength { expected: usize, actual: usize },

    #[error("Invalid report ID: {0:#04x}")]
    InvalidReportId(u8),

    #[error("HID++ protocol error from device: code {code:#04x} on feature index {feature_index:#04x}, function {function_id:#04x}")]
    ProtocolError {
        code: u8,
        feature_index: u8,
        function_id: u8,
    },

    #[error("Feature not found on device: {0:#06x}")]
    FeatureNotFound(u16),

    #[error("Failed to parse device response: {0}")]
    ParseError(String),
}
