#!/usr/bin/env python3
import os
import sys
import json
import threading
import socket
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
DIST_DIR = os.path.abspath(os.path.join(SCRIPT_DIR, "../../ui/dist"))
if not os.path.exists(DIST_DIR):
    alt_dir = os.path.expanduser("~/.local/share/logioptions/ui/dist")
    if os.path.exists(alt_dir):
        DIST_DIR = alt_dir

PROFILES_PATH = os.path.expanduser("~/.config/logioptions/profiles.json")

def send_ipc(method, params=None):
    try:
        sock_path = f"/run/user/{os.getuid()}/logioptions.sock"
        if os.path.exists(sock_path):
            with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as s:
                s.settimeout(0.5)
                s.connect(sock_path)
                msg = json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params or {}}) + "\n"
                s.sendall(msg.encode())
    except Exception as e:
        print("IPC send error:", e)

# Custom HTTP handler with API support
class LogiOptionsHandler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=DIST_DIR, **kwargs)

    def do_POST(self):
        if self.path == "/api/set_dpi":
            length = int(self.headers.get('Content-Length', 0))
            body = self.rfile.read(length)
            try:
                data = json.loads(body.decode('utf-8'))
                dpi = int(data.get("dpi", 1600))
                print(f"[API] Setting hardware DPI to {dpi}")
                send_ipc("device.set_dpi", {"dpi": dpi})
                self.send_response(200)
                self.send_header('Content-Type', 'application/json')
                self.end_headers()
                self.wfile.write(b'{"status":"ok"}')
            except Exception as e:
                self.send_response(500)
                self.end_headers()
                self.wfile.write(str(e).encode('utf-8'))
            return

        if self.path == "/api/save_action":
            length = int(self.headers.get('Content-Length', 0))
            body = self.rfile.read(length)
            try:
                data = json.loads(body.decode('utf-8'))
                btn_id = data.get("button_id")
                action_name = data.get("action_name")
                print(f"[API] Saving action for {btn_id}: {action_name}")

                # Update profiles.json
                if os.path.exists(PROFILES_PATH):
                    with open(PROFILES_PATH, 'r') as f:
                        cfg = json.load(f)
                    
                    dev = cfg.get("devices", {}).get("mx_master_3s", {})
                    profiles = dev.get("profiles", [])
                    for p in profiles:
                        if p.get("match_app") == "*":
                            if "buttons" not in p:
                                p["buttons"] = {}
                            if "Next Tab" in action_name:
                                p["buttons"][btn_id] = {"type": "key_combo", "keys": ["KEY_LEFTCTRL", "KEY_TAB"]}
                            elif "Previous Tab" in action_name:
                                p["buttons"][btn_id] = {"type": "key_combo", "keys": ["KEY_LEFTCTRL", "KEY_LEFTSHIFT", "KEY_TAB"]}
                            elif "Close" in action_name:
                                p["buttons"][btn_id] = {"type": "key_combo", "keys": ["KEY_LEFTCTRL", "KEY_W"]}
                            elif "Show Desktop" in action_name:
                                p["buttons"][btn_id] = {"type": "key_combo", "keys": ["KEY_LEFTMETA", "KEY_D"]}
                            elif "Volume Up" in action_name:
                                p["buttons"][btn_id] = {"type": "key_combo", "keys": ["KEY_VOLUMEUP"]}
                            elif "Volume Down" in action_name:
                                p["buttons"][btn_id] = {"type": "key_combo", "keys": ["KEY_VOLUMEDOWN"]}
                            elif "Play" in action_name:
                                p["buttons"][btn_id] = {"type": "key_combo", "keys": ["KEY_PLAYPAUSE"]}
                            elif "Overview" in action_name or "Mission" in action_name:
                                p["buttons"][btn_id] = {"type": "gesture", "click": "OVERVIEW"}
                    
                    with open(PROFILES_PATH, 'w') as f:
                        json.dump(cfg, f, indent=2)

                # Tell daemon to reload profile
                send_ipc("profile.reload")

                self.send_response(200)
                self.send_header('Content-Type', 'application/json')
                self.end_headers()
                self.wfile.write(b'{"status":"ok"}')
            except Exception as e:
                self.send_response(500)
                self.end_headers()
                self.wfile.write(str(e).encode('utf-8'))
            return

        super().do_POST()

