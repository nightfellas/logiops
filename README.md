# 🐧 LogiOptions-Linux

<p align="center">
  <img src="assets/icons/logioptions.svg" width="128" height="128" alt="LogiOptions Linux Logo" />
</p>

<p align="center">
  <b>A Modern, Lightweight, and Universal Linux Companion for Logitech MX Peripherals</b><br>
  <i>Engineered as a high-performance alternative to Logi Options+, solving the critical weaknesses of Solaar and logiops.</i>
</p>

<p align="center">
  <a href="#-key-features">Features</a> •
  <a href="#-system-requirements">Requirements</a> •
  <a href="#-installation--setup">Installation</a> •
  <a href="#-architecture--crates">Architecture</a> •
  <a href="#-configuration--profiles">Profiles</a> •
  <a href="#-troubleshooting--faq">FAQ</a>
</p>

---

> [!NOTE]
> ### ⚠️ Experimental Project (Born Out of Desperation)
> This project is inherently **experimental**, born out of sheer frustration and desperation because Logitech has persistently refused to release an official **Logi Options+** client for Linux despite years of community requests.
>
> Built by and for Linux power users who refuse to let their flagship MX Master 3S, MX Mechanical, or MX Anywhere hardware function as dumb standard peripherals, this suite reverse-engineers Logitech's HID++ protocol directly on the Linux input and hidraw subsystems to bring native gestures, seamless per-app profiling, and glanceable battery telemetry to modern Wayland and X11 desktops.

---

## 📌 Problem Statement & Objective

While Logitech peripherals such as the **MX Master 3/3S**, **MX Mechanical / Mini**, and **MX Anywhere 3** are popular among Linux power users, existing software solutions suffer from major compromises:

* **Solaar**: Primarily a device pairing manager with an outdated GTK interface. Configuration is presented as raw, cryptic register flags rather than a modern visual representation. Lacks per-app profiling and smooth directional gestures.
* **logiops (`logid`)**: Runs headlessly via manual text files (`/etc/logid.cfg`). Requires editing terminal files, restarting root daemons, lacks live battery status, provides no GUI, and struggles with dynamic Wayland window focus changes.
* **Official Logi Options+**: Not available for Linux. On other platforms, it consumes 350 MB+ memory with heavy background telemetry.

