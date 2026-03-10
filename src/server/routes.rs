//! REST API routes
//!
//! All HTTP endpoints for the mobile companion app.

use crate::server::{ServerModelState, ServerState, WsEvent};
use crate::storage::conversations;
use crate::storage::models::scan_models_directory;
use crate::storage::settings::{load_settings, save_settings};
use crate::system::gpu::{detect_gpu, get_total_vram_gb};
use crate::system::resources::get_resource_usage;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    middleware,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

/// Build the API router with all endpoints
pub fn api_routes(state: ServerState) -> Router<ServerState> {
    Router::new()
        // Public endpoint (no auth)
        .route("/pair", post(pair))
        // Protected endpoints
        .route("/status", get(status))
        .route("/models", get(list_models))
        .route("/models/load", post(load_model))
        .route("/models/unload", post(unload_model))
        .route("/conversations", get(list_convos))
        .route("/conversations", post(create_conversation))
        .route("/conversations/{id}", get(get_conversation))
        .route("/conversations/{id}", delete(delete_conversation))
        .route("/chat", post(send_chat))
        .route("/chat/stop", post(stop_chat))
        .route("/settings", get(get_settings))
        .route("/settings", put(update_settings))
        .route("/system", get(system_info))
        .route("/tools/approve/{id}", post(approve_tool))
        .route("/tools/deny/{id}", post(deny_tool))
        .route("/ws", get(crate::server::ws::ws_handler))
        .route_layer(middleware::from_fn_with_state(
            state,
            crate::server::auth::auth_middleware,
        ))
}

// =============================================================================
// Request/Response types
// =============================================================================

#[derive(Deserialize)]
pub struct PairRequest {
    pub code: String,
}

#[derive(Serialize)]
pub struct PairResponse {
    pub token: String,
    pub app_version: String,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub connected: bool,
    pub model_state: String,
    pub model_name: Option<String>,
    pub is_generating: bool,
    pub app_version: String,
}

#[derive(Serialize)]
pub struct ModelListItem {
    pub filename: String,
    pub path: String,
    pub size_bytes: u64,
    pub size_display: String,
}

#[derive(Deserialize)]
pub struct LoadModelRequest {
    pub path: String,
}

#[derive(Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub conversation_id: Option<String>,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub conversation_id: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct SystemInfoResponse {
    pub ram_used_mb: u64,
    pub ram_total_mb: u64,
    pub gpu_name: String,
    pub gpu_vram_gb: f64,
}

// =============================================================================
// Handler implementations
// =============================================================================

/// POST /api/pair — Exchange pairing code for session token
async fn pair(
    State(state): State<ServerState>,
    Json(body): Json<PairRequest>,
) -> Result<Json<PairResponse>, StatusCode> {
    // Check both the server's internal code and the global shared code
    // (the UI's regenerate button updates the global code)
    let internal_code = state.pairing_code.lock().await.clone();
    let global_code = crate::server::get_pairing_code();

    let code_matches = match &internal_code {
        Some(code) if code == &body.code => true,
        _ => global_code == body.code && global_code != "------",
    };

    if code_matches {
        let token = uuid::Uuid::new_v4().to_string();
        state
            .session_tokens
            .insert(token.clone(), std::time::Instant::now());
        tracing::info!("Mobile device paired successfully");

        Ok(Json(PairResponse {
            token,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
        }))
    } else {
        tracing::warn!("Invalid pairing code attempt: {}", body.code);
        Err(StatusCode::UNAUTHORIZED)
    }
}

