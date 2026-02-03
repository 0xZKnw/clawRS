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
use crate::storage::conversations::save_conversation;
use crate::types::message::Message as StorageMessage;

#[component]
pub fn ChatView() -> Element {
    let app_state = use_context::<AppState>();
    
    // State for messages - will be populated from current_conversation
    let messages = use_signal(Vec::<Message>::new);
    
    // State for generation status
    let is_generating = use_signal(|| false);
    
    // Load messages when current_conversation changes
    {
        let mut messages = messages.clone();
        let current_conv = app_state.current_conversation.clone();
        use_effect(move || {
            let conv_read = current_conv.read();
            if let Some(ref conv) = *conv_read {
                if conv.messages.is_empty() {
                    // New conversation - show greeting
                    messages.set(vec![Message {
                        role: MessageRole::Assistant,
                        content: "Hello! I'm LocaLM. How can I assist you today?".to_string(),
                    }]);
                } else {
                    // Load existing messages from storage
                    let ui_messages: Vec<Message> = conv.messages.iter()
                        .cloned()
                        .map(|m| m.into())
                        .collect();
                    messages.set(ui_messages);
                }
            }
        });
    }

    // Handler for sending a message
    let handle_send = {
        let mut messages = messages.clone();
        let mut is_generating = is_generating.clone();
        let app_state = app_state.clone();
        move |text: String| {
            if !matches!(*app_state.model_state.read(), ModelState::Loaded(_)) {
                messages.write().push(Message {
                    role: MessageRole::Assistant,
                    content: "Model not loaded. Please select and load a model first.".to_string(),
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
            let mut app_state = app_state.clone();

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
                            // Small delay to allow UI to repaint between token updates
                            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                        }
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
                    }
                }

                is_generating.set(false);
                
                // Save messages to conversation after generation completes
                {
                    let msgs = messages.read();
                    let storage_messages: Vec<StorageMessage> = msgs.iter()
                        .cloned()
                        .map(|m| m.into())
                        .collect();
                    
                    let mut conv_write = app_state.current_conversation.write();
                    if let Some(ref mut conv) = *conv_write {
                        conv.messages = storage_messages;
                        if let Err(e) = save_conversation(conv) {
                            tracing::error!("Failed to save conversation: {}", e);
                        }
                    }
                }
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
            
            // Messages Area
            div { class: "flex-1 overflow-y-auto px-4 py-6 space-y-6 custom-scrollbar scroll-smooth",
                div { class: "max-w-4xl mx-auto w-full flex flex-col space-y-6 pb-4",
                    // Message List
                    for (idx, msg) in messages.read().iter().enumerate() {
                        MessageBubble { key: "{idx}", message: msg.clone() }
                    }
                    
                    // Typing / Generating Indicator
                    if is_generating() {
                        div { class: "flex items-center gap-2 text-[var(--text-tertiary)] text-sm ml-12 animate-fade-in",
                            div { class: "w-1.5 h-1.5 bg-[var(--accent-primary)] rounded-full animate-bounce" }
                            div { class: "w-1.5 h-1.5 bg-[var(--accent-primary)] rounded-full animate-bounce delay-75" }
                            div { class: "w-1.5 h-1.5 bg-[var(--accent-primary)] rounded-full animate-bounce delay-150" }
                        }
                    }
                    
                    div { class: "h-8" } // Spacer
                }
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
