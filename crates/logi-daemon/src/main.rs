use logi_daemon::input_listener::InputListener;
use logi_daemon::ipc::IpcServer;
use logi_daemon::profile_manager::ProfileManager;
use logi_daemon::safety::SafetyManager;
use logi_daemon::telemetry::TelemetryService;
use logi_daemon::virtual_input::VirtualInputDevice;
use logi_hidpp::discovery::scan_receivers;
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as AsyncMutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("Initializing LogiOptions Linux Daemon...");

    // 1. Safety & Crash Watchdog
    let safety = SafetyManager::new();

    // 2. Scan connected hardware
    let receivers = scan_receivers();
    tracing::info!("Found {} Logitech receiver(s) / device(s):", receivers.len());
    for r in &receivers {
        tracing::info!(
            " - [{:?}] {} (VID: {:04x}, PID: {:04x}) at {:?}",
            r.receiver_type,
            r.name,
            r.vendor_id,
            r.product_id,
            r.hidraw_path
        );
    }

    // 3. Virtual Input subsystem (/dev/uinput)
    let virtual_input = match VirtualInputDevice::new("LogiOptions Virtual Input") {
        Ok(v) => {
            tracing::info!("Initialized /dev/uinput virtual input device");
            Some(Arc::new(AsyncMutex::new(v)))
        }
        Err(e) => {
            tracing::warn!("Failed to initialize /dev/uinput (running without virtual input): {}", e);
            None
        }
    };

    // 4. Load Profile Manager (~/.config/logioptions/profiles.json)
    let profile_manager = Arc::new(Mutex::new(ProfileManager::load_or_default()));

    // 5. Start Hardware Input Listener & Action Dispatcher
    if let Some(v_input) = virtual_input.clone() {
        let listener = InputListener::new(
            safety.running_handle(),
            Arc::clone(&profile_manager),
            v_input,
        );
        tokio::spawn(async move {
            listener.run_loop().await;
        });
    }

    // 6. Start IPC Server
    let ipc = Arc::new(IpcServer::with_profile_manager(Some(Arc::clone(&profile_manager))));
    let ipc_runner = Arc::clone(&ipc);
    tokio::spawn(async move {
        if let Err(e) = ipc_runner.start().await {
            tracing::error!("IPC Server stopped with error: {}", e);
        }
    });

    // 7. Start Telemetry / Battery polling loop
    let telemetry = TelemetryService::new(Arc::clone(&ipc));
    tokio::spawn(async move {
        telemetry.run_polling_loop().await;
    });

    tracing::info!("LogiOptions Daemon is ready and running with custom action dispatcher.");

    // 8. Wait for termination signal (SIGINT, SIGTERM)
    safety.wait_for_shutdown_signal().await;
    tracing::info!("Shutdown complete. Bye!");

    Ok(())
}
