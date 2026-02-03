use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    // We could add timestamp, id, etc. later
}

// Convert storage Message to UI Message
impl From<crate::types::message::Message> for Message {
    fn from(msg: crate::types::message::Message) -> Self {
        Message {
            role: match msg.role {
                crate::types::message::Role::User => MessageRole::User,
                crate::types::message::Role::Assistant => MessageRole::Assistant,
                crate::types::message::Role::System => MessageRole::System,
            },
            content: msg.content,
        }
    }
}

// Convert UI Message to storage Message
impl From<Message> for crate::types::message::Message {
    fn from(msg: Message) -> Self {
        crate::types::message::Message::new(
            match msg.role {
                MessageRole::User => crate::types::message::Role::User,
                MessageRole::Assistant => crate::types::message::Role::Assistant,
                MessageRole::System => crate::types::message::Role::System,
            },
            msg.content,
        )
    }
}

// Content parts for parsed message content
#[derive(Clone, PartialEq, Debug)]
enum ContentPart {
    Text(String),
    Thinking(String),
}

/// Parse <think>...</think> blocks from message content
fn parse_thinking_blocks(content: &str) -> Vec<ContentPart> {
    let mut parts = Vec::new();
    let mut remaining = content;

    while let Some(start) = remaining.find("<think>") {
        // Add text before <think>
        if start > 0 {
            let text = remaining[..start].to_string();
            if !text.trim().is_empty() {
                parts.push(ContentPart::Text(text));
            }
        }

        // Find closing tag
        if let Some(end_offset) = remaining[start..].find("</think>") {
            let think_start = start + 7; // length of "<think>"
            let think_end = start + end_offset;
            let think_content = remaining[think_start..think_end].to_string();
            if !think_content.trim().is_empty() {
                parts.push(ContentPart::Thinking(think_content));
            }
            remaining = &remaining[think_end + 8..]; // length of "</think>"
        } else {
            // No closing tag, treat rest as text
            parts.push(ContentPart::Text(remaining[start..].to_string()));
            remaining = "";
        }
    }

    // Add remaining text
    if !remaining.is_empty() {
        parts.push(ContentPart::Text(remaining.to_string()));
    }

    // If no parts found, return the whole content as text
    if parts.is_empty() {
        parts.push(ContentPart::Text(content.to_string()));
    }

    parts
}

/// Collapsible thinking block component
#[component]
fn ThinkingBlock(content: String) -> Element {
    let mut is_expanded = use_signal(|| false);

    let chevron_class = if is_expanded() {
        "thinking-chevron expanded"
    } else {
        "thinking-chevron"
    };

    let content_class = if is_expanded() {
        "thinking-content expanded"
    } else {
        "thinking-content"
    };

    rsx! {
        div { class: "thinking-block",
            // Clickable header
            div {
                class: "thinking-header",
                onclick: move |_| is_expanded.set(!is_expanded()),

                // Chevron icon (right-pointing arrow)
                svg {
                    class: "{chevron_class}",
                    width: "16",
                    height: "16",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    polyline { points: "9 18 15 12 9 6" }
                }

                span { "Thinking..." }
            }

            // Collapsible content
            div {
                class: "{content_class}",
                div {
                    class: "whitespace-pre-wrap break-words",
                    "{content}"
                }
            }
        }
    }
}

#[component]
pub fn MessageBubble(message: Message) -> Element {
    let is_user = message.role == MessageRole::User;

    // Minimalist: No container background for AI, distinct bubble for User
    let container_class = if is_user {
        "flex flex-row-reverse items-start gap-4 animate-fade-in-up"
    } else {
        "flex flex-row items-start gap-4 animate-fade-in"
    };

    let bubble_class = if is_user {
        "bg-[var(--accent-primary)] text-[var(--accent-text)] rounded-2xl rounded-tr-md px-5 py-3 shadow-sm max-w-[85%] leading-relaxed text-base selection:bg-white/20"
    } else {
        // AI: Clean text, no bubble background, just pure typography
        "text-[var(--text-primary)] px-1 py-1 max-w-[90%] leading-relaxed text-base font-normal selection:bg-[var(--accent-subtle)]"
    };

    // Parse content for AI messages only
    let content_parts = if !is_user {
        parse_thinking_blocks(&message.content)
    } else {
        vec![ContentPart::Text(message.content.clone())]
    };

    rsx! {
        div { class: "{container_class}",
            // Avatar
            div {
                class: "flex-shrink-0 mt-1 select-none",
                div {
                    class: "w-8 h-8 rounded-lg flex items-center justify-center shadow-sm text-xs font-bold transition-transform hover:scale-105 " .to_string() +
                    if is_user { "bg-[var(--bg-surface)] text-[var(--text-primary)] border border-[var(--border-subtle)]" } else { "bg-gradient-to-br from-[var(--accent-primary)] to-[var(--accent-hover)] text-white shadow-glow" },

                    if is_user {
                        svg { width: "16", height: "16", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2", path { d: "M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" }, circle { cx: "12", cy: "7", r: "4" } }
                    } else {
                        svg { width: "18", height: "18", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2", path { d: "M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" } }
                    }
                }
            }

            // Message Content
            div {
                class: "flex flex-col " .to_string() + if is_user { "items-end" } else { "items-start" },

                div {
                    class: "{bubble_class}",
                    // Render parsed content parts
                    for part in content_parts {
                        match part {
                            ContentPart::Thinking(text) => rsx! {
                                ThinkingBlock { content: text }
                            },
                            ContentPart::Text(text) => rsx! {
                                div {
                                    class: "whitespace-pre-wrap break-words",
                                    "{text}"
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}