**LogiOptions-Linux** delivers a complete, cohesive Linux desktop experience:
1. **Ultra-lightweight footprint:** Complete compiled suite is barely **~2.2 MB** and uses `< 15 MB` RAM.
2. **Glanceable Telemetry:** Integrates features inspired by **[LogiBatteryBar](https://github.com/nightfellas/LogiBatteryBar)** for instant tray battery monitoring.
3. **Interactive 2D Visual GUI:** Clickable indicator pins on device schematics for intuitive button mapping.
4. **Per-App Custom Profiles:** Adapts button shortcuts, horizontal thumb-wheel behavior, and sensor DPI dynamically based on the active application.
5. **Universal Linux Compatibility:** Works seamlessly across GNOME, KDE Plasma, XFCE, Hyprland, and Sway under both Wayland and X11.

---

## ✨ Key Features

### 🔋 1. Glanceable BatteryBar & Live Telemetry
Inspired by the macOS utility *LogiBatteryBar*, you no longer need to open bulky windows to check your device power:
* **Multi-Device Support:** Tracks both mouse and keyboard simultaneously on a single Logi Bolt or Unifying receiver.
* **⚡ USB-C Charging Detection:** Real-time charging state reporting using HID++ 2.0 Unified Battery (`0x1004`). Automatically shows a charging lightning badge.
* **⚡ Instant Power-Off & Standby Detection:** Leverages native Bolt receiver error reporting (`ERR_CONNECT_FAIL` `0x04`). When a device power switch is turned off, the daemon detects it in under 2ms and updates its status to `Standby (Off)` without I/O timeouts or UI lag.
* **💤 Smart RF Sleep & Standby Retention:** Wireless Logitech keyboards enter deep RF sleep when idle. LogiOptions retains the last known battery percentage, marking the device as `Standby (Off)` in the tray menu while cleanly keeping the top bar clutter-free. It immediately reactivates on the first keystroke or movement.
* **Clean Top Bar Widget:** Native GNOME Shell / Ayatana AppIndicator integration with zero visual clutter:
  * Mouse active: `[Mouse Icon] 70%`
  * Mouse + Keyboard active: `[Mouse Icon] 70%   ⌨️ 85%`
  * Charging indicator: `⚡ 70%`
* **System Wake Hook:** Integrates with systemd / logind DBus (`PrepareForSleep`) to instantly refresh device status the moment your laptop or PC wakes from suspend.

### 🖱️ 2. Interactive Visual Hardware Mapping
* High-detail 2D vector schematic of the mouse with pulsing, interactive pins.
* Click any pin to remap physical actions:
  * **Thumb Rest Gesture Button**
  * **Thumb Wheel** (horizontal scroll / timeline scrub / volume)
  * **Thumb Forward & Back Buttons**
  * **Middle Click & Mode Shift Button**

### 🎯 3. Dynamic Per-App Profiles
* Automatically switches configuration profiles based on the focused window:
  * **Google Chrome / Firefox:** Horizontal wheel switches browser tabs; gesture button opens new tab.
  * **DaVinci Resolve / Premiere Pro:** Horizontal wheel scrubs timeline; buttons map to ripple delete and cut.
  * **VS Code / IDEs:** Thumb buttons navigate code symbols or history.
* Built-in support for Wayland (**Hyprland**, **Sway**, **KDE Plasma**) and **X11** with graceful fallback to the Global profile.

### ✋ 4. Directional Gesture Recognition Engine
* Hold the gesture button and move the mouse in four directions:
  * **Swipe Left:** Previous Workspace (`Ctrl + Alt + Left`)
  * **Swipe Right:** Next Workspace (`Ctrl + Alt + Right`)
  * **Swipe Up:** Maximize Active Window
  * **Swipe Down:** Minimize Active Window / Show Desktop
  * **Tap Click:** Mission Control / Overview (`Super` / `Meta` key)
* **🔄 Self-Healing Hardware Diversion:** Logitech MX peripherals automatically reset their internal hardware diversion registers back to default when entering deep RF power-saving sleep or upon computer reboot. LogiOptions-Linux implements proactive kernel-event driven self-healing: the instant the mouse detects user interaction (`evdev` motion or click), the daemon verifies and re-asserts the diversion flags in `< 2ms`, ensuring gesture navigation never drops offline.

### ⚙️ 5. Precision Hardware Controls
* **Adjustable DPI (Feature `0x2201`):** Smooth resolution adjustment from 200 up to 8000 DPI in 50 DPI steps with one-click presets (800, 1200, 1600, 2400, 4000).
* **SmartShift Ratchet Mode (Feature `0x2110`):** Toggle between free-spin and ratchet mode, and adjust torque threshold sensitivity.

### 🛡️ 6. Safety Crash Watchdog
* Intercepts `SIGINT`, `SIGTERM`, and Rust panic hooks to cleanly reset diverted hardware buttons back to standard kernel input before exit, preventing your physical mouse from becoming unresponsive if the daemon stops.

---

## 🖥️ System Requirements

| Component | Minimum Requirement | Recommended |
| :--- | :--- | :--- |
| **Operating System** | Linux (Ubuntu, Debian, Fedora, Arch, openSUSE) | Any modern Linux distro |
| **Linux Kernel** | `>= 5.15` | `>= 6.0` |
| **Display Server** | Wayland or X11 | Wayland (GNOME, KDE Plasma, Hyprland, Sway) |
| **Hardware** | Logitech Bolt Receiver (`0xc548`), Unifying (`0xc52b`), or Bluetooth | Logitech MX Master 3/3S, MX Mechanical, MX Anywhere |
| **Build Tools** | Rust stable (`cargo`, `rustc`), Node.js `>= 18` (for UI build) | `rustup`, Node.js 20+ |

### System Dependencies

Before compiling, ensure required GUI runtime packages are installed:

**Ubuntu / Debian:**
```bash
sudo apt install -y build-essential python3-gi gir1.2-gtk-3.0 gir1.2-webkit2-4.1 gir1.2-ayatanaappindicator3-0.1
```

**Fedora:**
```bash
sudo dnf install -y gcc python3-gobject gtk3 webkit2gtk4.1 libayatana-appindicator-gtk3
```

**Arch Linux:**
```bash
sudo pacman -S --needed base-devel python-gobject gtk3 webkit2gtk-4.1 libayatana-appindicator
```

---

## 🚀 Installation & Setup

### Step 1: Clone the Repository

```bash
git clone https://github.com/yusuf/logiops.git
cd logiops
```

---

### Step 2: Configure Hardware Permissions (udev Rules)

To enable communication with Logitech receivers (`/dev/hidraw*`) and virtual input creation (`/dev/uinput`) without requiring `sudo` or root privileges:

```bash
sudo cp packaging/udev/99-logitech-options.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules && sudo udevadm trigger --action=change
```

> **Note:** If your user is not already in the `plugdev` or `input` group, you can add yourself:
> ```bash
> sudo usermod -aG plugdev $USER
> ```
> Then unplug and re-plug your USB receiver once.

---

### Step 3: Run the Automated Local Installer

LogiOptions-Linux includes a complete, one-step installer script:

```bash
./packaging/install.sh
```

**What this script does automatically:**
1. Compiles optimized release binaries (`logid-service` and `logi-options`).
2. Builds the frontend UI assets.
3. Installs executables to `~/.local/bin/`.
4. Installs the desktop application menu shortcut and scalable SVG icon to `~/.local/share/applications/`.
5. Configures desktop autostart (`~/.config/autostart/logioptions.desktop`) with `--tray` so your battery indicator launches seamlessly on graphical login without opening a bulky window.
6. Configures and starts the background service under `systemctl --user enable --now logioptions-daemon.service`.

Once completed, you can launch the app directly from your application launcher or terminal:
```bash
logi-options
```

---

### Alternative: Building a Standalone AppImage

If you prefer a single, self-contained executable that runs on any distribution without installation:

```bash
./packaging/appimage/build-appimage.sh
```

The script will compile the release binaries, bundle assets into `build/AppDir`, and package the final `LogiOptions-x86_64.AppImage`.

---

## 🏗️ Architecture & Crates

The codebase is organized as a Cargo workspace with clean separation of concerns:

```
logiops/
├── assets/
│   └── icons/
│       └── logioptions.svg             # Modern scalable app icon
├── packaging/
│   ├── udev/
│   │   └── 99-logitech-options.rules   # Zero-sudo uaccess rules
│   ├── appimage/                       # AppRun wrapper & desktop file
│   ├── systemd/                        # User systemd service unit
│   ├── flatpak/                        # Flathub manifest
│   ├── install.sh                      # One-click automated installer
│   └── uninstall.sh                    # Clean uninstallation script
├── crates/
│   ├── logi-hidpp/                     # Low-level HID++ 2.0 protocol crate
│   │   ├── src/
│   │   │   ├── error.rs                # Protocol & transport error types
│   │   │   ├── report.rs               # Short/Long/VeryLong packet parser
│   │   │   ├── transport.rs            # Non-blocking /dev/hidraw + ioctl
│   │   │   ├── device.rs               # HidppDevice abstraction & root cache
│   │   │   ├── discovery.rs            # /sys/class/hidraw auto-scanner
│   │   │   └── features/               # Battery, Name, DPI, SmartShift
│   │   └── tests/                      # Automated protocol tests
│   ├── logi-daemon/                    # Background async service
│   │   ├── src/
│   │   │   ├── gesture_engine.rs       # Directional gesture vector math
│   │   │   ├── profile_manager.rs      # JSON profile loader & app matcher
│   │   │   ├── window_tracker.rs       # Active window focus detector
│   │   │   ├── virtual_input.rs        # Kernel /dev/uinput synthetic injector
│   │   │   ├── telemetry.rs            # Real-time adaptive battery & device presence polling
│   │   │   ├── safety.rs               # Crash recovery & panic watchdog
│   │   │   ├── ipc.rs                  # JSON-RPC 2.0 Unix Domain Socket
│   │   │   └── main.rs                 # Daemon service entry point
│   │   └── tests/                      # Daemon integration tests
│   └── logi-gui/                       # Modern Frontend UI
│       ├── src-tauri/                  # Tauri v2 backend launcher
│       └── ui/                         # Vue 3 + Tailwind CSS + Lucide
│           └── src/components/
│               ├── BatteryPopover.vue  # LogiBatteryBar glanceable card
│               ├── DeviceVisual.vue    # Interactive SVG mouse diagram
│               ├── ButtonConfigModal.vue # Custom action & gesture mapper
│               ├── DpiSlider.vue       # 200-8000 DPI slider
│               ├── SmartShiftControl.vue # Ratchet torque slider
│               └── ProfileBar.vue      # Per-app profile switcher
└── README.md
```

---

## ⚙️ Configuration & Profiles

Configuration is stored in human-readable JSON at `~/.config/logioptions/profiles.json`.

### Example `profiles.json`:

```json
{
  "version": 1,
  "settings": {
    "poll_interval_battery_sec": 60,
    "notifications_enabled": true
  },
  "devices": {
    "mx_master_3s": {
      "default_dpi": 1600,
      "smartshift": {
        "enabled": true,
        "threshold": 32
      },
      "profiles": [
        {
          "id": "global_default",
          "name": "Global (Default)",
          "match_app": "*",
          "dpi": 1600,
          "buttons": {
            "gesture_button": {
              "type": "gesture",
              "click": "OVERVIEW",
              "up": "MAXIMIZE_WINDOW",
              "down": "MINIMIZE_WINDOW",
              "left": "WORKSPACE_LEFT",
              "right": "WORKSPACE_RIGHT"
            },
            "thumb_forward": {
              "type": "key_combo",
              "keys": ["KEY_LEFTCTRL", "KEY_TAB"]
            },
            "thumb_back": {
              "type": "key_combo",
              "keys": ["KEY_LEFTCTRL", "KEY_LEFTSHIFT", "KEY_TAB"]
            }
          },
          "thumbwheel": {
            "divert": false,
            "invert": false
          }
        },
        {
          "id": "chrome",
          "name": "Google Chrome",
          "match_app": "google-chrome",
          "dpi": 2000,
          "buttons": {
            "gesture_button": {
              "type": "key_combo",
              "keys": ["KEY_LEFTCTRL", "KEY_T"]
            }
          },
          "thumbwheel": {
            "divert": true,
            "action_up": "TAB_PREVIOUS",
            "action_down": "TAB_NEXT"
          }
        }
      ]
    }
  }
}
```

---

## 📡 IPC Protocol (JSON-RPC 2.0)

The daemon exposes a Unix Domain Socket at:
```
/run/user/<UID>/logioptions.sock
```

You can interact with it via `nc -U`, Python, or any JSON-RPC client:

### Query Daemon Version:
```bash
echo '{"jsonrpc": "2.0", "id": 1, "method": "daemon.version"}' | nc -U /run/user/$UID/logioptions.sock
```
**Response:**
```json
{"jsonrpc":"2.0","id":1,"result":{"daemon":"logi-daemon","version":"0.1.0"}}
```

### List Connected Devices & Real-Time Battery:
```bash
echo '{"jsonrpc": "2.0", "id": 1, "method": "device.list"}' | nc -U /run/user/$UID/logioptions.sock
```
**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "devices": [
      {
        "id": "bolt:slot_1",
        "name": "MX Master 3S",
        "slot": 1,
        "type": "mouse",
        "battery": { "percentage": 70, "charging": false, "status": "discharging" },
        "dpi": { "current": 1600, "min": 200, "max": 8000, "step": 50 }
      },
      {
        "id": "bolt:slot_5",
        "name": "MX Mechanical Mini",
        "slot": 5,
        "type": "keyboard",
        "battery": { "percentage": 85, "charging": false, "status": "standby" }
      }
    ]
  }
}
```

### Change Hardware DPI on the Fly:
```bash
echo '{"jsonrpc": "2.0", "id": 1, "method": "device.set_dpi", "params": {"dpi": 2400}}' | nc -U /run/user/$UID/logioptions.sock
```

### Reload Profile Settings:
```bash
echo '{"jsonrpc": "2.0", "id": 1, "method": "profile.reload"}' | nc -U /run/user/$UID/logioptions.sock
```

### Battery Update Broadcast Event:
```json
{
  "jsonrpc": "2.0",
  "method": "event.battery_updated",
  "params": {
    "receiver": "Logitech USB Receiver",
    "slot": 1,
    "percentage": 75,
    "charging": false,
    "status": "discharging",
    "standby": false,
    "critical": false
  }
}
```

---

## ❓ Troubleshooting & FAQ

#### 1. Why does `logid-service` report `Permission denied` on `/dev/hidraw*` or `/dev/uinput`?
* **Cause:** The udev rule has not been applied or your user lacks group access.
* **Solution:** Re-apply the udev rule and trigger change:
  ```bash
  sudo cp packaging/udev/99-logitech-options.rules /etc/udev/rules.d/
  sudo udevadm control --reload-rules && sudo udevadm trigger --action=change
  ```
  Ensure your user is in the `plugdev` group (`groups $USER`).

#### 2. Why does my keyboard show `Standby (Off)` in the menu or disappear from the top bar?
* **Explanation:** Wireless Logitech keyboards enter an aggressive RF sleep timer after a few seconds of inactivity to preserve battery life, or are physically turned off. LogiOptions retains the last known battery percentage, marking the device as `Standby (Off)` in the dropdown menu while omitting it from the top bar to keep the panel clean. The moment the keyboard wakes up or is switched on, the top bar re-displays it within 2 seconds.

#### 3. Gesture button not working after computer reboot?
* **Explanation:** Logitech MX mice firmware resets reprogrammable button diversion registers upon deep RF sleep or boot. LogiOptions-Linux includes kernel-event-driven self-healing diversion that automatically re-asserts diversion flags on your very first mouse movement after waking up. If gesture actions are still unresponsive, verify that another tool (such as Solaar with button diversion enabled or an old `logid` service) is not holding exclusive locks on the Bolt receiver's hidraw interface. You can restart the daemon at any time:
  ```bash
  systemctl --user restart logioptions-daemon.service
  ```

#### 4. Top bar tray icon does not appear in GNOME:
* GNOME Shell requires the AppIndicator extension to show tray icons:
  ```bash
  sudo apt install gnome-shell-extension-appindicator
  ```
  Enable it via the GNOME Extensions app.

#### 5. How do I preview or develop the UI in a browser?
* Run the Vite development server:
  ```bash
  cd crates/logi-gui/ui
  npm run dev
  ```
  Open `http://localhost:5173` in your browser.

#### 6. How do I completely uninstall LogiOptions-Linux?
* Run the included uninstall script:
  ```bash
  ./packaging/uninstall.sh
  ```
  This will stop the systemd daemon, disable auto-start, and remove all installed binaries, desktop shortcuts, and icons.

---

## 🧪 Running Automated Tests

Run the full automated test suite across all crates:

```bash
cargo test --workspace
```

---

## 📄 License

This project is licensed under the **MIT License**. Free to use, modify, and distribute.
