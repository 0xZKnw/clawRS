//! Root Dioxus application component
//!
//! This module contains the main App component that serves as the root of the UI tree.

use crate::inference::LlamaEngine;
use crate::storage::conversations::Conversation;
use crate::storage::settings::{AppSettings, load_settings};
use crate::ui::Layout;
use crate::agent::{Agent, AgentConfig};
use dioxus::prelude::*;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::ui::chat::message::Message;

/// Represents the current state of the model
#[derive(Clone, PartialEq, Debug)]
pub enum ModelState {
    NotLoaded,
    Loading,
    Loaded(String),
    Error(String),
}

/// Global application state shared across components
#[derive(Clone)]
pub struct AppState {
    pub agent: Arc<Agent>,
    pub engine: Arc<Mutex<LlamaEngine>>,
    pub current_conversation: Signal<Option<Conversation>>,
    pub conversations: Signal<Vec<Conversation>>,
    pub settings: Signal<AppSettings>,
    pub model_state: Signal<ModelState>,
    pub stop_signal: Arc<AtomicBool>,
    /// Global generation flag - generation continues even when navigating away
    pub is_generating: Signal<bool>,
    /// Active messages buffer - persists across navigation
    pub active_messages: Signal<Vec<Message>>,
}

impl AppState {
    pub fn new() -> Self {
        tracing::info!("AppState initialized");
        let settings = load_settings();
        let mut agent_config = AgentConfig::default();
        agent_config.disabled_mcp_servers = settings.disabled_mcp_servers.clone();
        
        Self {
            agent: crate::server::get_shared_agent(),
            engine: crate::server::get_shared_engine(),
            current_conversation: Signal::new(None),
            conversations: Signal::new(Vec::new()),
            settings: Signal::new(settings),
            model_state: Signal::new(ModelState::NotLoaded),
            stop_signal: crate::server::get_shared_stop_signal(),
            is_generating: Signal::new(false),
            active_messages: Signal::new(Vec::new()),
        }
    }
}

#[component]
pub fn App() -> Element {
    let mut app_state = AppState::new();
    use_context_provider(|| app_state.clone());

    {
        let agent = use_context::<AppState>().agent.clone();
        use_effect(move || {
            let agent = agent.clone();
            spawn(async move {
                if let Err(e) = agent.initialize_tools().await {
                    tracing::error!("Failed to initialize tools: {}", e);
                }
            });
        });
    }

    // Coroutine to listen for Mobile Server events and update the Desktop UI safely
    use_coroutine(move |mut _rx: UnboundedReceiver<()>| async move {
        // Subscribe to WS channel
        let mut ws_rx = crate::server::get_shared_ws_tx().subscribe();
        
        while let Ok(event) = ws_rx.recv().await {
            match event {
                crate::server::WsEvent::ModelStateChanged { state, model_name } => {
                    let new_state = match state.as_str() {
                        "loaded" => {
                            if let Some(name) = model_name {
                                ModelState::Loaded(name)
                            } else {
                                ModelState::Loaded("Unknown".to_string())
                            }
                        }
                        "loading" => ModelState::Loading,
                        "not_loaded" => ModelState::NotLoaded,
                        _ => {
                            // Try to see if it's an error
                            if state.starts_with("error") {
                                ModelState::Error(state.clone())
                            } else {
                                ModelState::NotLoaded
                            }
                        }
                    };
                    app_state.model_state.set(new_state);
                }
                _ => {}
            }
        }
    });

    rsx! {
        Layout {}
    }
}
