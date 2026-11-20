use std::process::Command;

pub struct WindowTracker {
    last_app: Option<String>,
}

impl WindowTracker {
    pub fn new() -> Self {
        Self { last_app: None }
    }

    /// Tries to detect active window name under Wayland (Hyprland/Sway/KDE) or X11.
    pub fn get_active_app(&mut self) -> Option<String> {
        let app = self.detect_active_app();
        if app != self.last_app {
            self.last_app = app.clone();
        }
        app
    }

    fn detect_active_app(&self) -> Option<String> {
        // 1. Check Hyprland Wayland
        if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            if let Ok(output) = Command::new("hyprctl").args(["activewindow", "-j"]).output() {
                if let Ok(text) = String::from_utf8(output.stdout) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                        if let Some(class) = json.get("class").and_then(|c| c.as_str()) {
                            if !class.is_empty() {
                                return Some(class.to_lowercase());
                            }
                        }
                    }
                }
            }
        }

        // 2. Check Sway Wayland
        if std::env::var("SWAYSOCK").is_ok() {
            if let Ok(output) = Command::new("swaymsg").args(["-t", "get_tree"]).output() {
                if let Ok(text) = String::from_utf8(output.stdout) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                        if let Some(app) = find_sway_focused_app(&json) {
                            return Some(app.to_lowercase());
                        }
                    }
                }
            }
        }

        // 3. Check X11 via xdotool or xprop
        if std::env::var("DISPLAY").is_ok() {
            if let Ok(output) = Command::new("xdotool").args(["getactivewindow", "getwindowclassname"]).output() {
                if output.status.success() {
                    let name = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();
                    if !name.is_empty() {
                        return Some(name);
                    }
                }
            }
        }

        None
    }
}

fn find_sway_focused_app(val: &serde_json::Value) -> Option<String> {
    if val.get("focused").and_then(|f| f.as_bool()).unwrap_or(false) {
        if let Some(app_id) = val.get("app_id").and_then(|a| a.as_str()) {
            return Some(app_id.to_string());
        }
        if let Some(window_properties) = val.get("window_properties") {
            if let Some(class) = window_properties.get("class").and_then(|c| c.as_str()) {
                return Some(class.to_string());
            }
        }
    }

    if let Some(nodes) = val.get("nodes").and_then(|n| n.as_array()) {
        for node in nodes {
            if let Some(found) = find_sway_focused_app(node) {
                return Some(found);
            }
        }
    }

    if let Some(floating_nodes) = val.get("floating_nodes").and_then(|n| n.as_array()) {
        for node in floating_nodes {
            if let Some(found) = find_sway_focused_app(node) {
                return Some(found);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_initialization() {
        let mut tracker = WindowTracker::new();
        // Just verify it runs gracefully even if no GUI session in test
        let _ = tracker.get_active_app();
    }
}