def find_free_port():
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(('127.0.0.1', 0))
        return s.getsockname()[1]

PORT = 8765
try:
    server = ThreadingHTTPServer(('127.0.0.1', PORT), LogiOptionsHandler)
except OSError:
    PORT = find_free_port()
    server = ThreadingHTTPServer(('127.0.0.1', PORT), LogiOptionsHandler)

http_thread = threading.Thread(target=server.serve_forever, daemon=True)
http_thread.start()

URL = f"http://127.0.0.1:{PORT}/index.html"

# Initialize GTK & AyatanaAppIndicator
try:
    import gi
    gi.require_version('Gtk', '3.0')
    try:
        gi.require_version('WebKit2', '4.1')
    except (ValueError, AttributeError):
        gi.require_version('WebKit2', '4.0')
    from gi.repository import Gtk, WebKit2, GdkPixbuf

    indicator_mod = None
    for name in ['AyatanaAppIndicator3', 'AppIndicator3']:
        try:
            gi.require_version(name, '0.1')
            indicator_mod = __import__(f'gi.repository.{name}', fromlist=[name])
            break
        except Exception:
            pass

    # Prefer dark theme
    settings = Gtk.Settings.get_default()
    if settings:
        settings.set_property("gtk-application-prefer-dark-theme", True)

    win = Gtk.Window()
    win.set_title("LogiOptions-Linux")
    win.set_default_size(1050, 720)
    win.set_position(Gtk.WindowPosition.CENTER)

    # Set App Icon
    icon_path = os.path.abspath(os.path.join(SCRIPT_DIR, "../../../assets/icons/logioptions.svg"))
    if not os.path.exists(icon_path):
        icon_path = os.path.expanduser("~/.local/share/icons/hicolor/scalable/apps/logioptions.svg")
    if os.path.exists(icon_path):
        try:
            pixbuf = GdkPixbuf.Pixbuf.new_from_file_at_scale(icon_path, 64, 64, True)
            win.set_icon(pixbuf)
        except Exception:
            pass

    wk_settings = WebKit2.Settings()
    wk_settings.set_enable_javascript(True)
    wk_settings.set_enable_developer_extras(True)

    webview = WebKit2.WebView.new_with_settings(wk_settings)
    webview.load_uri(URL)
    win.add(webview)

    # Minimize/Hide to tray instead of quitting on close
    def on_delete(widget, event):
        win.hide()
        return True # Prevent destruction

    win.connect("delete-event", on_delete)

    # Create System Tray / Top Bar Widget
    if indicator_mod:
        ind = indicator_mod.Indicator.new(
            "logioptions-battery-bar",
            "input-mouse",
            indicator_mod.IndicatorCategory.HARDWARE
        )
        ind.set_status(indicator_mod.IndicatorStatus.ACTIVE)
        ind.set_label("🖱️ 75%", "🖱️ 100%")

        # Top Bar Tray Menu
        menu = Gtk.Menu()

        item_open = Gtk.MenuItem(label="Open LogiOptions")
        def open_window(widget):
            win.show_all()
            win.present()
        item_open.connect("activate", open_window)
        menu.append(item_open)

        item_status = Gtk.MenuItem(label="MX Master 3S: 75% (Slot 1)")
        item_status.set_sensitive(False)
        menu.append(item_status)

        item_sep = Gtk.SeparatorMenuItem()
        menu.append(item_sep)

        item_quit = Gtk.MenuItem(label="Quit")
        item_quit.connect("activate", lambda w: Gtk.main_quit())
        menu.append(item_quit)

        menu.show_all()
        ind.set_menu(menu)

    win.show_all()
    win.present()

    Gtk.main()
    sys.exit(0)

except Exception as e:
    print(f"Native GTK WebKit error: {e}, falling back to browser mode...")
    import subprocess
    import shutil

    if shutil.which("google-chrome"):
        subprocess.run(["google-chrome", f"--app={URL}"])
    else:
        import webbrowser
        webbrowser.open(URL)
        try:
            http_thread.join()
        except KeyboardInterrupt:
            pass
