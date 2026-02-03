use dioxus::prelude::*;

#[component]
pub fn ChatInput(
    on_send: EventHandler<String>,
    on_stop: EventHandler<()>,
    is_generating: bool,
) -> Element {
    let mut text = use_signal(|| String::new());

    // Auto-resize logic would go here in a full implementation,
    // for now we'll rely on CSS or fixed height with scroll

    let handle_keydown = move |evt: KeyboardEvent| {
        if evt.key() == Key::Escape && is_generating {
            on_stop.call(());
        } else if evt.key() == Key::Enter
            && (evt.modifiers().contains(Modifiers::CONTROL)
                || evt.modifiers().contains(Modifiers::META))
        {
            if !is_generating && !text().trim().is_empty() {
                on_send.call(text());
                text.set(String::new());
            }
        }
    };

    let handle_send_click = move |_| {
        if !is_generating && !text().trim().is_empty() {
            on_send.call(text());
            text.set(String::new());
        }
    };

    rsx! {
        div {
            class: "w-full p-4 border-t border-[var(--border-subtle)] bg-[var(--bg-main)]",

            div {
                class: "relative flex items-end gap-2 max-w-4xl mx-auto",

                // Textarea container
                div {
                    class: "relative flex-1 bg-[var(--bg-input)] rounded-xl border border-[var(--border-subtle)] focus-within:border-[var(--border-focus)] focus-within:ring-1 focus-within:ring-[var(--border-focus)] transition-all shadow-sm",

                    textarea {
                        class: "w-full max-h-48 min-h-[56px] py-3 px-4 bg-transparent border-none outline-none text-[var(--text-primary)] resize-none placeholder-[var(--text-tertiary)] rounded-xl font-sans",
                        placeholder: "Type a message...",
                        value: "{text}",
                        oninput: move |evt| text.set(evt.value()),
                        onkeydown: handle_keydown,
                        disabled: is_generating,
                    }
                }

                // Send / Stop Button
                div {
                    class: "flex-shrink-0",
                    if is_generating {
                        button {
                            onclick: move |_| on_stop.call(()),
                            class: "p-3 rounded-xl bg-[var(--bg-active)] text-[var(--text-secondary)] hover:bg-red-500/10 hover:text-red-500 transition-colors border border-[var(--border-subtle)] hover:border-red-500/30",
                            title: "Stop generating (Esc)",
                            svg { width: "20", height: "20", view_box: "0 0 24 24", fill: "currentColor", rect { x: "6", y: "6", width: "12", height: "12", rx: "2" } }
                        }
                    } else {
                        button {
                            onclick: handle_send_click,
                            disabled: text().trim().is_empty(),
                            class: "p-3 rounded-xl bg-[var(--accent-primary)] text-[var(--accent-text)] hover:bg-[var(--accent-hover)] disabled:opacity-50 disabled:cursor-not-allowed transition-all shadow-md active:scale-95",
                            title: "Send message (Ctrl + Enter)",
                            svg { width: "20", height: "20", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round", line { x1: "22", y1: "2", x2: "11", y2: "13" }, polygon { points: "22 2 15 22 11 13 2 9 22 2" } }
                        }
                    }
                }
            }

            // Footer text
            div {
                class: "text-center mt-2 text-xs text-[var(--text-tertiary)]",
                "LocaLM can make mistakes. Consider checking important information."
            }
        }
    }
}
