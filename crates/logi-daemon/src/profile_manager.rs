use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ButtonAction {
    Native,
    KeyCombo {
        keys: Vec<String>,
    },
    Gesture {
        click: Option<String>,
        up: Option<String>,
        down: Option<String>,
        left: Option<String>,
        right: Option<String>,
    },
    DpiSwitch {
        dpi: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThumbWheelConfig {
    pub divert: bool,
    pub invert: bool,
    pub action_up: Option<String>,
    pub action_down: Option<String>,
}

impl Default for ThumbWheelConfig {
    fn default() -> Self {
        Self {
            divert: false,
            invert: false,
            action_up: None,
            action_down: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub match_app: String,
    pub dpi: Option<u16>,
    pub buttons: HashMap<String, ButtonAction>,
    pub thumbwheel: ThumbWheelConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SmartShiftConfig {
    pub enabled: bool,
    pub threshold: u8,
}

impl Default for SmartShiftConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold: 32,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub default_dpi: u16,
    pub smartshift: SmartShiftConfig,
    pub profiles: Vec<Profile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonSettings {
    pub poll_interval_battery_sec: u64,
    pub notifications_enabled: bool,
}

impl Default for DaemonSettings {
    fn default() -> Self {
        Self {
            poll_interval_battery_sec: 60,
            notifications_enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    pub version: u32,
    pub settings: DaemonSettings,
    pub devices: HashMap<String, DeviceConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut buttons = HashMap::new();
        buttons.insert(
            "gesture_button".to_string(),
            ButtonAction::Gesture {
                click: Some("OVERVIEW".to_string()),
                up: Some("MAXIMIZE_WINDOW".to_string()),
                down: Some("MINIMIZE_WINDOW".to_string()),
                left: Some("WORKSPACE_LEFT".to_string()),
                right: Some("WORKSPACE_RIGHT".to_string()),
            },
        );
        buttons.insert(
            "thumb_forward".to_string(),
            ButtonAction::KeyCombo {
                keys: vec!["KEY_LEFTCTRL".to_string(), "KEY_TAB".to_string()],
            },
        );
        buttons.insert(
            "thumb_back".to_string(),
            ButtonAction::KeyCombo {
                keys: vec![
                    "KEY_LEFTCTRL".to_string(),
                    "KEY_LEFTSHIFT".to_string(),
                    "KEY_TAB".to_string(),
                ],
            },
        );

        let global_profile = Profile {
            id: "global_default".to_string(),
            name: "Global (Default)".to_string(),
            match_app: "*".to_string(),
            dpi: Some(1600),
            buttons,
            thumbwheel: ThumbWheelConfig::default(),
        };

        let mut devices = HashMap::new();
        devices.insert(
            "mx_master_3s".to_string(),
            DeviceConfig {
                default_dpi: 1600,
                smartshift: SmartShiftConfig::default(),
                profiles: vec![global_profile],
            },
        );

        Self {
            version: 1,
            settings: DaemonSettings::default(),
            devices,
        }
    }
}

pub struct ProfileManager {
    config_path: PathBuf,
    pub config: AppConfig,
}

impl ProfileManager {
    pub fn config_file_path() -> PathBuf {
        if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(config_home).join("logioptions").join("profiles.json")
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".config").join("logioptions").join("profiles.json")
        } else {
            PathBuf::from("profiles.json")
        }
    }

    pub fn load_or_default() -> Self {
        let path = Self::config_file_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                    return Self {
                        config_path: path,
                        config,
                    };
                }
            }
        }

        let default_config = AppConfig::default();
        let manager = Self {
            config_path: path,
            config: default_config,
        };
        let _ = manager.save();
        manager
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&self.config).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
        })?;
        fs::write(&self.config_path, json)?;
        Ok(())
    }

    pub fn get_profile_for_app(&self, device_key: &str, app_name: &str) -> Option<&Profile> {
        let dev = self.config.devices.get(device_key)?;
        // First look for exact or substring match in per-app profiles
        for profile in &dev.profiles {
            if profile.match_app != "*" && (app_name.contains(&profile.match_app) || profile.match_app.contains(app_name)) {
                return Some(profile);
            }
        }
        // Fallback to global profile
        dev.profiles.iter().find(|p| p.match_app == "*")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_matching() {
        let mut app_config = AppConfig::default();
        let dev_config = app_config.devices.get_mut("mx_master_3s").unwrap();

        dev_config.profiles.push(Profile {
            id: "chrome_profile".to_string(),
            name: "Google Chrome".to_string(),
            match_app: "google-chrome".to_string(),
            dpi: Some(2400),
            buttons: HashMap::new(),
            thumbwheel: ThumbWheelConfig::default(),
        });

        let mgr = ProfileManager {
            config_path: PathBuf::from("/tmp/test_profiles.json"),
            config: app_config,
        };

        // Match chrome
        let p_chrome = mgr.get_profile_for_app("mx_master_3s", "google-chrome").unwrap();
        assert_eq!(p_chrome.id, "chrome_profile");
        assert_eq!(p_chrome.dpi, Some(2400));

        // Match unknown app -> fallback to global
        let p_default = mgr.get_profile_for_app("mx_master_3s", "unknown_app").unwrap();
        assert_eq!(p_default.id, "global_default");
    }
}
