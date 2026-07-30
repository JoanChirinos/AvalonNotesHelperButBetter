mod helpers;

use helpers::*;
use serde_json::{json, Value};

/// A WS client connected to a game receives the initial state on connect, and
/// then a fresh broadcast whenever the game is mutated via the REST API.
#[tokio::test]
async fn websocket_receives_initial_state_and_broadcasts() {
    let server = ws_test_server();
    let state = create_game_with_players(&server, &["A", "B"], &["merlin", "assassin"]).await;
    let game_id = state["game"]["id"].as_str().unwrap().to_string();

    let mut ws = server
        .get_websocket(&format!("/api/games/{game_id}/ws"))
        .await
        .into_websocket()
        .await;

    // Initial message on connect: full state with the two players.
    let initial: Value = ws.receive_json().await;
    assert_eq!(initial["type"], "game_state");
    assert_eq!(initial["data"]["players"].as_array().unwrap().len(), 2);

    // Mutate via REST — should trigger a broadcast to this socket.
    server
        .post(&format!("/api/games/{game_id}/players"))
        .json(&json!({ "name": "C" }))
        .await;

    let broadcast: Value = ws.receive_json().await;
    assert_eq!(broadcast["type"], "game_state");
    assert_eq!(broadcast["data"]["players"].as_array().unwrap().len(), 3);
}

/// Two clients on the same game both receive a broadcast from a single mutation.
#[tokio::test]
async fn websocket_broadcasts_to_all_clients() {
    let server = ws_test_server();
    let state = create_game_with_players(&server, &["A", "B"], &["merlin", "assassin"]).await;
    let game_id = state["game"]["id"].as_str().unwrap().to_string();

    let mut ws1 = server
        .get_websocket(&format!("/api/games/{game_id}/ws"))
        .await
        .into_websocket()
        .await;
    let mut ws2 = server
        .get_websocket(&format!("/api/games/{game_id}/ws"))
        .await
        .into_websocket()
        .await;

    // Drain the initial state on both.
    let _: Value = ws1.receive_json().await;
    let _: Value = ws2.receive_json().await;

    server
        .post(&format!("/api/games/{game_id}/players"))
        .json(&json!({ "name": "C" }))
        .await;

    let b1: Value = ws1.receive_json().await;
    let b2: Value = ws2.receive_json().await;
    assert_eq!(b1["data"]["players"].as_array().unwrap().len(), 3);
    assert_eq!(b2["data"]["players"].as_array().unwrap().len(), 3);
}
