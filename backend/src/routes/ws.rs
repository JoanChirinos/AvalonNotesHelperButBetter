use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};

use crate::queries;
use crate::state::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(game_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, game_id, state))
}

async fn handle_socket(socket: WebSocket, game_id: String, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe first to avoid missing broadcasts during initial load
    let tx = state.get_channel(&game_id).await;
    let mut rx = tx.subscribe();

    // Send initial full state
    if let Ok(game_state) = queries::load_full_game_state(&state.db, &game_id) {
        let msg = serde_json::json!({ "type": "game_state", "data": game_state });
        let _ = sender.send(Message::Text(msg.to_string().into())).await;
    }

    // Spawn task to forward broadcasts to this client
    let mut send_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    if sender.send(Message::Text(msg.into())).await.is_err() {
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    // Receive task — just keep connection alive, drain incoming messages
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(_)) = receiver.next().await {}
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
}
