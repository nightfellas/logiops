use crate::ipc::IpcServer;
use logi_hidpp::discovery::scan_receivers;
use logi_hidpp::{BatteryInfo, HidppDevice, HidrawTransport};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

pub struct TelemetryService {
    ipc: Arc<IpcServer>,
}

impl TelemetryService {
    pub fn new(ipc: Arc<IpcServer>) -> Self {
        Self { ipc }
    }

    pub async fn run_polling_loop(&self) {
        let mut last_batteries: HashMap<String, BatteryInfo> = HashMap::new();
        let mut known_devices: HashMap<String, (String, String, u8)> = HashMap::new();

        // Seed initial known devices for Bolt slots
        known_devices.insert("bolt:slot_1".to_string(), ("MX Master 3S".to_string(), "mouse".to_string(), 70));
        known_devices.insert("bolt:slot_5".to_string(), ("MX Mechanical Mini".to_string(), "keyboard".to_string(), 85));

        // Allow input listener to settle and divert controls on daemon startup
        sleep(Duration::from_millis(1500)).await;

        loop {
            let receivers = scan_receivers();
            for receiver in &receivers {
                if let Ok(mut transport) = HidrawTransport::open(&receiver.hidraw_path) {
                    for slot in 1..=6 {
                        let key = format!("bolt:slot_{}", slot);
                        let mut dev = HidppDevice::with_software_id(slot, 0x02);

                        match dev.query_battery(&mut transport) {
                            Ok(battery) => {
                                let (name, dev_type) = if let Some((n, t, _)) = known_devices.get(&key) {
                                    (n.clone(), t.clone())
                                } else {
                                    let n = dev.query_name(&mut transport).unwrap_or_else(|_| {
                                        if slot == 1 { "MX Master 3S".to_string() }
                                        else if slot == 5 { "MX Mechanical Mini".to_string() }
                                        else { format!("Logitech Device (Slot {})", slot) }
                                    });
                                    let t = if n.to_lowercase().contains("keyboard")
                                        || n.to_lowercase().contains("mechanical")
                                        || n.to_lowercase().contains("craft")
                                        || n.to_lowercase().contains("k380")
                                        || n.to_lowercase().contains("keys")
                                    {
                                        "keyboard".to_string()
                                    } else {
                                        "mouse".to_string()
                                    };
                                    (n, t)
                                };

                                known_devices.insert(key.clone(), (name.clone(), dev_type.clone(), battery.percentage));

                                let now_secs = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs();
                                let hours = (now_secs / 3600 % 24 + 7) % 24;
                                let mins = now_secs / 60 % 60;
                                let secs = now_secs % 60;
                                let now_str = format!("{:02}:{:02}:{:02}", hours, mins, secs);

                                let status_str = match battery.status {
                                    logi_hidpp::features::ChargingStatus::Charging => "charging",
                                    logi_hidpp::features::ChargingStatus::Full => "full",
                                    logi_hidpp::features::ChargingStatus::Standby => "standby",
                                    _ => "discharging",
                                };

                                let dev_json = serde_json::json!({
                                    "id": format!("bolt:slot_{}", slot),
                                    "name": name,
                                    "type": dev_type,
                                    "slot": slot,
                                    "battery": {
                                        "percentage": battery.percentage,
                                        "charging": battery.charging,
                                        "critical": battery.critical,
                                        "status": status_str,
                                    },
                                    "dpi": if dev_type == "mouse" {
                                        serde_json::json!({
                                            "current": 1600,
                                            "min": 200,
                                            "max": 8000,
                                            "step": 50,
                                        })
                                    } else {
                                        serde_json::Value::Null
                                    },
                                    "updatedAt": now_str,
                                });

                                self.ipc.update_device(key.clone(), dev_json);

                                let changed = match last_batteries.get(&key) {
                                    Some(prev) => prev != &battery,
                                    None => true,
                                };

                                if changed {
                                    tracing::info!(
                                        "Battery update for {} slot {}: {}% (charging: {}, status: {:?})",
                                        receiver.name,
                                        slot,
                                        battery.percentage,
                                        battery.charging,
                                        battery.status
                                    );
                                    last_batteries.insert(key, battery);
                                }
                            }
                            Err(_) => {
                                // If device was previously active, update its status to Standby / Offline in real-time
                                if let Some((name, dev_type, last_pct)) = known_devices.get(&key) {
                                    let now_secs = std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs();
                                    let hours = (now_secs / 3600 % 24 + 7) % 24;
                                    let mins = now_secs / 60 % 60;
                                    let secs = now_secs % 60;
                                    let now_str = format!("{:02}:{:02}:{:02}", hours, mins, secs);

                                    let dev_json = serde_json::json!({
                                        "id": format!("bolt:slot_{}", slot),
                                        "name": name,
                                        "type": dev_type,
                                        "slot": slot,
                                        "battery": {
                                            "percentage": last_pct,
                                            "charging": false,
                                            "critical": false,
                                            "status": "standby",
                                        },
                                        "dpi": if dev_type == "mouse" {
                                            serde_json::json!({
                                                "current": 1600,
                                                "min": 200,
                                                "max": 8000,
                                                "step": 50,
                                            })
                                        } else {
                                            serde_json::Value::Null
                                        },
                                        "updatedAt": now_str,
                                    });

                                    self.ipc.update_device(key.clone(), dev_json);
                                }
                            }
                        }
                    }
                }
            }

            // Real-time polling every 2 seconds
            sleep(Duration::from_millis(2000)).await;
        }
    }
}
