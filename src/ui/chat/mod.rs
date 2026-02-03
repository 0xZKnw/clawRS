//! Chat interface components
//!
//! Contains the main chat view, message display, and input components.

pub mod input;
pub mod message;

use dioxus::prelude::*;
use input::ChatInput;
use message::{Message, MessageBubble, MessageRole};
use std::sync::atomic::Ordering;

use crate::app::{AppState, ModelState};
use crate::inference::engine::GenerationParams;
use crate::inference::streaming::StreamToken;

#[component]
pub fn ChatView() -> Element {
    let app_state = use_context::<AppState>();
    // State for messages
    let mut messages = use_signal(|| vec![
        Message {
            role: MessageRole::Assistant,
            content: "Hello! I'm LocaLM, your private AI assistant. How can I help you today?".to_string(),
        }
    ]);
    
    // State for generation status
    let mut is_generating = use_signal(|| false);

    // Handler for sending a message
    let handle_send = {
        let mut messages = messages.clone();
        let mut is_generating = is_generating.clone();
        let app_state = app_state.clone();
        move |text: String| {
            if !matches!(*app_state.model_state.read(), ModelState::Loaded(_)) {
                messages.write().push(Message {
                    role: MessageRole::Assistant,
                    content: "Model not loaded. Please load a model before generating responses.".to_string(),
                });
                return;
            }

            let prompt = text.clone();

            // Add user message immediately
            messages.write().push(Message {
                role: MessageRole::User,
                content: text,
            });

            // Add empty assistant message to stream into
            messages.write().push(Message {
                role: MessageRole::Assistant,
                content: String::new(),
            });

            app_state.stop_signal.store(false, Ordering::Relaxed);
            is_generating.set(true);

            let mut messages = messages.clone();
            let mut is_generating = is_generating.clone();
            let app_state = app_state.clone();

            spawn(async move {
                let params = GenerationParams::default();
                let (rx, stop_signal) = {
                    let engine = app_state.engine.lock().await;
                    match engine.generate_stream(&prompt, params) {
                        Ok(result) => result,
                        Err(e) => {
                            messages.write().push(Message {
                                role: MessageRole::Assistant,
                                content: format!("Error starting generation: {e}"),
                            });
                            is_generating.set(false);
                            return;
                        }
                    }
                };

                loop {
                    if app_state.stop_signal.load(Ordering::Relaxed) {
                        stop_signal.store(true, Ordering::Relaxed);
                    }

                    match rx.try_recv() {
                        Ok(StreamToken::Token(text)) => {
                            let mut msgs = messages.write();
                            if let Some(last) = msgs.last_mut() {
                                last.content.push_str(&text);
                            }
                        }
                        Ok(StreamToken::Done) => break,
                        Ok(StreamToken::Error(e)) => {
                            let mut msgs = messages.write();
                            if let Some(last) = msgs.last_mut() {
                                if !last.content.is_empty() {
                                    last.content.push_str("\n\n");
                                }
                                last.content.push_str(&format!("[Error] {e}"));
                            } else {
                                msgs.push(Message {
                                    role: MessageRole::Assistant,
                                    content: format!("Error: {e}"),
                                });
                            }
                            break;
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => {
                            tokio::task::yield_now().await;
                        }
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
                    }
                }

                is_generating.set(false);
            });
        }
    };

    // Handler for stopping generation
    let handle_stop = {
        let mut is_generating = is_generating.clone();
        let app_state = app_state.clone();
        move |_| {
            app_state.stop_signal.store(true, Ordering::Relaxed);
            is_generating.set(false);
        }
    };

    rsx! {
        div { class: "flex flex-col h-full bg-[var(--bg-main)] relative",
            
            // Header / Toolbar (Optional, can be added later)
            // div { class: "h-12 border-b border-[var(--border-subtle)] flex items-center px-4", ... }

            // Messages Area
            div { class: "flex-1 overflow-y-auto p-4 space-y-2 custom-scrollbar scroll-smooth",
                // Message List
                for (idx, msg) in messages.read().iter().enumerate() {
                    MessageBubble { key: "{idx}", message: msg.clone() }
                }
                
                // Typing / Generating Indicator
                if is_generating() {
                    div { class: "flex items-center gap-2 text-[var(--text-tertiary)] text-sm ml-4 mt-2 animate-fade-in",
                        div { class: "w-2 h-2 bg-[var(--accent-primary)] rounded-full animate-bounce" }
                        div { class: "w-2 h-2 bg-[var(--accent-primary)] rounded-full animate-bounce delay-75" }
                        div { class: "w-2 h-2 bg-[var(--accent-primary)] rounded-full animate-bounce delay-150" }
                        span { "LocaLM is thinking..." }
                    }
                }
                
                // Invisible anchor for auto-scrolling
                // In a real implementation, we'd use a use_effect to scroll this into view
                div { class: "h-4" }
            }

            // Input Area
            ChatInput {
                on_send: handle_send,
                on_stop: handle_stop,
                is_generating: is_generating(),
            }
        }
    }
}
