//! WebSocket handler
//!
//! Real-time bidirectional communication with the mobile app.
//! Streams tokens, tool permission requests, and system updates.

use crate::server::ServerState;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};

/// WebSocket upgrade handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<ServerState>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle an individual WebSocket connection
async fn handle_socket(socket: WebSocket, state: ServerState) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to broadcast events
    let mut rx = state.ws_tx.subscribe();

    // Send connected confirmation
    let connected_msg = serde_json::to_string(&crate::server::WsEvent::Connected)
        .unwrap_or_default();
    let _ = sender.send(Message::Text(connected_msg.into())).await;

    tracing::info!("Mobile WebSocket client connected");

    // Spawn a task to forward broadcast events to this client
    let send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            let json = match serde_json::to_string(&event) {
                Ok(j) => j,
                Err(_) => continue,
            };

            if sender.send(Message::Text(json.into())).await.is_err() {
                break; // Client disconnected
            }
        }
    });

    // Handle incoming messages from mobile (e.g., tool approvals)
    let state_clone = state.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    handle_ws_message(&text, &state_clone).await;
                }
                Message::Close(_) => {
                    tracing::info!("Mobile WebSocket client disconnected");
                    break;
                }
                _ => {}
            }
        }
    });

    // Wait for either task to complete (client disconnect)
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    tracing::info!("Mobile WebSocket connection closed");
}

/// Handle a message received from the mobile app via WebSocket
async fn handle_ws_message(text: &str, state: &ServerState) {
    // Parse incoming JSON commands
    if let Ok(cmd) = serde_json::from_str::<serde_json::Value>(text) {
        let action = cmd.get("action").and_then(|a| a.as_str()).unwrap_or("");

        match action {
            "approve_tool" => {
                if let Some(id) = cmd.get("id").and_then(|i| i.as_str()) {
                    if let Some((_, approval)) = state.pending_approvals.remove(id) {
                        if let Some(tx) = approval.response_tx.lock().await.take() {
                            let _ = tx.send(true);
                        }
                    }
                }
            }
            "deny_tool" => {
                if let Some(id) = cmd.get("id").and_then(|i| i.as_str()) {
                    if let Some((_, approval)) = state.pending_approvals.remove(id) {
                        if let Some(tx) = approval.response_tx.lock().await.take() {
                            let _ = tx.send(false);
                        }
                    }
                }
            }
            "ping" => {
                // Keep-alive, no action needed
            }
            _ => {
                tracing::debug!("Unknown WebSocket action: {}", action);
            }
        }
    }
}
