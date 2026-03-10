#![allow(non_snake_case)]

//! Mobile Companion settings tab
//!
//! Shows a QR code for instant pairing and connection info.

use dioxus::prelude::*;
use crate::app::AppState;
use crate::server;

pub fn MobileSettings() -> Element {
    let app_state = use_context::<AppState>();
    let is_en = app_state.settings.read().language == "en";

    let local_ip = server::get_cached_local_ip();
    let mut pairing_code = use_signal(|| server::get_pairing_code());
    let port = server::DEFAULT_PORT;

    // Generate QR code data URL
    let qr_svg = use_memo(move || {
        let code = pairing_code();
        let ip = server::get_cached_local_ip();
        let payload = format!("clawrs://{}:{}/{}", ip, port, code);
        generate_qr_svg(&payload)
    });

    rsx! {
        div {
            class: "max-w-3xl mx-auto w-full space-y-6",

            // Title
            h2 {
                class: "text-lg font-semibold mb-2",
                style: "color: var(--text-primary);",
                if is_en { "Mobile Companion" } else { "Application Mobile" }
            }
            p {
                class: "text-sm mb-6",
                style: "color: var(--text-secondary);",
                if is_en {
                    "Scan the QR code below with the ClawRS mobile app to connect instantly."
                } else {
                    "Scannez le code QR ci-dessous avec l'application mobile ClawRS pour vous connecter instantanément."
                }
            }

            // QR Code Card
            div {
                class: "rounded-xl p-6 flex flex-col items-center",
                style: "background: rgba(242,237,231,0.04); border: 1px solid rgba(242,237,231,0.07);",

                // Status badge
                div {
                    class: "flex items-center gap-2 mb-5",
                    div {
                        class: "w-2 h-2 rounded-full",
                        style: "background: var(--success);",
                    }
                    span {
                        class: "text-xs font-medium",
                        style: "color: var(--success);",
                        if is_en { "Server Running" } else { "Serveur Actif" }
                    }
                }

                // QR Code
                div {
                    class: "rounded-xl p-4 mb-4",
                    style: "background: white;",
                    div {
                        dangerous_inner_html: "{qr_svg}",
                    }
                }

                // Connection details underneath
                div {
                    class: "text-center space-y-2 w-full",

                    // IP & Port
                    div {
                        class: "flex items-center justify-center gap-2",
                        span {
                            class: "text-xs font-medium uppercase tracking-widest",
                            style: "color: var(--text-tertiary);",
                            "IP"
                        }
                        span {
                            class: "text-sm font-mono font-semibold",
                            style: "color: var(--text-primary);",
                            "{local_ip}:{port}"
                        }
                    }

                    // Code
                    div {
                        class: "flex items-center justify-center gap-2",
                        span {
                            class: "text-xs font-medium uppercase tracking-widest",
                            style: "color: var(--text-tertiary);",
                            "CODE"
                        }
                        span {
                            class: "text-lg font-bold font-mono",
                            style: "color: var(--accent-primary); letter-spacing: 0.2em;",
                            "{pairing_code}"
                        }
                    }
                }

                // Regenerate button
                button {
                    class: "mt-4 rounded-lg px-4 py-2 text-xs font-medium transition-all hover:opacity-80 flex items-center gap-2",
                    style: "background: rgba(242,237,231,0.06); border: 1px solid rgba(242,237,231,0.08); color: var(--text-secondary);",
                    onclick: move |_| {
                        let new_code = server::regenerate_pairing_code();
                        pairing_code.set(new_code);
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
                        path { d: "M23 4v6h-6" }
                        path { d: "M1 20v-6h6" }
                        path { d: "M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" }
                    }
                    if is_en { "New Code" } else { "Nouveau Code" }
                }
            }

            // Instructions
            div {
                class: "rounded-xl p-5 mt-2",
                style: "background: rgba(139, 38, 53, 0.06); border: 1px solid rgba(139, 38, 53, 0.12);",

                h3 {
                    class: "text-sm font-semibold mb-3",
                    style: "color: var(--text-primary);",
                    if is_en { "How to connect" } else { "Comment se connecter" }
                }
                ol {
                    class: "space-y-2 text-sm list-decimal list-inside",
                    style: "color: var(--text-secondary);",
                    li {
                        if is_en {
                            "Open ClawRS Mobile on your phone"
                        } else {
                            "Ouvrez ClawRS Mobile sur votre téléphone"
                        }
                    }
                    li {
                        if is_en {
                            "Tap \"Scan QR Code\" on the pairing screen"
                        } else {
                            "Appuyez sur \"Scanner le QR Code\" sur l'écran d'appairage"
                        }
                    }
                    li {
                        if is_en {
                            "Point your phone camera at the QR code above"
                        } else {
                            "Pointez la caméra de votre téléphone vers le QR code ci-dessus"
                        }
                    }
                }
            }

            // Firewall notice
            div {
                class: "rounded-xl p-4 mt-2",
                style: "background: rgba(196, 153, 59, 0.06); border: 1px solid rgba(196, 153, 59, 0.12);",

                div {
                    class: "flex items-start gap-3",
                    span { class: "text-base", "⚠️" }
                    div {
                        p {
                            class: "text-sm font-medium mb-1",
                            style: "color: var(--warning);",
                            if is_en { "Troubleshooting" } else { "Dépannage" }
                        }
                        p {
                            class: "text-xs",
                            style: "color: var(--text-secondary);",
                            if is_en {
                                "If connecting fails, ensure both devices are on the same local network. Some routers isolate Wi-Fi and Ethernet — check your router's \"AP isolation\" or \"Client isolation\" setting."
                            } else {
                                "Si la connexion échoue, vérifiez que les deux appareils sont sur le même réseau local. Certains routeurs isolent le Wi-Fi et l'Ethernet — vérifiez le paramètre \"Isolation AP\" de votre routeur."
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Generate an inline SVG string for a QR code
fn generate_qr_svg(data: &str) -> String {
    use qrcode::QrCode;
    use qrcode::render::svg;

    match QrCode::new(data) {
        Ok(code) => {
            code.render::<svg::Color<'_>>()
                .min_dimensions(200, 200)
                .max_dimensions(200, 200)
                .dark_color(svg::Color("#171614"))
                .light_color(svg::Color("#ffffff"))
                .build()
        }
        Err(e) => {
            tracing::error!("Failed to generate QR code: {}", e);
            format!("<svg width=\"200\" height=\"200\"><text x=\"50\" y=\"100\" fill=\"red\">QR Error</text></svg>")
        }
    }
}