/// GET /api/status — Get app status
async fn status(State(state): State<ServerState>) -> Json<StatusResponse> {
    let model_state = state.model_state.lock().await;
    let is_generating = state.is_generating.load(std::sync::atomic::Ordering::Relaxed);

    let (state_str, model_name) = match &*model_state {
        ServerModelState::NotLoaded => ("not_loaded".to_string(), None),
        ServerModelState::Loading => ("loading".to_string(), None),
        ServerModelState::Loaded(name) => ("loaded".to_string(), Some(name.clone())),
        ServerModelState::Error(e) => (format!("error: {}", e), None),
    };

    Json(StatusResponse {
        connected: true,
        model_state: state_str,
        model_name,
        is_generating,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// GET /api/models — List available models
async fn list_models() -> Json<Vec<ModelListItem>> {
    let settings = load_settings();
    let models_dir = settings.models_directory.clone();

    match scan_models_directory(&models_dir) {
        Ok(models) => Json(
            models
                .iter()
                .map(|m| ModelListItem {
                    filename: m.filename.clone(),
                    path: m.path.to_string_lossy().to_string(),
                    size_bytes: m.size_bytes,
                    size_display: m.size_string(),
                })
                .collect(),
        ),
        Err(e) => {
            tracing::error!("Failed to scan models: {}", e);
            Json(vec![])
        }
    }
}

/// POST /api/models/load — Load a model
async fn load_model(
    State(state): State<ServerState>,
    Json(body): Json<LoadModelRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let settings = load_settings();
    let gpu_layers = settings.gpu_layers;

    {
        let mut ms = state.model_state.lock().await;
        *ms = ServerModelState::Loading;
    }

    let _ = state.ws_tx.send(WsEvent::ModelStateChanged {
        state: "loading".to_string(),
        model_name: None,
    });

    let mut engine = state.engine.lock().await;

    // Initialize if needed
    if !engine.is_initialized() {
        if let Err(e) = engine.init() {
            tracing::error!("Failed to init engine: {}", e);
            let mut ms = state.model_state.lock().await;
            *ms = ServerModelState::Error(e.to_string());
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    match engine.load_model_async(&body.path, gpu_layers).await {
        Ok(info) => {
            let model_name = std::path::Path::new(&body.path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            {
                let mut ms = state.model_state.lock().await;
                *ms = ServerModelState::Loaded(model_name.clone());
            }

            let _ = state.ws_tx.send(WsEvent::ModelStateChanged {
                state: "loaded".to_string(),
                model_name: Some(model_name),
            });

            Ok(Json(serde_json::json!({
                "status": "loaded",
                "size_bytes": info.size_bytes,
                "context_length": info.context_length,
            })))
        }
        Err(e) => {
            tracing::error!("Failed to load model: {}", e);
            let mut ms = state.model_state.lock().await;
            *ms = ServerModelState::Error(e.to_string());
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// POST /api/models/unload — Unload current model
async fn unload_model(State(state): State<ServerState>) -> Json<serde_json::Value> {
    let mut engine = state.engine.lock().await;
    engine.unload_model();

    {
        let mut ms = state.model_state.lock().await;
        *ms = ServerModelState::NotLoaded;
    }

    let _ = state.ws_tx.send(WsEvent::ModelStateChanged {
        state: "not_loaded".to_string(),
        model_name: None,
    });

    Json(serde_json::json!({ "status": "unloaded" }))
}

/// GET /api/conversations — List all conversations
async fn list_convos() -> Json<serde_json::Value> {
    match conversations::list_conversations() {
        Ok(convos) => {
            let items: Vec<serde_json::Value> = convos
                .iter()
                .map(|c| {
                    serde_json::json!({
                        "id": c.id,
                        "title": c.title,
                        "message_count": c.messages.len(),
                        "created_at": c.created_at.to_rfc3339(),
                        "updated_at": c.updated_at.to_rfc3339(),
                    })
                })
                .collect();
            Json(serde_json::json!(items))
        }
        Err(e) => {
            tracing::error!("Failed to list conversations: {}", e);
            Json(serde_json::json!([]))
        }
    }
}

/// POST /api/conversations — Create a new conversation
async fn create_conversation() -> Json<serde_json::Value> {
    let conv = conversations::Conversation::new(None);
    if let Err(e) = conversations::save_conversation(&conv) {
        tracing::error!("Failed to save conversation: {}", e);
    }
    Json(serde_json::json!({
        "id": conv.id,
        "title": conv.title,
    }))
}

/// GET /api/conversations/:id — Get a conversation with messages
async fn get_conversation(Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    match conversations::load_conversation(&id) {
        Ok(conv) => Ok(Json(serde_json::to_value(&conv).unwrap_or_default())),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

/// DELETE /api/conversations/:id — Delete a conversation
async fn delete_conversation(Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    match conversations::delete_conversation(&id) {
        Ok(_) => Ok(Json(serde_json::json!({ "deleted": true }))),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

/// POST /api/chat — Send a chat message and start generation
async fn send_chat(
    State(state): State<ServerState>,
    Json(body): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, StatusCode> {
    use crate::inference::engine::GenerationParams;
    use crate::inference::streaming::StreamToken;
    use crate::types::message::{Message, Role};

    // Check if a model is loaded
    {
        let ms = state.model_state.lock().await;
        if !matches!(&*ms, ServerModelState::Loaded(_)) {
            return Err(StatusCode::PRECONDITION_FAILED);
        }
    }

    // Check if already generating
    if state.is_generating.load(std::sync::atomic::Ordering::Relaxed) {
        return Err(StatusCode::CONFLICT);
    }

    // Get or create conversation
    let conv_id = match body.conversation_id {
        Some(id) => {
            // Add user message to existing conversation
            if let Ok(mut conv) = conversations::load_conversation(&id) {
                conv.add_message(Message::new(Role::User, &body.message));
                let _ = conversations::save_conversation(&conv);
            }
            id
        }
        None => {
            let conv = conversations::Conversation::new(Some(Message::new(Role::User, &body.message)));
            let id = conv.id.clone();
            let _ = conversations::save_conversation(&conv);
            id
        }
    };

    // Initial system prompt will be built with tools if needed
    let settings = load_settings();
    let agent = crate::server::get_shared_agent();
    
    // Build messages for the engine
    let mut history_messages = vec![];
    if let Ok(conv) = conversations::load_conversation(&conv_id) {
        for msg in &conv.messages {
            history_messages.push(msg.clone());
        }
    }

    // Start generation in background
    let ws_tx = state.ws_tx.clone();
    let engine = state.engine.clone();
    let is_generating = state.is_generating.clone();
    let conv_id_clone = conv_id.clone();
    let agent_clone = agent.clone();
    let max_iterations = agent.config.loop_config.max_iterations;
    let enable_tools = agent.config.enable_tools;

    tokio::task::spawn_blocking(move || {
        is_generating.store(true, std::sync::atomic::Ordering::SeqCst);
        
        // Agent loop state (mirrors desktop AgentContext)
        let mut iteration = 0usize;
        let mut consecutive_errors = 0u32;
        let mut continuous_history = history_messages;
        let loop_start = std::time::Instant::now();
        let tool_timeout_secs = agent_clone.config.tool_timeout_secs;

        while iteration < max_iterations {
            iteration += 1;

            // Check stop signal
            if state.stop_signal.load(std::sync::atomic::Ordering::SeqCst) {
                tracing::info!("[mobile] Agent stopped by user at iteration {}", iteration);
                break;
            }

            // Max runtime guard (5 minutes)
            if loop_start.elapsed().as_secs() > 300 {
                let _ = ws_tx.send(WsEvent::Token(
                    "\n\n⏱️ Temps d'exécution maximal atteint.".to_string()
                ));
                break;
            }

            // Build generation params from settings
            let params = GenerationParams {
                temperature: settings.temperature,
                top_p: settings.top_p,
                top_k: settings.top_k,
                max_tokens: settings.max_tokens,
                max_context_size: settings.context_size,
                ..Default::default()
            };

            // Build system prompt with tool instructions
            let system_prompt = if enable_tools {
                crate::agent::prompts::build_agent_system_prompt(
                    &settings.system_prompt, 
                    &agent_clone.list_tools(), 
                    None, 
                    None
                )
            } else {
                settings.system_prompt.clone()
            };
            
            let mut prompt_messages = vec![Message::new(Role::System, &system_prompt)];
            for msg in &continuous_history {
                prompt_messages.push(msg.clone());
            }

            // Generate
            let result = tokio::runtime::Handle::current().block_on(async {
                let engine_guard = engine.lock().await;
                engine_guard.generate_stream_messages(prompt_messages, params)
            });

            match result {
                Ok((rx, stop_signal)) => {
                    let mut full_response = String::new();

                    while let Ok(token) = rx.recv() {
                        if state.stop_signal.load(std::sync::atomic::Ordering::SeqCst) {
                            stop_signal.store(true, std::sync::atomic::Ordering::Relaxed);
                            break;
                        }
                        
                        match token {
                            StreamToken::Token(t) => {
                                full_response.push_str(&t);
                                let _ = ws_tx.send(WsEvent::Token(t));
                            }
                            StreamToken::Done | StreamToken::Truncated { .. } => {
                                break;
                            }
                            StreamToken::Error(e) => {
                                let _ = ws_tx.send(WsEvent::Error(e.clone()));
                                full_response.push_str(&format!("\n\n❌ Erreur: {}", e));
                                consecutive_errors += 1;
                                break;
                            }
                        }
                    }

                    // Push the assistant response to history
                    if !full_response.is_empty() {
                        continuous_history.push(Message::new(Role::Assistant, &full_response));
                    }

                    // If tools disabled, we're done after first generation
                    if !enable_tools {
                        break;
                    }

                    // Check for stream errors — give LLM a chance to recover
                    if full_response.contains("❌ Erreur:") {
                        if consecutive_errors >= 3 { break; }
                        continuous_history.push(Message::new(
                            Role::System,
                            "Une erreur est survenue. Reformule ta réponse ou essaie une approche différente."
                        ));
                        continue;
                    }
                    consecutive_errors = 0;

                    // Check stop signal after generation
                    if state.stop_signal.load(std::sync::atomic::Ordering::SeqCst) { break; }

                    // Extract tool call from LLM response
                    let tool_call = match crate::agent::extract_tool_call(&full_response) {
                        Some(call) => {
                            tracing::info!("[mobile] Tool call extracted: {}", call.tool);
                            call
                        }
                        None => {
                            // Check if LLM tried but malformed the JSON
                            let looks_like_failed = 
                                (full_response.contains("{\"tool\"") || full_response.contains("{ \"tool\""))
                                && full_response.contains("\"params\"");
                            
                            if looks_like_failed && consecutive_errors < 2 {
                                consecutive_errors += 1;
                                continuous_history.push(Message::new(
                                    Role::System,
                                    "Le format JSON de l'appel d'outil était invalide. Rappel: utilise exactement ce format:\n{\"tool\": \"nom_outil\", \"params\": {...}}\nRéessaie avec le bon format."
                                ));
                                continue;
                            }
                            // Genuine final response — no tool call
                            tracing::info!("[mobile] Final response (no tool call), ending loop");
                            break;
                        }
                    };

                    // Notify the client about tool usage
                    let _ = ws_tx.send(WsEvent::Token(
                        format!("\n\n🔧 Utilisation de l'outil `{}`... (itération {}/{})\n",
                            tool_call.tool, iteration, max_iterations)
                    ));

                    // Mobile auto-approves ALL tools — the phone is just a remote display.
                    // All execution happens on the desktop, just like the desktop app.
                    // The desktop PermissionManager handles its own approval dialogs independently.
                    tracing::info!("[mobile] Auto-approving tool: {}", tool_call.tool);

                    // Retrieve tool from registry
                    let tool = match agent_clone.tool_registry.get(&tool_call.tool) {
                        Some(t) => t,
                        None => {
                            consecutive_errors += 1;
                            let available: Vec<String> = agent_clone.tool_registry.list_tools()
                                .iter().map(|t| t.name.clone()).collect();
                            continuous_history.push(Message::new(
                                Role::System,
                                &format!(
                                    "L'outil '{}' n'existe pas. Outils disponibles: {}.",
                                    tool_call.tool, available.join(", ")
                                )
                            ));
                            if consecutive_errors >= 3 { break; }
                            continue;
                        }
                    };

                    // Execute tool with timeout
                    tracing::info!("[mobile] Executing tool: {} (timeout: {}s)", tool_call.tool, tool_timeout_secs);
                    let tool_result = tokio::runtime::Handle::current().block_on(async {
                        match tokio::time::timeout(
                            std::time::Duration::from_secs(tool_timeout_secs),
                            tool.execute(tool_call.params.clone()),
                        ).await {
                            Ok(Ok(result)) => Ok(result),
                            Ok(Err(e)) => Err(e.to_string()),
                            Err(_) => Err("Timeout dépassé".to_string()),
                        }
                    });

                    match tool_result {
                        Ok(result) => {
                            tracing::info!("[mobile] Tool {} succeeded", tool_call.tool);
                            let result_str = crate::agent::format_tool_result_for_system(&tool_call.tool, &result);
                            continuous_history.push(Message::new(Role::System, &result_str));
                        }
                        Err(e) => {
                            tracing::error!("[mobile] Tool {} failed: {}", tool_call.tool, e);
                            consecutive_errors += 1;
                            continuous_history.push(Message::new(
                                Role::System,
                                &format!("Erreur '{}': {}. Essaie une autre approche.", tool_call.tool, e)
                            ));
                            if consecutive_errors >= 3 { break; }
                        }
                    }
                    continue; // Feed tool result back to LLM
                }
                Err(e) => {
                    let _ = ws_tx.send(WsEvent::Error(e.to_string()));
                    consecutive_errors += 1;
                    if consecutive_errors >= 3 { break; }
                    continue;
                }
            }
        } // end while

        // Save final conversation state
        if !continuous_history.is_empty() {
            if let Ok(mut conv) = conversations::load_conversation(&conv_id_clone) {
                conv.messages = continuous_history;
                let _ = conversations::save_conversation(&conv);
            }
        }

        is_generating.store(false, std::sync::atomic::Ordering::SeqCst);
        state.stop_signal.store(false, std::sync::atomic::Ordering::SeqCst);
        let _ = ws_tx.send(WsEvent::GenerationDone);
    });

    Ok(Json(ChatResponse {
        conversation_id: conv_id,
        status: "generating".to_string(),
    }))
}

/// POST /api/chat/stop — Stop current generation
async fn stop_chat(State(state): State<ServerState>) -> Json<serde_json::Value> {
    state
        .stop_signal
        .store(true, std::sync::atomic::Ordering::SeqCst);
    Json(serde_json::json!({ "status": "stopping" }))
}

/// GET /api/settings — Get current settings
async fn get_settings() -> Json<serde_json::Value> {
    let settings = load_settings();
    Json(serde_json::json!({
        "temperature": settings.temperature,
        "top_p": settings.top_p,
        "top_k": settings.top_k,
        "max_tokens": settings.max_tokens,
        "context_size": settings.context_size,
        "gpu_layers": settings.gpu_layers,
        "theme": settings.theme,
        "font_size": settings.font_size,
        "language": settings.language,
        "auto_approve_all_tools": settings.auto_approve_all_tools,
        "auto_load_model": settings.auto_load_model,
        "models_directory": settings.models_directory.to_string_lossy(),
    }))
}

/// PUT /api/settings — Update settings
async fn update_settings(
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut settings = load_settings();

    if let Some(v) = body.get("temperature").and_then(|v| v.as_f64()) {
        settings.temperature = v as f32;
    }
    if let Some(v) = body.get("top_p").and_then(|v| v.as_f64()) {
        settings.top_p = v as f32;
    }
    if let Some(v) = body.get("top_k").and_then(|v| v.as_u64()) {
        settings.top_k = v as u32;
    }
    if let Some(v) = body.get("max_tokens").and_then(|v| v.as_u64()) {
        settings.max_tokens = v as u32;
    }
    if let Some(v) = body.get("context_size").and_then(|v| v.as_u64()) {
        settings.context_size = v as u32;
    }
    if let Some(v) = body.get("gpu_layers").and_then(|v| v.as_u64()) {
        settings.gpu_layers = v as u32;
    }
    if let Some(v) = body.get("theme").and_then(|v| v.as_str()) {
        settings.theme = v.to_string();
    }
    if let Some(v) = body.get("font_size").and_then(|v| v.as_str()) {
        settings.font_size = v.to_string();
    }
    if let Some(v) = body.get("language").and_then(|v| v.as_str()) {
        settings.language = v.to_string();
    }
    if let Some(v) = body.get("auto_approve_all_tools").and_then(|v| v.as_bool()) {
        settings.auto_approve_all_tools = v;
    }

    settings.validate();

    if let Err(e) = save_settings(&settings) {
        tracing::error!("Failed to save settings: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(serde_json::json!({ "status": "updated" })))
}

/// GET /api/system — Get system info
async fn system_info() -> Json<SystemInfoResponse> {
    let resources = get_resource_usage();
    let gpu = detect_gpu();
    let vram_gb = get_total_vram_gb().unwrap_or(0.0);

    Json(SystemInfoResponse {
        ram_used_mb: resources.ram_used_mb,
        ram_total_mb: resources.ram_total_mb,
        gpu_name: gpu.name,
        gpu_vram_gb: vram_gb,
    })
}

/// POST /api/tools/approve/:id — Approve a pending tool call
async fn approve_tool(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Some((_, approval)) = state.pending_approvals.remove(&id) {
        if let Some(tx) = approval.response_tx.lock().await.take() {
            let _ = tx.send(true);
        }
        Ok(Json(serde_json::json!({ "status": "approved" })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// POST /api/tools/deny/:id — Deny a pending tool call
async fn deny_tool(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Some((_, approval)) = state.pending_approvals.remove(&id) {
        if let Some(tx) = approval.response_tx.lock().await.take() {
            let _ = tx.send(false);
        }
        Ok(Json(serde_json::json!({ "status": "denied" })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
