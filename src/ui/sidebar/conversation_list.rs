use dioxus::prelude::*;

use crate::app::AppState;
use crate::storage::conversations::{
    delete_conversation, list_conversations, save_conversation, Conversation,
};

#[component]
pub fn ConversationList() -> Element {
    let app_state = use_context::<AppState>();
    let mut conversations_signal = app_state.conversations.clone();

    use_effect(move || match list_conversations() {
        Ok(conversations) => conversations_signal.set(conversations),
        Err(e) => tracing::error!("Failed to load conversations: {}", e),
    });

    let handle_new = {
        let mut conversations_signal = app_state.conversations.clone();
        let mut current_conversation_signal = app_state.current_conversation.clone();
        move |_| {
            let conversation = Conversation::new(None);
            if let Err(e) = save_conversation(&conversation) {
                tracing::error!("Failed to save conversation: {}", e);
                return;
            }
            current_conversation_signal.set(Some(conversation));
            if let Ok(conversations) = list_conversations() {
                conversations_signal.set(conversations);
            }
        }
    };

    let conversations = app_state.conversations.read().clone();
    let selected_id = app_state
        .current_conversation
        .read()
        .as_ref()
        .map(|conv| conv.id.clone());

    rsx! {
        div {
            class: "flex-1 overflow-y-auto px-2 py-2 space-y-1",
            style: "scrollbar-width: thin;",

            button {
                onclick: handle_new,
                class: "group flex w-full items-center gap-3 px-3 py-3 text-sm rounded-md transition-colors duration-200 text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]",
                div {
                    class: "shrink-0",
                    svg {
                        width: "16",
                        height: "16",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        path { d: "M12 5v14M5 12h14" }
                    }
                }
                span { class: "truncate", "New Conversation" }
            }

            if conversations.is_empty() {
                div {
                    class: "px-3 py-4 text-xs text-[var(--text-tertiary)]",
                    "No conversations"
                }
            } else {
                {conversations.into_iter().map(|conversation| {
                    let is_selected = selected_id
                        .as_ref()
                        .map(|id| id == &conversation.id)
                        .unwrap_or(false);
                    let row_class = if is_selected {
                        "group flex items-center gap-3 px-3 py-3 text-sm rounded-md cursor-pointer transition-colors duration-200 bg-[var(--bg-hover)] text-[var(--text-primary)]"
                    } else {
                        "group flex items-center gap-3 px-3 py-3 text-sm rounded-md cursor-pointer transition-colors duration-200 hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
                    };
                    let conversation_for_select = conversation.clone();
                    let conversation_id = conversation.id.clone();
                    let mut current_conversation_signal = app_state.current_conversation.clone();
                    let mut conversations_signal = app_state.conversations.clone();

                    rsx! {
                        div {
                            key: "{conversation.id}",
                            class: "group relative",
                            onclick: move |_| {
                                current_conversation_signal.set(Some(conversation_for_select.clone()));
                            },

                            div {
                                class: row_class,
                                // Icon
                                div {
                                    class: "shrink-0",
                                    svg {
                                        width: "16",
                                        height: "16",
                                        view_box: "0 0 24 24",
                                        fill: "none",
                                        stroke: "currentColor",
                                        stroke_width: "2",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        path { d: "M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" }
                                    }
                                }

                                // Title
                                div {
                                    class: "truncate flex-1",
                                    "{conversation.title}"
                                }

                                button {
                                    class: "opacity-0 group-hover:opacity-100 transition-opacity text-[var(--text-tertiary)] hover:text-[var(--text-primary)]",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        if let Err(e) = delete_conversation(&conversation_id) {
                                            tracing::error!("Failed to delete conversation: {}", e);
                                        }
                                        let should_clear = current_conversation_signal
                                            .read()
                                            .as_ref()
                                            .map(|conv| conv.id == conversation_id)
                                            .unwrap_or(false);
                                        if should_clear {
                                            current_conversation_signal.set(None);
                                        }
                                        if let Ok(conversations) = list_conversations() {
                                            conversations_signal.set(conversations);
                                        }
                                    },
                                    svg {
                                        width: "14",
                                        height: "14",
                                        view_box: "0 0 24 24",
                                        fill: "none",
                                        stroke: "currentColor",
                                        stroke_width: "2",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        path { d: "M3 6h18" }
                                        path { d: "M8 6v12" }
                                        path { d: "M16 6v12" }
                                        path { d: "M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" }
                                        path { d: "M10 6V4a2 2 0 0 1 2-2h0a2 2 0 0 1 2 2v2" }
                                    }
                                }
                            }
                        }
                    }
                })}
            }
        }
    }
}
