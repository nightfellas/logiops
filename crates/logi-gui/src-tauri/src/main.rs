use logi_daemon::ipc::get_socket_path;
use std::path::{Path, PathBuf};
use std::process::Command;

fn find_window_launcher() -> Option<PathBuf> {
    // 1. Check user share location
    if let Ok(home) = std::env::var("HOME") {
        let user_share = PathBuf::from(home).join(".local/share/logioptions/window_launcher.py");
        if user_share.exists() {
            return Some(user_share);
        }
    }

    // 2. Check relative to binary
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join("../share/logioptions/window_launcher.py");
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    // 3. Check development repository path
    let dev_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src-tauri/window_launcher.py");
    if dev_path.exists() {
        return Some(dev_path);
    }

    None
}

fn main() {
    let socket = get_socket_path();
    println!("LogiOptions Linux GUI connecting to: {:?}", socket);

    if let Some(launcher) = find_window_launcher() {
        println!("Launching GUI window via {:?}", launcher);
        let status = Command::new("python3")
            .arg(&launcher)
            .args(std::env::args().skip(1))
            .status();

        match status {
            Ok(s) => std::process::exit(s.code().unwrap_or(0)),
            Err(e) => eprintln!("Failed to spawn GUI window: {}", e),
        }
    } else {
        eprintln!("Error: Window launcher not found. Please run ./packaging/install.sh");
        std::process::exit(1);
    }
}
