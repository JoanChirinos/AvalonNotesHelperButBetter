mod helpers;

use helpers::*;
use serde_json::json;

#[tokio::test]
async fn create_game_returns_full_state() {
    let server = test_server();
    let state = create_game_with_players(
        &server,
        &["Alice", "Bob", "Charlie", "Diana", "Eve"],
        &["merlin", "percival", "loyal_servant", "assassin", "morgana"],
    ).await;

    assert_eq!(state["players"].as_array().unwrap().len(), 5);
    assert_eq!(state["quests"].as_array().unwrap().len(), 5);
    assert_eq!(state["roles"].as_array().unwrap().len(), 5);
    assert_eq!(state["game"]["current_quest"], 1);
    assert!(state["game"]["finished_at"].is_null());
}

#[tokio::test]
async fn list_games_returns_summaries() {
    let server = test_server();
    create_game_with_players(&server, &["A", "B", "C", "D", "E"], &["merlin", "assassin", "loyal_servant", "loyal_servant", "minion_of_mordred"]).await;

    let games = server.get("/api/games").await.json::<serde_json::Value>();
    let arr = games.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["player_names"].as_array().unwrap().len(), 5);
    assert_eq!(arr[0]["has_started"], false);
}

#[tokio::test]
async fn get_game_returns_full_state() {
    let server = test_server();
    let state = create_game_with_players(&server, &["A", "B"], &["merlin", "assassin"]).await;
    let game_id = state["game"]["id"].as_str().unwrap();

    let fetched = server.get(&format!("/api/games/{game_id}")).await.json::<serde_json::Value>();
    assert_eq!(fetched["game"]["id"], game_id);
}

#[tokio::test]
async fn add_and_remove_player() {
    let server = test_server();
    let state = create_game_with_players(&server, &["A"], &["merlin"]).await;
    let game_id = state["game"]["id"].as_str().unwrap();

    let state = server.post(&format!("/api/games/{game_id}/players"))
        .json(&json!({ "name": "B" }))
        .await.json::<serde_json::Value>();
    assert_eq!(state["players"].as_array().unwrap().len(), 2);

    let player_id = state["players"][1]["id"].as_str().unwrap();
    server.delete(&format!("/api/games/{game_id}/players/{player_id}")).await;

    let state = server.get(&format!("/api/games/{game_id}")).await.json::<serde_json::Value>();
    assert_eq!(state["players"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn reorder_players() {
    let server = test_server();
    let state = create_game_with_players(&server, &["A", "B", "C"], &["merlin", "assassin", "loyal_servant"]).await;
    let game_id = state["game"]["id"].as_str().unwrap();
    let players = state["players"].as_array().unwrap();
    let ids: Vec<&str> = players.iter().map(|p| p["id"].as_str().unwrap()).collect();

    let reversed = vec![ids[2], ids[1], ids[0]];
    let state = server.post(&format!("/api/games/{game_id}/players/reorder"))
        .json(&json!({ "player_ids": reversed }))
        .await.json::<serde_json::Value>();

    let new_players = state["players"].as_array().unwrap();
    assert_eq!(new_players[0]["id"].as_str().unwrap(), ids[2]);
    assert_eq!(new_players[2]["id"].as_str().unwrap(), ids[0]);
}

#[tokio::test]
async fn add_and_remove_roles() {
    let server = test_server();
    let state = create_game_with_players(&server, &["A"], &[]).await;
    let game_id = state["game"]["id"].as_str().unwrap();

    let state = server.post(&format!("/api/games/{game_id}/roles"))
        .json(&json!({ "role": "merlin" }))
        .await.json::<serde_json::Value>();
    assert_eq!(state["roles"].as_array().unwrap().len(), 1);

    let role_id = state["roles"][0]["id"].as_str().unwrap();
    let state = server.delete(&format!("/api/games/{game_id}/roles/{role_id}"))
        .await.json::<serde_json::Value>();
    assert_eq!(state["roles"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn votes_are_idempotent() {
    let server = test_server();
    let state = create_game_with_players(&server, &["A", "B", "C"], &["merlin", "assassin", "loyal_servant"]).await;
    let game_id = state["game"]["id"].as_str().unwrap();
    let players = state["players"].as_array().unwrap();
    let quest_id = state["quests"][0]["quest"]["id"].as_str().unwrap();

    let state = server.post(&format!("/api/games/{game_id}/quests/{quest_id}/rounds"))
        .json(&json!({
            "leader_player_id": players[0]["id"],
            "team_player_ids": [players[0]["id"], players[1]["id"]],
        }))
        .await.json::<serde_json::Value>();

    let round_id = state["quests"][0]["rounds"][0]["round"]["id"].as_str().unwrap();
    let votes = json!({
        "votes": [
            { "player_id": players[0]["id"], "vote": "approve" },
            { "player_id": players[1]["id"], "vote": "approve" },
            { "player_id": players[2]["id"], "vote": "reject" },
        ]
    });

    server.put(&format!("/api/games/{game_id}/rounds/{round_id}/votes"))
        .json(&votes).await;
    let state = server.put(&format!("/api/games/{game_id}/rounds/{round_id}/votes"))
        .json(&votes).await.json::<serde_json::Value>();

    let round_votes = state["quests"][0]["rounds"][0]["votes"].as_array().unwrap();
    assert_eq!(round_votes.len(), 3);
}

#[tokio::test]
async fn lady_of_the_lake_creates_holder_chain() {
    let server = test_server();
    let state = create_game_with_players(&server, &["A", "B", "C", "D", "E"], &["merlin", "assassin", "loyal_servant", "loyal_servant", "minion_of_mordred"]).await;
    let game_id = state["game"]["id"].as_str().unwrap();
    let players = state["players"].as_array().unwrap();
    let quest_id = state["quests"][1]["quest"]["id"].as_str().unwrap();

    server.post(&format!("/api/games/{game_id}/modules"))
        .json(&json!({ "module": "lady_of_the_lake" })).await;

    let state = server.post(&format!("/api/games/{game_id}/lady-investigations"))
        .json(&json!({
            "quest_id": quest_id,
            "investigator_player_id": players[0]["id"],
            "target_player_id": players[1]["id"],
            "claimed_affiliation": "good",
        }))
        .await.json::<serde_json::Value>();

    let holders = state["lady_holders"].as_array().unwrap();
    assert_eq!(holders.len(), 1);
    assert_eq!(holders[0]["player_id"], players[1]["id"]);
}
