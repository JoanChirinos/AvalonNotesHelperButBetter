mod helpers;

use helpers::*;
use serde_json::json;

#[tokio::test]
async fn foreign_key_enforcement() {
    let server = test_server();
    let state = create_game_with_players(&server, &["A", "B"], &["merlin", "assassin"]).await;
    let game_id = state["game"]["id"].as_str().unwrap();
    let quest_id = state["quests"][0]["quest"]["id"].as_str().unwrap();

    // Creating a round with a non-existent player_id should fail
    let resp = server.post(&format!("/api/games/{game_id}/quests/{quest_id}/rounds"))
        .json(&json!({
            "leader_player_id": "nonexistent-id",
            "team_player_ids": [],
        }))
        .await;
    assert_eq!(resp.status_code().as_u16(), 500);
}

#[tokio::test]
async fn create_round_transaction_rolls_back_on_bad_team_member() {
    let server = test_server();
    let state = create_game_with_players(&server, &["A", "B"], &["merlin", "assassin"]).await;
    let game_id = state["game"]["id"].as_str().unwrap();
    let players = state["players"].as_array().unwrap();
    let quest_id = state["quests"][0]["quest"]["id"].as_str().unwrap();

    // Try to create round with one valid and one invalid team member
    let resp = server.post(&format!("/api/games/{game_id}/quests/{quest_id}/rounds"))
        .json(&json!({
            "leader_player_id": players[0]["id"],
            "team_player_ids": [players[0]["id"], "bad-player-id"],
        }))
        .await;
    assert_eq!(resp.status_code().as_u16(), 500);

    // Verify no round was created (transaction rolled back)
    let state = server.get(&format!("/api/games/{game_id}")).await.json::<serde_json::Value>();
    assert_eq!(state["quests"][0]["rounds"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn record_votes_transaction_rolls_back_on_bad_player() {
    let server = test_server();
    let state = create_game_with_players(&server, &["A", "B"], &["merlin", "assassin"]).await;
    let game_id = state["game"]["id"].as_str().unwrap();
    let players = state["players"].as_array().unwrap();
    let quest_id = state["quests"][0]["quest"]["id"].as_str().unwrap();

    let state = server.post(&format!("/api/games/{game_id}/quests/{quest_id}/rounds"))
        .json(&json!({
            "leader_player_id": players[0]["id"],
            "team_player_ids": [players[0]["id"]],
        }))
        .await.json::<serde_json::Value>();

    let round_id = state["quests"][0]["rounds"][0]["round"]["id"].as_str().unwrap();

    // Try voting with a bad player id
    let resp = server.put(&format!("/api/games/{game_id}/rounds/{round_id}/votes"))
        .json(&json!({
            "votes": [
                { "player_id": players[0]["id"], "vote": "approve" },
                { "player_id": "bad-id", "vote": "reject" },
            ]
        }))
        .await;
    assert_eq!(resp.status_code().as_u16(), 500);

    // Verify no votes exist (transaction rolled back, including the delete)
    let state = server.get(&format!("/api/games/{game_id}")).await.json::<serde_json::Value>();
    assert_eq!(state["quests"][0]["rounds"][0]["votes"].as_array().unwrap().len(), 0);
}
