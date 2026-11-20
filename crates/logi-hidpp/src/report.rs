use crate::error::HidppError;

pub const REPORT_ID_SHORT: u8 = 0x10;
pub const REPORT_ID_LONG: u8 = 0x11;
pub const REPORT_ID_VERY_LONG: u8 = 0x12;
pub const ERROR_FEATURE_INDEX: u8 = 0x8F;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportType {
    Short,
    Long,
    VeryLong,
}

impl ReportType {
    pub fn report_id(&self) -> u8 {
        match self {
            ReportType::Short => REPORT_ID_SHORT,
            ReportType::Long => REPORT_ID_LONG,
            ReportType::VeryLong => REPORT_ID_VERY_LONG,
        }
    }

    pub fn length(&self) -> usize {
        match self {
            ReportType::Short => 7,
            ReportType::Long => 20,
            ReportType::VeryLong => 64,
        }
    }
}

/// HID++ 2.0 / 4.0 packet structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HidppReport {
    pub report_id: u8,
    pub device_slot: u8,
    pub feature_index: u8,
    pub function_id: u8,
    pub software_id: u8,
    pub payload: Vec<u8>,
}

impl HidppReport {
    pub fn new_long(device_slot: u8, feature_index: u8, function_id: u8, software_id: u8) -> Self {
        Self {
            report_id: REPORT_ID_LONG,
            device_slot,
            feature_index,
            function_id,
            software_id,
            payload: vec![0u8; 16],
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let len = match self.report_id {
            REPORT_ID_SHORT => 7,
            REPORT_ID_VERY_LONG => 64,
            _ => 20,
        };
        let mut buffer = vec![0u8; len];
        buffer[0] = self.report_id;
        buffer[1] = self.device_slot;
        buffer[2] = self.feature_index;
        buffer[3] = (self.function_id << 4) | (self.software_id & 0x0F);

        let copy_len = self.payload.len().min(len.saturating_sub(4));
        buffer[4..4 + copy_len].copy_from_slice(&self.payload[..copy_len]);
        buffer
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, HidppError> {
        if bytes.len() < 7 {
            return Err(HidppError::InvalidReportLength {
                expected: 7,
                actual: bytes.len(),
            });
        }

        let report_id = bytes[0];
        let device_slot = bytes[1];
        let feature_index = bytes[2];

        // Check for HID++ 2.0 Error packet (Feature index 0x8F)
        if feature_index == ERROR_FEATURE_INDEX {
            let error_feature = bytes[3];
            let error_function = (bytes[4] >> 4) & 0x0F;
            let error_code = bytes[5];
            
            // Error code 0x04 indicates connect fail / device disconnected; 0x09 indicates sleep / resource error
            if error_code == 0x04 || error_code == 0x09 {
                return Err(HidppError::DeviceOffline);
            }

            return Err(HidppError::ProtocolError {
                code: error_code,
                feature_index: error_feature,
                function_id: error_function,
            });
        }

        let function_id = (bytes[3] >> 4) & 0x0F;
        let software_id = bytes[3] & 0x0F;
        let payload = bytes[4..].to_vec();

        Ok(Self {
            report_id,
            device_slot,
            feature_index,
            function_id,
            software_id,
            payload,
        })
    }
}
