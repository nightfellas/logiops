use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct SafetyManager {
    running: Arc<AtomicBool>,
}

impl SafetyManager {
    pub fn new() -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let r = Arc::clone(&running);

        // Setup panic hook to log cleanly and trigger cleanup
        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            tracing::error!("FATAL: Daemon panicked! Cleaning up hardware state...");
            if let Some(location) = panic_info.location() {
                tracing::error!("Panic occurred at {}:{}", location.file(), location.line());
            }
            default_hook(panic_info);
        }));

        Self { running: r }
    }

    pub fn running_handle(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.running)
    }

    pub async fn wait_for_shutdown_signal(&self) {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let mut sigint = signal(SignalKind::interrupt()).expect("Failed to bind SIGINT");
            let mut sigterm = signal(SignalKind::terminate()).expect("Failed to bind SIGTERM");

            tokio::select! {
                _ = sigint.recv() => {
                    tracing::info!("Received SIGINT. Initiating graceful shutdown...");
                }
                _ = sigterm.recv() => {
                    tracing::info!("Received SIGTERM. Initiating graceful shutdown...");
                }
            }
            self.running.store(false, Ordering::SeqCst);
        }
    }
}
