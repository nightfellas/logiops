use crate::error::HidppError;
use crate::features::{
    BatteryInfo, DeviceType, DpiSettings, SmartShiftMode, SmartShiftSettings,
    CID_GESTURE_BUTTON, CID_MODE_SHIFT, FEATURE_ADJUSTABLE_DPI, FEATURE_BATTERY_STATUS,
    FEATURE_DEVICE_NAME_TYPE, FEATURE_REPROG_CONTROLS_V4, FEATURE_SMART_SHIFT,
    FEATURE_UNIFIED_BATTERY,
};
use crate::report::HidppReport;
use crate::transport::Transport;
use std::collections::HashMap;

pub struct HidppDevice {
    pub slot: u8,
    pub name: Option<String>,
    pub device_type: Option<DeviceType>,
    pub last_battery: Option<BatteryInfo>,
    feature_cache: HashMap<u16, u8>,
    software_id: u8,
}

impl HidppDevice {
    pub fn new(slot: u8) -> Self {
        Self {
            slot,
            name: None,
            device_type: None,
            last_battery: None,
            feature_cache: HashMap::new(),
            software_id: 0x01,
        }
    }

    /// Resolves feature index dynamically by querying the Root feature (0x0000).
    pub fn resolve_feature<T: Transport>(
        &mut self,
        transport: &mut T,
        feature_id: u16,
    ) -> Result<u8, HidppError> {
        if let Some(&index) = self.feature_cache.get(&feature_id) {
            return Ok(index);
        }

        // Query Root Feature 0x0000 Function 0: getFeature(featureId)
        let mut report = HidppReport::new_long(self.slot, 0x00, 0x00, self.software_id);
        report.payload[0] = (feature_id >> 8) as u8;
        report.payload[1] = (feature_id & 0xFF) as u8;

        transport.write_report(&report.to_bytes())?;
        let resp_bytes = transport.read_report(1000)?;
        let parsed = HidppReport::parse(&resp_bytes)?;

        let feature_index = parsed.payload.first().copied().unwrap_or(0);
        if feature_index == 0 {
            return Err(HidppError::FeatureNotFound(feature_id));
        }

        self.feature_cache.insert(feature_id, feature_index);
        Ok(feature_index)
    }

    /// Reads model name from device (Feature 0x0005).
    pub fn query_name<T: Transport>(&mut self, transport: &mut T) -> Result<String, HidppError> {
        let feature_index = self.resolve_feature(transport, FEATURE_DEVICE_NAME_TYPE)?;

        // Function 0: getDeviceNameCount
        let report = HidppReport::new_long(self.slot, feature_index, 0x00, self.software_id);
        transport.write_report(&report.to_bytes())?;
        let resp_bytes = transport.read_report(1000)?;
        let parsed = HidppReport::parse(&resp_bytes)?;
        let name_len = parsed.payload.first().copied().unwrap_or(0) as usize;

        if name_len == 0 {
            return Ok("Unknown Logitech Device".to_string());
        }

        let mut name_bytes = Vec::new();
        let mut char_index = 0u8;

        while name_bytes.len() < name_len {
            // Function 1: getDeviceName(charIndex)
            let mut chunk_report = HidppReport::new_long(self.slot, feature_index, 0x01, self.software_id);
            chunk_report.payload[0] = char_index;
            transport.write_report(&chunk_report.to_bytes())?;
            let chunk_resp = transport.read_report(1000)?;
            let parsed_chunk = HidppReport::parse(&chunk_resp)?;

            let bytes_to_take = (name_len - name_bytes.len()).min(16);
            name_bytes.extend_from_slice(&parsed_chunk.payload[..bytes_to_take]);
            char_index += 16;
        }

        let name = String::from_utf8_lossy(&name_bytes).trim().to_string();
        self.name = Some(name.clone());
        Ok(name)
    }

