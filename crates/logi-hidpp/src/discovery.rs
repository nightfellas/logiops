use std::fs;
use std::path::{Path, PathBuf};

pub const LOGITECH_VID: u16 = 0x046d;
pub const BOLT_RECEIVER_PID: u16 = 0xc548;
pub const UNIFYING_RECEIVER_PID: u16 = 0xc52b;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReceiverType {
    Bolt,
    Unifying,
    BluetoothDirect,
    Other,
}

#[derive(Debug, Clone)]
pub struct DiscoveredReceiver {
    pub hidraw_path: PathBuf,
    pub vendor_id: u16,
    pub product_id: u16,
    pub name: String,
    pub receiver_type: ReceiverType,
}

/// Scans `/sys/class/hidraw` to locate all connected Logitech receivers and direct devices.
pub fn scan_receivers() -> Vec<DiscoveredReceiver> {
    let mut receivers = Vec::new();
    let sys_hidraw = Path::new("/sys/class/hidraw");

    if let Ok(entries) = fs::read_dir(sys_hidraw) {
        for entry in entries.flatten() {
            let desc_path = entry.path().join("device").join("report_descriptor");
            // Only accept devices whose report descriptor supports HID++ (report ID 0x10 or 0x11)
            if let Ok(desc) = fs::read(&desc_path) {
                if !desc.windows(2).any(|w| w == [0x85, 0x10] || w == [0x85, 0x11]) {
                    continue;
                }
            }

            let uevent_path = entry.path().join("device").join("uevent");
            if let Ok(content) = fs::read_to_string(&uevent_path) {
                if let Some(receiver) = parse_uevent(&content, entry.file_name().to_string_lossy().as_ref()) {
                    receivers.push(receiver);
                }
            }
        }
    }

    receivers
}

fn parse_uevent(content: &str, hidraw_name: &str) -> Option<DiscoveredReceiver> {
    let mut vid = None;
    let mut pid = None;
    let mut name = String::new();

    for line in content.lines() {
        if let Some(hid_id) = line.strip_prefix("HID_ID=") {
            // Format: BUS:VENDOR:PRODUCT (e.g. 0003:0000046D:0000C548)
            let parts: Vec<&str> = hid_id.split(':').collect();
            if parts.len() >= 3 {
                vid = u16::from_str_radix(parts[1], 16).ok();
                pid = u16::from_str_radix(parts[2], 16).ok();
            }
        } else if let Some(hid_name) = line.strip_prefix("HID_NAME=") {
            name = hid_name.to_string();
        }
    }

    if let (Some(v), Some(p)) = (vid, pid) {
        if v == LOGITECH_VID {
            let receiver_type = match p {
                BOLT_RECEIVER_PID => ReceiverType::Bolt,
                UNIFYING_RECEIVER_PID => ReceiverType::Unifying,
                _ if name.to_lowercase().contains("bluetooth") => ReceiverType::BluetoothDirect,
                _ => ReceiverType::Other,
            };

            return Some(DiscoveredReceiver {
                hidraw_path: PathBuf::from(format!("/dev/{}", hidraw_name)),
                vendor_id: v,
                product_id: p,
                name,
                receiver_type,
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uevent_bolt() {
        let sample = "DRIVER=logitech-djreceiver\nHID_ID=0003:0000046D:0000C548\nHID_NAME=Logitech Logi Bolt Receiver\nHID_PHYS=usb-0000:00:14.0-1/input2\nHID_UNIQ=\nMODALIAS=hid:b0003g0001v0000046Dp0000C548\n";
        let parsed = parse_uevent(sample, "hidraw3").expect("Should parse bolt receiver");
        assert_eq!(parsed.vendor_id, 0x046d);
        assert_eq!(parsed.product_id, 0xc548);
        assert_eq!(parsed.receiver_type, ReceiverType::Bolt);
        assert_eq!(parsed.hidraw_path, PathBuf::from("/dev/hidraw3"));
    }
}
