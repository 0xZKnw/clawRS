//! Mobile companion server
//!
//! Embedded HTTP + WebSocket server that allows the ClawRS mobile app
//! to control the desktop application over local Wi-Fi.
//!
//! ## Architecture
//! The server runs in its own Tokio task, completely independent of Dioxus.
//! It communicates with the desktop app through:
//! - Disk storage (settings.json, conversations/) — shared filesystem
//! - Its own LlamaEngine instance for inference
//! - Broadcast channels for WebSocket events

pub mod auth;
pub mod routes;
pub mod ws;

use crate::inference::LlamaEngine;
use axum::Router;
use dashmap::DashMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, OnceLock};
use tokio::sync::{broadcast, Mutex};
use tower_http::cors::{Any, CorsLayer};

// ============================================================================
// Global shared state — accessible from both server thread and Dioxus UI
// ============================================================================

static SHARED_PAIRING_CODE: OnceLock<std::sync::Mutex<String>> = OnceLock::new();
static SHARED_LOCAL_IP: OnceLock<String> = OnceLock::new();

/// Global LlamaEngine shared between Desktop UI and Mobile API
pub static SHARED_ENGINE: OnceLock<Arc<Mutex<LlamaEngine>>> = OnceLock::new();

/// Global stop signal
pub static SHARED_STOP_SIGNAL: OnceLock<Arc<AtomicBool>> = OnceLock::new();

/// Global is generating flag
pub static SHARED_IS_GENERATING: OnceLock<Arc<AtomicBool>> = OnceLock::new();

pub static SHARED_WS_TX: OnceLock<broadcast::Sender<WsEvent>> = OnceLock::new();

/// Global Agent shared between Desktop UI and Mobile API for handling tool execution
pub static SHARED_AGENT: OnceLock<Arc<crate::agent::Agent>> = OnceLock::new();


pub fn get_shared_engine() -> Arc<Mutex<LlamaEngine>> {
    SHARED_ENGINE.get_or_init(|| Arc::new(Mutex::new(LlamaEngine::new()))).clone()
}

pub fn get_shared_agent() -> Arc<crate::agent::Agent> {
    SHARED_AGENT.get_or_init(|| {
        let config = crate::agent::AgentConfig::default();
        // Create Agent without tool initialization.
        // Tools are initialized by the Dioxus `use_effect` in app.rs,
        // which shares this same Arc<Agent> instance.
        // We must NOT call tokio::spawn or block_on here because OnceLock
        // may be triggered from any thread/runtime context.
        Arc::new(crate::agent::Agent::new(config))
    }).clone()
}

pub fn get_shared_stop_signal() -> Arc<AtomicBool> {
    SHARED_STOP_SIGNAL.get_or_init(|| Arc::new(AtomicBool::new(false))).clone()
}

pub fn get_shared_is_generating() -> Arc<AtomicBool> {
    SHARED_IS_GENERATING.get_or_init(|| Arc::new(AtomicBool::new(false))).clone()
}

pub fn get_shared_ws_tx() -> broadcast::Sender<WsEvent> {
    SHARED_WS_TX.get_or_init(|| {
        let (tx, _) = broadcast::channel(256);
        tx
    }).clone()
}

/// Get the current pairing code (callable from any thread including Dioxus UI)
pub fn get_pairing_code() -> String {
    SHARED_PAIRING_CODE
        .get()
        .map(|m| m.lock().unwrap().clone())
        .unwrap_or_else(|| "------".to_string())
}

/// Set/regenerate the pairing code (callable from any thread including Dioxus UI)
pub fn regenerate_pairing_code() -> String {
    use rand::Rng;
    let code: u32 = rand::thread_rng().gen_range(100_000..999_999);
    let code_str = code.to_string();

    match SHARED_PAIRING_CODE.get() {
        Some(m) => {
            *m.lock().unwrap() = code_str.clone();
        }
        None => {
            let _ = SHARED_PAIRING_CODE.set(std::sync::Mutex::new(code_str.clone()));
        }
    }

    tracing::info!("Pairing code set to: {}", code_str);
    code_str
}

/// Get the local network IP (cached, callable from any thread)
pub fn get_cached_local_ip() -> String {
    SHARED_LOCAL_IP
        .get_or_init(|| {
            get_local_ip().unwrap_or_else(|_| "Unknown".to_string())
        })
        .clone()
}

/// Events broadcast over WebSocket to connected mobile clients
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type", content = "data")]
pub enum WsEvent {
    /// Token streamed during generation
    Token(String),
    /// Generation finished
    GenerationDone,
    /// Model state changed
    ModelStateChanged {
        state: String,
        model_name: Option<String>,
    },
    /// Tool permission request from the agent
    ToolPermissionRequest {
        id: String,
        tool_name: String,
        description: String,
        params: String,
    },
    /// System resource update
    SystemUpdate {
        ram_used_mb: u64,
        ram_total_mb: u64,
        gpu_name: Option<String>,
        vram_total_gb: Option<f64>,
    },
    /// Connection confirmed
    Connected,
    /// Error message
    Error(String),
}

