//! ClawRS - Local LLM Chat Application
//!
//! A desktop application for running local Large Language Models with a beautiful GUI.

use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use clawrs::app::App;

fn main() {
    // Initialize tracing subscriber for logging
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive("clawrs=info".parse().unwrap()))
        .init();

    info!("Starting ClawRS v{}", env!("CARGO_PKG_VERSION"));

    // Initialize storage directory structure
    if let Err(e) = clawrs::storage::init_storage() {
        tracing::error!("Failed to initialize storage: {}", e);
    }

    // Ensure Windows Firewall allows the mobile companion server port
    ensure_firewall_rule();

    // Spawn the mobile companion server in a background thread
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime for server");
        rt.block_on(async {
            let server_state = clawrs::server::ServerState::new();

            // Generate initial pairing code
            let code = server_state.generate_pairing_code().await;
            info!("Mobile companion server pairing code: {}", code);

            if let Err(e) = clawrs::server::start_server(
                server_state,
                clawrs::server::DEFAULT_PORT,
            ).await {
                tracing::error!("Mobile companion server error: {}", e);
            }
        });
    });

    // Launch Dioxus desktop application
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            Config::default()
                .with_menu(None) // Remove the default menu bar
                .with_window(
                    WindowBuilder::new()
                        .with_title("ClawRS")
                        .with_inner_size(LogicalSize::new(1200.0, 800.0)),
                ),
        )
        .launch(App);
}

/// Ensure the Windows Firewall has rules allowing the mobile server ports.
/// On first launch, triggers a UAC prompt asking for admin permission.
fn ensure_firewall_rule() {
    use std::process::Command;
    #[cfg(target_os = "windows")]
    use std::os::windows::process::CommandExt;

    let tcp_port = clawrs::server::DEFAULT_PORT;
    let udp_port = clawrs::server::DISCOVERY_PORT;
    let rule_name_tcp = "ClawRS Mobile Companion";
    let rule_name_udp = "ClawRS Mobile Discovery";

    // Check if TCP rule already exists
    let check = Command::new("netsh")
        .args([
            "advfirewall", "firewall", "show", "rule",
            &format!("name={}", rule_name_tcp),
        ])
        .creation_flags(0x08000000)
        .output();

    let tcp_exists = matches!(check, Ok(output) if output.status.success());

    // Check if UDP rule already exists
    let check_udp = Command::new("netsh")
        .args([
            "advfirewall", "firewall", "show", "rule",
            &format!("name={}", rule_name_udp),
        ])
        .creation_flags(0x08000000)
        .output();

    let udp_exists = matches!(check_udp, Ok(output) if output.status.success());

    if tcp_exists && udp_exists {
        info!("Firewall rules already exist");
        return;
    }

    info!("Requesting admin permission to add firewall rules...");

    // Build a PowerShell script that adds both rules in one UAC prompt
    let mut cmds = Vec::new();
    if !tcp_exists {
        cmds.push(format!(
            "netsh advfirewall firewall add rule name=\"{}\" dir=in action=allow protocol=TCP localport={}",
            rule_name_tcp, tcp_port
        ));
    }
    if !udp_exists {
        cmds.push(format!(
            "netsh advfirewall firewall add rule name=\"{}\" dir=in action=allow protocol=UDP localport={}",
            rule_name_udp, udp_port
        ));
    }

    let script = cmds.join("; ");

    let result = Command::new("powershell")
        .args([
            "-WindowStyle", "Hidden",
            "-Command",
            &format!(
                "Start-Process powershell -ArgumentList '-WindowStyle Hidden -Command \"{}\"' -Verb RunAs -Wait -WindowStyle Hidden",
                script.replace('"', "`\"")
            ),
        ])
        .creation_flags(0x08000000)
        .output();

    match result {
        Ok(output) if output.status.success() => {
            info!("Firewall rules added successfully (TCP:{}, UDP:{})", tcp_port, udp_port);
        }
        Ok(_) => {
            tracing::warn!("User declined or firewall rules could not be added");
        }
        Err(e) => {
            tracing::warn!("Failed to request firewall permission: {}", e);
        }
    }
}


