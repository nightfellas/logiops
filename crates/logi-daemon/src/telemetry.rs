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

        loop {
            let receivers = scan_receivers();
            for receiver in receivers {
                if let Ok(mut transport) = HidrawTransport::open(&receiver.hidraw_path) {
                    for slot in 1..=6 {
                        let mut dev = HidppDevice::new(slot);
                        if let Ok(battery) = dev.query_battery(&mut transport) {
                            let key = format!("{}:slot_{}", receiver.product_id, slot);
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

                                self.ipc.broadcast_event(
                                    "event.battery_updated",
                                    serde_json::json!({
                                        "receiver": receiver.name,
                                        "slot": slot,
                                        "percentage": battery.percentage,
                                        "charging": battery.charging,
                                        "status": battery.status,
                                        "standby": battery.standby,
                                        "critical": battery.critical,
                                    }),
                                );

                                last_batteries.insert(key, battery);
                            }
                        }
                    }
                }
            }

            // Sleep 60s for next polling cycle
            sleep(Duration::from_secs(60)).await;
        }
    }
}
