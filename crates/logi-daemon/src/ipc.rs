use crate::profile_manager::ProfileManager;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::broadcast;

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

pub fn get_socket_path() -> PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        Path::new(&runtime_dir).join("logioptions.sock")
    } else {
        PathBuf::from(format!("/tmp/logioptions-{}.sock", unsafe { libc::getuid() }))
    }
}

pub struct IpcServer {
    socket_path: PathBuf,
    event_tx: broadcast::Sender<String>,
    profile_manager: Option<Arc<Mutex<ProfileManager>>>,
}

impl IpcServer {
    pub fn new() -> Self {
        Self::with_profile_manager(None)
    }

    pub fn with_profile_manager(pm: Option<Arc<Mutex<ProfileManager>>>) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        Self {
            socket_path: get_socket_path(),
            event_tx,
            profile_manager: pm,
        }
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    pub fn broadcast_event(&self, method: &str, params: Value) {
        let event_msg = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });
        if let Ok(serialized) = serde_json::to_string(&event_msg) {
            let _ = self.event_tx.send(serialized);
        }
    }

    pub async fn start(self: Arc<Self>) -> Result<(), std::io::Error> {
        if self.socket_path.exists() {
            let _ = std::fs::remove_file(&self.socket_path);
        }

        let listener = UnixListener::bind(&self.socket_path)?;
        tracing::info!("IPC Server listening on {:?}", self.socket_path);

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let server = Arc::clone(&self);
                    let mut event_rx = server.event_tx.subscribe();

                    tokio::spawn(async move {
                        let (reader, mut writer) = stream.into_split();
                        let mut buf_reader = BufReader::new(reader);
                        let mut line = String::new();

                        loop {
                            tokio::select! {
                                read_res = buf_reader.read_line(&mut line) => {
                                    match read_res {
                                        Ok(0) => break,
                                        Ok(_) => {
                                            let trimmed = line.trim();
                                            if !trimmed.is_empty() {
                                                let resp = server.handle_request(trimmed).await;
                                                if let Ok(mut resp_str) = serde_json::to_string(&resp) {
                                                    resp_str.push('\n');
                                                    let _ = writer.write_all(resp_str.as_bytes()).await;
                                                }
                                            }
                                            line.clear();
                                        }
                                        Err(_) => break,
                                    }
                                }
                                event_res = event_rx.recv() => {
                                    if let Ok(mut event_str) = event_res {
                                        event_str.push('\n');
                                        let _ = writer.write_all(event_str.as_bytes()).await;
                                    }
                                }
                            }
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("IPC accept error: {}", e);
                }
            }
        }
    }

    async fn handle_request(&self, req_str: &str) -> JsonRpcResponse {
        let parsed: Result<JsonRpcRequest, _> = serde_json::from_str(req_str);
        match parsed {
            Ok(req) => {
                let result = match req.method.as_str() {
                    "ping" => Ok(serde_json::json!("pong")),
                    "daemon.version" => Ok(serde_json::json!({
                        "version": env!("CARGO_PKG_VERSION"),
                        "daemon": "logi-daemon",
                    })),
                    "device.list" => Ok(serde_json::json!({
                        "devices": []
                    })),
                    "device.set_dpi" => {
                        let dpi = req.params.as_ref()
                            .and_then(|p| p.get("dpi"))
                            .and_then(|d| d.as_u64())
                            .unwrap_or(1600) as u16;

                        tracing::info!("Setting hardware DPI to {}", dpi);
                        let receivers = logi_hidpp::discovery::scan_receivers();
                        let mut success = false;
                        for r in &receivers {
                            tracing::info!("Scanning receiver at {:?} for DPI adjustment", r.hidraw_path);
                            if let Ok(mut transport) = logi_hidpp::HidrawTransport::open(&r.hidraw_path) {
                                for slot in 1..=6 {
                                    let mut dev = logi_hidpp::HidppDevice::new(slot);
                                    match dev.set_dpi(&mut transport, dpi) {
                                        Ok(_) => {
                                            tracing::info!("Successfully set DPI {} on slot {}", dpi, slot);
                                            success = true;
                                        }
                                        Err(e) => {
                                            tracing::debug!("dev.set_dpi on slot {} failed: {:?}", slot, e);
                                        }
                                    }
                                }
                            }
                        }
                        Ok(serde_json::json!({ "success": success, "dpi": dpi }))
                    }
                    "profile.reload" => {
                        if let Some(ref pm) = self.profile_manager {
                            if let Ok(mut mgr) = pm.lock() {
                                *mgr = ProfileManager::load_or_default();
                                tracing::info!("ProfileManager reloaded from disk.");
                            }
                        }
                        Ok(serde_json::json!({ "reloaded": true }))
                    }
                    _ => Err(serde_json::json!({
                        "code": -32601,
                        "message": format!("Method not found: {}", req.method),
                    })),
                };

                match result {
                    Ok(val) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: req.id,
                        result: Some(val),
                        error: None,
                    },
                    Err(err) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: req.id,
                        result: None,
                        error: Some(err),
                    },
                }
            }
            Err(e) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: None,
                result: None,
                error: Some(serde_json::json!({
                    "code": -32700,
                    "message": format!("Parse error: {}", e),
                })),
            },
        }
    }
}