    /// Reads device type (Mouse, Keyboard, etc.) from Feature 0x0005.
    pub fn query_device_type<T: Transport>(&mut self, transport: &mut T) -> Result<DeviceType, HidppError> {
        let feature_index = self.resolve_feature(transport, FEATURE_DEVICE_NAME_TYPE)?;

        // Function 2: getDeviceType
        let report = HidppReport::new_long(self.slot, feature_index, 0x02, self.software_id);
        transport.write_report(&report.to_bytes())?;
        let resp_bytes = transport.read_report(1000)?;
        let parsed = HidppReport::parse(&resp_bytes)?;

        let type_byte = parsed.payload.first().copied().unwrap_or(255);
        let dev_type = DeviceType::from(type_byte);
        self.device_type = Some(dev_type);
        Ok(dev_type)
    }

    /// Reads battery status, supporting Smart Sleep & Idle detection.
    pub fn query_battery<T: Transport>(&mut self, transport: &mut T) -> Result<BatteryInfo, HidppError> {
        // Try Unified Battery (0x1004) first
        let batt_res = self.resolve_feature(transport, FEATURE_UNIFIED_BATTERY);

        let info = match batt_res {
            Ok(feature_index) => {
                // Function 1: getBatteryStatus
                let report = HidppReport::new_long(self.slot, feature_index, 0x01, self.software_id);
                if let Err(e) = transport.write_report(&report.to_bytes()) {
                    return self.handle_battery_error(e);
                }

                match transport.read_report(1000) {
                    Ok(resp_bytes) => match HidppReport::parse(&resp_bytes) {
                        Ok(parsed) => {
                            let info = BatteryInfo::from_unified_battery_payload(&parsed.payload);
                            self.last_battery = Some(info.clone());
                            Ok(info)
                        }
                        Err(e) => self.handle_battery_error(e),
                    },
                    Err(e) => self.handle_battery_error(e),
                }
            }
            Err(_) => {
                // Fallback to legacy battery (0x1000)
                let legacy_res = self.resolve_feature(transport, FEATURE_BATTERY_STATUS)?;
                let report = HidppReport::new_long(self.slot, legacy_res, 0x00, self.software_id);
                if let Err(e) = transport.write_report(&report.to_bytes()) {
                    return self.handle_battery_error(e);
                }

                match transport.read_report(1000) {
                    Ok(resp_bytes) => match HidppReport::parse(&resp_bytes) {
                        Ok(parsed) => {
                            let info = BatteryInfo::from_legacy_battery_payload(&parsed.payload);
                            self.last_battery = Some(info.clone());
                            Ok(info)
                        }
                        Err(e) => self.handle_battery_error(e),
                    },
                    Err(e) => self.handle_battery_error(e),
                }
            }
        };

        info
    }

    fn handle_battery_error(&mut self, err: HidppError) -> Result<BatteryInfo, HidppError> {
        match err {
            HidppError::DeviceOffline | HidppError::Timeout => {
                if let Some(ref last) = self.last_battery {
                    let standby = last.to_standby();
                    self.last_battery = Some(standby.clone());
                    Ok(standby)
                } else {
                    Err(err)
                }
            }
            other => Err(other),
        }
    }

    /// Queries DPI sensor settings (Feature 0x2201).
    pub fn query_dpi<T: Transport>(&mut self, transport: &mut T) -> Result<DpiSettings, HidppError> {
        let feature_index = self.resolve_feature(transport, FEATURE_ADJUSTABLE_DPI)?;

        // Function 2: getSensorDpi
        let report = HidppReport::new_long(self.slot, feature_index, 0x02, self.software_id);
        transport.write_report(&report.to_bytes())?;
        let resp_bytes = transport.read_report(1000)?;
        let parsed = HidppReport::parse(&resp_bytes)?;

        let current_dpi = u16::from_be_bytes([parsed.payload[1], parsed.payload[2]]);

        Ok(DpiSettings {
            current: current_dpi,
            min: 200,
            max: 8000,
            step: 50,
        })
    }

