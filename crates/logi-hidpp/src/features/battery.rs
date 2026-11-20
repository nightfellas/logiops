use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChargingStatus {
    Discharging,
    Charging,
    Full,
    Standby,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatteryInfo {
    pub percentage: u8,
    pub charging: bool,
    pub status: ChargingStatus,
    pub critical: bool,
    pub standby: bool,
}

impl BatteryInfo {
    pub fn new_active(percentage: u8, charging: bool, status: ChargingStatus, critical: bool) -> Self {
        Self {
            percentage,
            charging,
            status,
            critical,
            standby: false,
        }
    }

    pub fn to_standby(&self) -> Self {
        Self {
            percentage: self.percentage,
            charging: false,
            status: ChargingStatus::Standby,
            critical: self.critical,
            standby: true,
        }
    }

    /// Parse Unified Battery 0x1004 Function 0/1 response payload.
    /// Payload byte 0: Percentage (0..100)
    /// Payload byte 1: Charging status
    /// Payload byte 2: External power status / flags
    pub fn from_unified_battery_payload(payload: &[u8]) -> Self {
        let percentage = payload.first().copied().unwrap_or(0).min(100);
        let charging_byte = payload.get(1).copied().unwrap_or(0);
        let critical_byte = payload.get(2).copied().unwrap_or(0);

        let (charging, status) = match charging_byte {
            0x00 => (false, ChargingStatus::Discharging),
            0x01 | 0x02 => (true, ChargingStatus::Charging),
            0x03 => (false, ChargingStatus::Full),
            _ => (false, ChargingStatus::Unknown),
        };

        let critical = (critical_byte & 0x01) != 0 || percentage <= 10;

        Self {
            percentage,
            charging,
            status,
            critical,
            standby: false,
        }
    }

    /// Parse Legacy Battery 0x1000 payload.
    pub fn from_legacy_battery_payload(payload: &[u8]) -> Self {
        let percentage = payload.first().copied().unwrap_or(0).min(100);
        let status_byte = payload.get(2).copied().unwrap_or(0);
        let charging = (status_byte & 0x80) != 0;

        let status = if charging {
            ChargingStatus::Charging
        } else {
            ChargingStatus::Discharging
        };

        let critical = percentage <= 10;

        Self {
            percentage,
            charging,
            status,
            critical,
            standby: false,
        }
    }
}