/// Pending tool approval waiting for mobile response
#[derive(Debug, Clone)]
pub struct PendingToolApproval {
    pub tool_name: String,
    pub description: String,
    pub params: String,
    pub response_tx: Arc<Mutex<Option<tokio::sync::oneshot::Sender<bool>>>>,
}

/// Current model state (thread-safe version, not using Dioxus Signal)
#[derive(Debug, Clone, PartialEq)]
pub enum ServerModelState {
    NotLoaded,
    Loading,
    Loaded(String),
    Error(String),
}

/// Server-side shared state (fully thread-safe, no Dioxus dependency)
#[derive(Clone)]
pub struct ServerState {
    /// Own engine instance for inference
    pub engine: Arc<Mutex<LlamaEngine>>,
    /// Current model state
    pub model_state: Arc<Mutex<ServerModelState>>,
    /// Whether generation is in progress
    pub is_generating: Arc<AtomicBool>,
    /// Stop signal for generation
    pub stop_signal: Arc<AtomicBool>,
    /// Active pairing code (None = server disabled)
    pub pairing_code: Arc<Mutex<Option<String>>>,
    /// Valid session tokens (token → timestamp)
    pub session_tokens: Arc<DashMap<String, std::time::Instant>>,
    /// Pending tool approvals (id → approval)
    pub pending_approvals: Arc<DashMap<String, PendingToolApproval>>,
    /// Broadcast channel for WebSocket events
    pub ws_tx: broadcast::Sender<WsEvent>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            engine: get_shared_engine(),
            model_state: Arc::new(Mutex::new(ServerModelState::NotLoaded)),
            is_generating: get_shared_is_generating(),
            stop_signal: get_shared_stop_signal(),
            pairing_code: Arc::new(Mutex::new(None)),
            session_tokens: Arc::new(DashMap::new()),
            pending_approvals: Arc::new(DashMap::new()),
            ws_tx: get_shared_ws_tx(),
        }
    }

    /// Generate a new 6-digit pairing code (syncs with global shared state)
    pub async fn generate_pairing_code(&self) -> String {
        let code_str = regenerate_pairing_code();
        *self.pairing_code.lock().await = Some(code_str.clone());
        code_str
    }

    /// Refresh pairing code from the global shared state
    pub async fn refresh_pairing_code(&self) {
        let code = get_pairing_code();
        if code != "------" {
            *self.pairing_code.lock().await = Some(code);
        }
    }

    /// Validate a session token
    pub fn is_valid_session(&self, token: &str) -> bool {
        self.session_tokens.contains_key(token)
    }
}

/// Start the mobile companion server on the given port
pub async fn start_server(state: ServerState, port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .nest("/api", routes::api_routes(state.clone()))
        .layer(cors)
        .with_state(state.clone());

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("Mobile companion server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    // Log the local IP for the user
    if let Ok(local_ip) = get_local_ip() {
        tracing::info!("Mobile app can connect to: {}:{}", local_ip, port);
    }

    // Spawn UDP discovery responder alongside HTTP server
    tokio::spawn(run_discovery_responder(port));

    axum::serve(listener, app).await?;

    Ok(())
}

/// UDP discovery responder
///
/// Listens on port DEFAULT_PORT + 1 for broadcast packets containing "CLAWRS_DISCOVER".
/// Responds with a JSON payload so the mobile app can auto-detect the desktop.
async fn run_discovery_responder(http_port: u16) {
    let discovery_port = http_port + 1; // 9742
    let addr = format!("0.0.0.0:{}", discovery_port);

    let socket = match tokio::net::UdpSocket::bind(&addr).await {
        Ok(s) => {
            // Allow the socket to receive broadcast packets
            if let Err(e) = s.set_broadcast(true) {
                tracing::warn!("Failed to set broadcast on discovery socket: {}", e);
            }
            tracing::info!("UDP discovery responder listening on port {}", discovery_port);
            s
        }
        Err(e) => {
            tracing::warn!("Could not bind discovery socket on {}: {}", addr, e);
            return;
        }
    };

    let mut buf = [0u8; 256];

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, src)) => {
                let msg = String::from_utf8_lossy(&buf[..len]);
                if msg.trim() == "CLAWRS_DISCOVER" {
                    let local_ip = get_cached_local_ip();
                    let code = get_pairing_code();
                    let response = serde_json::json!({
                        "service": "clawrs",
                        "ip": local_ip,
                        "port": http_port,
                        "code": code,
                        "version": env!("CARGO_PKG_VERSION"),
                    });
                    let response_bytes = response.to_string();

                    if let Err(e) = socket.send_to(response_bytes.as_bytes(), src).await {
                        tracing::warn!("Failed to send discovery response: {}", e);
                    } else {
                        tracing::info!("Discovery response sent to {}", src);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Discovery socket error: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }
}

/// Get the local network IP address
pub fn get_local_ip() -> Result<String, Box<dyn std::error::Error>> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;
    let local_addr = socket.local_addr()?;
    Ok(local_addr.ip().to_string())
}

/// Default server port
pub const DEFAULT_PORT: u16 = 9741;

/// Discovery port (DEFAULT_PORT + 1)
pub const DISCOVERY_PORT: u16 = DEFAULT_PORT + 1;