    /// Sets DPI resolution on sensor (Feature 0x2201).
    pub fn set_dpi<T: Transport>(&mut self, transport: &mut T, dpi: u16) -> Result<(), HidppError> {
        let feature_index = self.resolve_feature(transport, FEATURE_ADJUSTABLE_DPI)?;

        // Function 3: setSensorDpi(sensorIndex = 0, dpi)
        let mut report = HidppReport::new_long(self.slot, feature_index, 0x03, self.software_id);
        report.payload[0] = 0x00; // Sensor index 0
        report.payload[1] = (dpi >> 8) as u8;
        report.payload[2] = (dpi & 0xFF) as u8;

        transport.write_report(&report.to_bytes())?;
        let resp_bytes = transport.read_report(1000)?;
        let _ = HidppReport::parse(&resp_bytes)?;
        Ok(())
    }

    /// Queries SmartShift settings (Feature 0x2110).
    pub fn query_smartshift<T: Transport>(
        &mut self,
        transport: &mut T,
    ) -> Result<SmartShiftSettings, HidppError> {
        let feature_index = self.resolve_feature(transport, FEATURE_SMART_SHIFT)?;

        // Function 0: getSmartShift
        let report = HidppReport::new_long(self.slot, feature_index, 0x00, self.software_id);
        transport.write_report(&report.to_bytes())?;
        let resp_bytes = transport.read_report(1000)?;
        let parsed = HidppReport::parse(&resp_bytes)?;

        let mode_byte = parsed.payload[0];
        let threshold = parsed.payload[1];

        let mode = match mode_byte {
            1 => SmartShiftMode::AlwaysRatchet,
            2 => SmartShiftMode::AlwaysFreeSpin,
            _ => SmartShiftMode::AutoDisengage,
        };

        Ok(SmartShiftSettings {
            enabled: mode != SmartShiftMode::AlwaysFreeSpin,
            threshold,
            mode,
        })
    }

    /// Sets SmartShift ratchet threshold torque (Feature 0x2110).
    pub fn set_smartshift<T: Transport>(
        &mut self,
        transport: &mut T,
        settings: &SmartShiftSettings,
    ) -> Result<(), HidppError> {
        let feature_index = self.resolve_feature(transport, FEATURE_SMART_SHIFT)?;

        // Function 1: setSmartShift(mode, threshold)
        let mut report = HidppReport::new_long(self.slot, feature_index, 0x01, self.software_id);
        report.payload[0] = match settings.mode {
            SmartShiftMode::AlwaysRatchet => 1,
            SmartShiftMode::AlwaysFreeSpin => 2,
            SmartShiftMode::AutoDisengage => 0,
        };
        report.payload[1] = settings.threshold;

        transport.write_report(&report.to_bytes())?;
        let resp_bytes = transport.read_report(1000)?;
        let _ = HidppReport::parse(&resp_bytes)?;
        Ok(())
    }

    /// Diverts physical buttons (Gesture button 0x00C3, Mode shift 0x00C4) so hardware reports HID++ events.
    pub fn divert_reprogrammable_controls<T: Transport>(&mut self, transport: &mut T) -> Result<u8, HidppError> {
        let feature_index = self.resolve_feature(transport, FEATURE_REPROG_CONTROLS_V4)?;

        for cid in [CID_GESTURE_BUTTON, CID_MODE_SHIFT] {
            // Function 3: setCidReporting(cid, flags=0x03 [divert + divert_valid], remap=0x00)
            let mut report = HidppReport::new_long(self.slot, feature_index, 0x03, self.software_id);
            report.payload[0] = (cid >> 8) as u8;
            report.payload[1] = (cid & 0xFF) as u8;
            report.payload[2] = 0x03; // Divert flag (0x01 | 0x02 valid bit)
            report.payload[3] = 0x00;
            report.payload[4] = 0x00;
            let _ = transport.write_report(&report.to_bytes());
            let _ = transport.read_report(200);
        }

        Ok(feature_index)
    }
}
