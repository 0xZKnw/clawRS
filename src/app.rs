//! Root Dioxus application component
//!
//! This module contains the main App component that serves as the root of the UI tree.

use crate::inference::LlamaEngine;
use crate::storage::conversations::Conversation;
use crate::storage::settings::AppSettings;
use crate::ui::Layout;
use dioxus::prelude::*;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::Mutex;

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
    pub engine: Arc<Mutex<LlamaEngine>>,
    pub current_conversation: Signal<Option<Conversation>>,
    pub conversations: Signal<Vec<Conversation>>,
    pub settings: Signal<AppSettings>,
    pub model_state: Signal<ModelState>,
    pub stop_signal: Arc<AtomicBool>,
}

impl AppState {
    pub fn new() -> Self {
        tracing::info!("AppState initialized");
        Self {
            engine: Arc::new(Mutex::new(LlamaEngine::new())),
            current_conversation: Signal::new(None),
            conversations: Signal::new(Vec::new()),
            settings: Signal::new(AppSettings::default()),
            model_state: Signal::new(ModelState::NotLoaded),
            stop_signal: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[component]
pub fn App() -> Element {
    let app_state = AppState::new();
    use_context_provider(|| app_state);

    rsx! {
        Layout {}
    }
}
