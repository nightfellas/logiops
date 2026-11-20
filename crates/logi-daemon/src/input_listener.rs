use crate::gesture_engine::{GestureDirection, GestureTracker};
use crate::profile_manager::ProfileManager;
use crate::virtual_input::VirtualInputDevice;
use crate::window_tracker::WindowTracker;
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::Mutex as AsyncMutex;

const EV_SYN: u16 = 0x00;
const EV_KEY: u16 = 0x01;
const EV_REL: u16 = 0x02;

const REL_X: u16 = 0x00;
const REL_Y: u16 = 0x01;

const BTN_SIDE: u16 = 275;  // Thumb Back
const BTN_EXTRA: u16 = 276; // Thumb Forward
const BTN_TASK: u16 = 279;  // Gesture button
const KEY_LEFTMETA: u16 = 125;

// Linux EVIOCGRAB ioctl: _IOW('E', 0x90, int) = 0x40044590
const EVIOCGRAB: libc::c_ulong = 0x40044590;

#[repr(C)]
struct RawInputEvent {
    time: libc::timeval,
    type_: u16,
    code: u16,
    value: i32,
}

pub struct InputListener {
    running: Arc<AtomicBool>,
    profile_manager: Arc<Mutex<ProfileManager>>,
    virtual_input: Arc<AsyncMutex<VirtualInputDevice>>,
}

impl InputListener {
    pub fn new(
        running: Arc<AtomicBool>,
        profile_manager: Arc<Mutex<ProfileManager>>,
        virtual_input: Arc<AsyncMutex<VirtualInputDevice>>,
    ) -> Self {
        Self {
            running,
            profile_manager,
            virtual_input,
        }
    }

    pub fn find_logitech_mouse_event_node() -> Option<PathBuf> {
        let by_id = Path::new("/dev/input/by-id");
        if let Ok(entries) = std::fs::read_dir(by_id) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.contains("Logitech") && name.contains("event-mouse") {
                    if let Ok(canon) = entry.path().canonicalize() {
                        return Some(canon);
                    }
                    return Some(entry.path());
                }
            }
        }

        for i in 0..32 {
            let path = PathBuf::from(format!("/dev/input/event{}", i));
            if path.exists() {
                if let Ok(file) = File::open(&path) {
                    let fd = file.as_raw_fd();
                    let mut id: [u16; 4] = [0; 4];
                    let ioc = ((2u64) << 30) | ((b'E' as u64) << 8) | 0x02 | (8u64 << 16);
                    if unsafe { libc::ioctl(fd, ioc, id.as_mut_ptr()) } >= 0 {
                        if id[1] == 0x046d {
                            return Some(path);
                        }
                    }
                }
            }
        }

        None
    }

    pub async fn run_loop(self) {
        let mut gesture_tracker = GestureTracker::new(140);
        let mut window_tracker = WindowTracker::new();

        while self.running.load(Ordering::SeqCst) {
            let node = match Self::find_logitech_mouse_event_node() {
                Some(n) => n,
                None => {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }
            };

            tracing::info!("InputListener: Attaching to mouse event node at {:?}", node);

            let file_res = OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_NONBLOCK)
                .open(&node);

            let mut file = match file_res {
                Ok(f) => f,
                Err(e) => {
                    tracing::warn!("InputListener: Failed to open {:?}: {}. Retrying in 3s...", node, e);
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    continue;
                }
            };

            let mouse_fd = file.as_raw_fd();

            // Try to grab device exclusively so remapped buttons don't fire native actions
            let grabbed = unsafe { libc::ioctl(mouse_fd, EVIOCGRAB, 1) } == 0;
            if grabbed {
                tracing::info!("InputListener: Successfully grabbed mouse event node exclusively.");
            }

            // Also find and attach to HID++ node for reprogrammable button diversion (gesture button 0x00C3, mode shift 0x00C4)
            let receivers = logi_hidpp::discovery::scan_receivers();
            let mut hidraw_file = None;
            let mut reprog_feature_index: Option<u8> = None;

            for r in &receivers {
                if let Ok(mut transport) = logi_hidpp::HidrawTransport::open(&r.hidraw_path) {
                    let mut dev = logi_hidpp::HidppDevice::new(1);
                    if let Ok(feat_idx) = dev.divert_reprogrammable_controls(&mut transport) {
                        tracing::info!("Diverted reprogrammable controls (Gesture & Mode Shift) on feature index 0x{:02x}", feat_idx);
                        reprog_feature_index = Some(feat_idx);
                    }
                }
                if let Ok(f) = OpenOptions::new().read(true).write(true).custom_flags(libc::O_NONBLOCK).open(&r.hidraw_path) {
                    hidraw_file = Some(f);
                    break;
                }
            }

            let mut event_buf = [0u8; std::mem::size_of::<RawInputEvent>()];
            let mut hid_buf = [0u8; 64];
            let mut gesture_button_held = false;
            let mut mode_shift_held = false;

            while self.running.load(Ordering::SeqCst) {
                let mut pollfds = Vec::new();
                pollfds.push(libc::pollfd {
                    fd: mouse_fd,
                    events: libc::POLLIN,
                    revents: 0,
                });

                if let Some(ref hf) = hidraw_file {
                    pollfds.push(libc::pollfd {
                        fd: hf.as_raw_fd(),
                        events: libc::POLLIN,
                        revents: 0,
                    });
                }

                let ret = unsafe { libc::poll(pollfds.as_mut_ptr(), pollfds.len() as libc::nfds_t, 100) };
                if ret <= 0 {
                    tokio::task::yield_now().await;
                    continue;
                }

                let active_app = window_tracker.get_active_app().unwrap_or_else(|| "*".to_string());

                // 1. Process HID++ diverted button events from hidraw
                if pollfds.len() > 1 && (pollfds[1].revents & libc::POLLIN) != 0 {
                    if let Some(ref mut hf) = hidraw_file {
                        while let Ok(n) = hf.read(&mut hid_buf) {
                            if n < 6 { break; }
                            let feat_idx = hid_buf[2];

                            if Some(feat_idx) == reprog_feature_index || feat_idx == 0x09 {
                                tracing::info!("HID++ Reprog Controls packet (n={}): {:02x?}", n, &hid_buf[..n]);

                                let sw_id = hid_buf[3] & 0x0F;
                                let func_or_event = (hid_buf[3] >> 4) & 0x0F;

                                // Filter out command responses (software_id != 0).
                                // Notification events from mouse always have software_id == 0.
                                if sw_id == 0 && func_or_event == 0 {
                                    // Extract active CIDs
                                    let cid1 = u16::from_be_bytes([hid_buf[4], hid_buf[5]]);
                                    let cid2 = if n >= 8 { u16::from_be_bytes([hid_buf[6], hid_buf[7]]) } else { 0 };
                                    let cid3 = if n >= 10 { u16::from_be_bytes([hid_buf[8], hid_buf[9]]) } else { 0 };
                                    let cid4 = if n >= 12 { u16::from_be_bytes([hid_buf[10], hid_buf[11]]) } else { 0 };
                                    let active_cids = [cid1, cid2, cid3, cid4];

                                    let gesture_down = active_cids.contains(&logi_hidpp::features::CID_GESTURE_BUTTON);
                                    let mode_shift_down = active_cids.contains(&logi_hidpp::features::CID_MODE_SHIFT);

                                    let button_action = self.get_button_action("gesture_button", &active_app);
                                    let is_gesture_mode = matches!(button_action, Some(crate::profile_manager::ButtonAction::Gesture { .. }));

                                    if gesture_down && !gesture_button_held {
                                        gesture_button_held = true;
                                        tracing::info!("HID++ Gesture Button DOWN (is_gesture_mode: {})", is_gesture_mode);
                                        if is_gesture_mode {
                                            gesture_tracker.on_button_down();
                                        } else {
                                            // Non-gesture actions (e.g. Volume Up, KeyCombos) execute immediately on button press
                                            self.dispatch_button("gesture_button", &active_app).await;
                                        }
                                    } else if !gesture_down && gesture_button_held {
                                        gesture_button_held = false;
                                        tracing::info!("HID++ Gesture Button UP");
                                        if is_gesture_mode {
                                            let action = gesture_tracker.on_button_up();
                                            if action == GestureDirection::Click {
                                                self.dispatch_button("gesture_button", &active_app).await;
                                            }
                                        }
                                    }

                                    if mode_shift_down && !mode_shift_held {
                                        mode_shift_held = true;
                                        tracing::info!("HID++ Mode Shift / Top Button CLICK");
                                        self.dispatch_button("dpi_switch", &active_app).await;
                                    } else if !mode_shift_down {
                                        mode_shift_held = false;
                                    }
                                }
                            }
                        }
                    }
                }

                // 2. Process Mouse evdev events
                if (pollfds[0].revents & libc::POLLIN) != 0 {
                    while let Ok(n) = file.read(&mut event_buf) {
                        if n < event_buf.len() { break; }

                        let ev = unsafe { &*(event_buf.as_ptr() as *const RawInputEvent) };
                        let mut intercepted = false;

                        if ev.type_ == EV_KEY {
                            let pressed = ev.value == 1;
                            let released = ev.value == 0;

                            if ev.code == BTN_EXTRA {
                                intercepted = true;
                                if pressed {
                                    self.dispatch_button("thumb_forward", &active_app).await;
                                }
                            } else if ev.code == BTN_SIDE {
                                intercepted = true;
                                if pressed {
                                    self.dispatch_button("thumb_back", &active_app).await;
                                }
                            } else if ev.code == BTN_TASK || ev.code == KEY_LEFTMETA {
                                intercepted = true;
                                if pressed {
                                    gesture_tracker.on_button_down();
                                } else if released {
                                    let action = gesture_tracker.on_button_up();
                                    if action == GestureDirection::Click {
                                        self.dispatch_button("gesture_button", &active_app).await;
                                    }
                                }
                            }
                        } else if ev.type_ == EV_REL {
                            let button_action = self.get_button_action("gesture_button", &active_app);
                            let is_gesture_mode = matches!(button_action, Some(crate::profile_manager::ButtonAction::Gesture { .. }));

                            if (gesture_button_held || gesture_tracker.is_holding()) && is_gesture_mode {
                                intercepted = true;
                                let dx = if ev.code == REL_X { ev.value } else { 0 };
                                let dy = if ev.code == REL_Y { ev.value } else { 0 };

                                if let Some(direction) = gesture_tracker.on_motion(dx, dy) {
                                    match direction {
                                        GestureDirection::Left => self.dispatch_action("WORKSPACE_LEFT").await,
                                        GestureDirection::Right => self.dispatch_action("WORKSPACE_RIGHT").await,
                                        GestureDirection::Up => self.dispatch_action("MAXIMIZE_WINDOW").await,
                                        GestureDirection::Down => self.dispatch_action("MINIMIZE_WINDOW").await,
                                        _ => {}
                                    }
                                }
                            }
                        }

                        // Forward non-intercepted events directly to virtual input if grabbed
                        if grabbed && !intercepted {
                            let mut uinput = self.virtual_input.lock().await;
                            if ev.type_ == EV_KEY {
                                let _ = uinput.emit_key(ev.code, ev.value == 1);
                            } else if ev.type_ == EV_REL {
                                let _ = uinput.emit_rel(ev.code, ev.value);
                            } else if ev.type_ == EV_SYN {
                                let _ = uinput.sync();
                            }
                        }
                    }
                } else if (pollfds[0].revents & (libc::POLLERR | libc::POLLHUP)) != 0 {
                    tracing::warn!("InputListener: Mouse device node closed or disconnected.");
                    break;
                }
            }

            if grabbed {
                unsafe { libc::ioctl(mouse_fd, EVIOCGRAB, 0); }
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    fn get_button_action(&self, button_id: &str, app: &str) -> Option<crate::profile_manager::ButtonAction> {
        let mgr = self.profile_manager.lock().ok()?;
        let profile = mgr.get_profile_for_app("mx_master_3s", app)?;
        profile.buttons.get(button_id).cloned()
    }

    async fn dispatch_button(&self, button_id: &str, app: &str) {
        let action = self.get_button_action(button_id, app);

        if let Some(act) = action {
            match act {
                crate::profile_manager::ButtonAction::KeyCombo { keys } => {
                    let mut keycodes = Vec::new();
                    for k in &keys {
                        let code = match k.as_str() {
                            "KEY_LEFTCTRL" | "CTRL" => Some(29),
                            "KEY_LEFTSHIFT" | "SHIFT" => Some(42),
                            "KEY_LEFTALT" | "ALT" => Some(56),
                            "KEY_TAB" | "TAB" => Some(15),
                            "KEY_W" => Some(17),
                            "KEY_LEFTMETA" | "META" | "SUPER" => Some(125),
                            "KEY_D" => Some(32),
                            "KEY_VOLUMEUP" => Some(115),
                            "KEY_VOLUMEDOWN" => Some(114),
                            "KEY_MUTE" => Some(113),
                            "KEY_PLAYPAUSE" => Some(164),
                            _ => None,
                        };
                        if let Some(c) = code {
                            keycodes.push(c);
                        }
                    }
                    if !keycodes.is_empty() {
                        tracing::info!("Emitting virtual key combo {:?} for button {}", keycodes, button_id);
                        let mut uinput = self.virtual_input.lock().await;
                        let _ = uinput.emit_combo(&keycodes).await;
                    }
                }
                crate::profile_manager::ButtonAction::Gesture { click, .. } => {
                    if let Some(action_name) = click {
                        self.dispatch_action(&action_name).await;
                    }
                }
                _ => {}
            }
        }
    }

    pub async fn dispatch_action(&self, action_name: &str) {
        tracing::info!("Executing custom action: {}", action_name);

        match action_name {
            "OVERVIEW" | "Mission Control / Overview (Swipe for Workspaces)" => {
                // 1. Native GNOME Shell Overview toggle via DBus (Wayland & X11)
                let dbus_res = std::process::Command::new("gdbus")
                    .args(&[
                        "call", "--session",
                        "--dest", "org.gnome.Shell",
                        "--object-path", "/org/gnome/Shell",
                        "--method", "org.freedesktop.DBus.Properties.Get",
                        "org.gnome.Shell", "OverviewActive",
                    ])
                    .output();

                if let Ok(out) = dbus_res {
                    let text = String::from_utf8_lossy(&out.stdout);
                    let new_state = if text.contains("true") { "<false>" } else { "<true>" };
                    let _ = std::process::Command::new("gdbus")
                        .args(&[
                            "call", "--session",
                            "--dest", "org.gnome.Shell",
                            "--object-path", "/org/gnome/Shell",
                            "--method", "org.freedesktop.DBus.Properties.Set",
                            "org.gnome.Shell", "OverviewActive",
                            new_state,
                        ])
                        .spawn();
                }

                // 2. Also emit Alt+F1 on virtual input (universal Linux shortcut for overview / app menu)
                let mut uinput = self.virtual_input.lock().await;
                let _ = uinput.emit_combo(&[56, 59]).await;
            }
            "Next Tab (Ctrl + Tab)" | "TAB_NEXT" => {
                let mut uinput = self.virtual_input.lock().await;
                let _ = uinput.emit_combo(&[29, 15]).await; // Ctrl + Tab
            }
            "Previous Tab (Ctrl + Shift + Tab)" | "TAB_PREVIOUS" => {
                let mut uinput = self.virtual_input.lock().await;
                let _ = uinput.emit_combo(&[29, 42, 15]).await; // Ctrl + Shift + Tab
            }
            "Close Window / Tab (Ctrl + W)" => {
                let mut uinput = self.virtual_input.lock().await;
                let _ = uinput.emit_combo(&[29, 17]).await; // Ctrl + W
            }
            "Show Desktop" => {
                let mut uinput = self.virtual_input.lock().await;
                let _ = uinput.emit_combo(&[KEY_LEFTMETA, 32]).await; // Meta + D
            }
            "WORKSPACE_LEFT" => {
                let mut uinput = self.virtual_input.lock().await;
                let _ = uinput.emit_combo(&[29, 56, 105]).await; // Ctrl + Alt + Left
            }
            "WORKSPACE_RIGHT" => {
                let mut uinput = self.virtual_input.lock().await;
                let _ = uinput.emit_combo(&[29, 56, 106]).await; // Ctrl + Alt + Right
            }
            "MAXIMIZE_WINDOW" => {
                let mut uinput = self.virtual_input.lock().await;
                let _ = uinput.emit_combo(&[56, 68]).await; // Alt + F10 (GNOME default toggle-maximized)
            }
            "MINIMIZE_WINDOW" => {
                let mut uinput = self.virtual_input.lock().await;
                let _ = uinput.emit_combo(&[KEY_LEFTMETA, 35]).await; // Super + H (GNOME default minimize)
            }
            "Volume Up" => {
                let mut uinput = self.virtual_input.lock().await;
                let _ = uinput.emit_click(115).await;
            }
            "Volume Down" => {
                let mut uinput = self.virtual_input.lock().await;
                let _ = uinput.emit_click(114).await;
            }
            "Play / Pause Media" => {
                let mut uinput = self.virtual_input.lock().await;
                let _ = uinput.emit_click(164).await;
            }
            _ => {}
        }
    }
}
