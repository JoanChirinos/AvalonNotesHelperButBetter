mod helpers;

use helpers::*;
use serde_json::json;

#[tokio::test]
async fn full_game_three_successes() {
    let server = test_server();
    let state = create_game_with_players(
        &server,
        &["A", "B", "C", "D", "E"],
        &["merlin", "percival", "loyal_servant", "assassin", "morgana"],
    ).await;
    let game_id = state["game"]["id"].as_str().unwrap();
    let players = state["players"].as_array().unwrap();
    let p: Vec<&str> = players.iter().map(|p| p["id"].as_str().unwrap()).collect();

    for quest_num in 0..3 {
        let state = server.get(&format!("/api/games/{game_id}")).await.json::<serde_json::Value>();
        let quest_id = state["quests"][quest_num]["quest"]["id"].as_str().unwrap();

        let state = server.post(&format!("/api/games/{game_id}/quests/{quest_id}/rounds"))
            .json(&json!({
                "leader_player_id": p[0],
                "team_player_ids": [p[0], p[1]],
            }))
            .await.json::<serde_json::Value>();

        let round_id = state["quests"][quest_num]["rounds"][0]["round"]["id"].as_str().unwrap();

        server.put(&format!("/api/games/{game_id}/rounds/{round_id}/votes"))
            .json(&json!({
                "votes": [
                    { "player_id": p[0], "vote": "approve" },
                    { "player_id": p[1], "vote": "approve" },
                    { "player_id": p[2], "vote": "approve" },
                    { "player_id": p[3], "vote": "reject" },
                    { "player_id": p[4], "vote": "reject" },
                ]
            })).await;

        server.patch(&format!("/api/games/{game_id}/rounds/{round_id}"))
            .json(&json!({ "status": "approved" })).await;

        server.patch(&format!("/api/games/{game_id}/quests/{quest_id}"))
            .json(&json!({
                "result": "success",
                "success_count": 2,
                "fail_count": 0,
            })).await;

        if quest_num < 2 {
            server.patch(&format!("/api/games/{game_id}"))
                .json(&json!({ "current_quest": quest_num + 2 })).await;
        }
    }

    let games = server.get("/api/games").await.json::<serde_json::Value>();
    let game_summary = &games.as_array().unwrap()[0];
    // With merlin in the game, result should be null (assassination pending)
    assert!(game_summary["result"].is_null());
}

#[tokio::test]
async fn quest_fails_on_five_rejections() {
    let server = test_server();
    let state = create_game_with_players(
        &server,
        &["A", "B", "C", "D", "E"],
        &["merlin", "percival", "loyal_servant", "assassin", "morgana"],
    ).await;
    let game_id = state["game"]["id"].as_str().unwrap();
    let players = state["players"].as_array().unwrap();
    let p: Vec<&str> = players.iter().map(|p| p["id"].as_str().unwrap()).collect();
    let quest_id = state["quests"][0]["quest"]["id"].as_str().unwrap();

    // Create 5 rounds, all rejected
    for i in 0..5 {
        let state = server.post(&format!("/api/games/{game_id}/quests/{quest_id}/rounds"))
            .json(&json!({
                "leader_player_id": p[i % 5],
                "team_player_ids": [p[0], p[1]],
            }))
            .await.json::<serde_json::Value>();

        let round_id = state["quests"][0]["rounds"].as_array().unwrap().last().unwrap()["round"]["id"].as_str().unwrap();

        server.put(&format!("/api/games/{game_id}/rounds/{round_id}/votes"))
            .json(&json!({
                "votes": [
                    { "player_id": p[0], "vote": "reject" },
                    { "player_id": p[1], "vote": "reject" },
                    { "player_id": p[2], "vote": "reject" },
                    { "player_id": p[3], "vote": "reject" },
                    { "player_id": p[4], "vote": "reject" },
                ]
            })).await;

        server.patch(&format!("/api/games/{game_id}/rounds/{round_id}"))
            .json(&json!({ "status": "rejected" })).await;
    }

    // Mark quest as failed (5 rejections)
    server.patch(&format!("/api/games/{game_id}/quests/{quest_id}"))
        .json(&json!({ "result": "fail" })).await;

    let state = server.get(&format!("/api/games/{game_id}")).await.json::<serde_json::Value>();
    assert_eq!(state["quests"][0]["quest"]["result"], "fail");
    assert!(state["quests"][0]["quest"]["success_count"].is_null());
    assert!(state["quests"][0]["quest"]["fail_count"].is_null());
}

#[tokio::test]
async fn assassination_attempt_overwrites_same_phase() {
    let server = test_server();
    let state = create_game_with_players(
        &server,
        &["A", "B", "C", "D", "E"],
        &["merlin", "percival", "loyal_servant", "assassin", "morgana"],
    ).await;
    let game_id = state["game"]["id"].as_str().unwrap();
    let players = state["players"].as_array().unwrap();
    let p: Vec<&str> = players.iter().map(|p| p["id"].as_str().unwrap()).collect();

    // First attempt succeeds
    let resp = server.post(&format!("/api/games/{game_id}/assassination-attempts"))
        .json(&json!({
            "phase": 2,
            "sniper_player_id": p[3],
            "snipe_type": "merlin",
            "target_player_ids": [p[0]],
            "correct": true,
        }))
        .await;
    assert_eq!(resp.status_code().as_u16(), 201);

    // Re-recording the same phase overwrites (editable), not a unique-violation error.
    let state = server.post(&format!("/api/games/{game_id}/assassination-attempts"))
        .json(&json!({
            "phase": 2,
            "sniper_player_id": p[3],
            "snipe_type": "merlin",
            "target_player_ids": [p[1]],
            "correct": false,
        }))
        .await.json::<serde_json::Value>();

    // Exactly one phase-2 attempt remains, reflecting the corrected values.
    let attempts = state["assassination_attempts"].as_array().unwrap();
    let phase2: Vec<_> = attempts.iter().filter(|a| a["phase"] == 2).collect();
    assert_eq!(phase2.len(), 1);
    assert_eq!(phase2[0]["correct"], 0);
    assert_eq!(phase2[0]["target_player_ids"], serde_json::to_string(&[p[1]]).unwrap());
}

#[tokio::test]
async fn finish_game_sets_timestamp() {
    let server = test_server();
    let state = create_game_with_players(&server, &["A", "B"], &["merlin", "assassin"]).await;
    let game_id = state["game"]["id"].as_str().unwrap();

    let state = server.patch(&format!("/api/games/{game_id}"))
        .json(&json!({ "finished_at": "2026-01-01T00:00:00Z" }))
        .await.json::<serde_json::Value>();

    assert_eq!(state["game"]["finished_at"], "2026-01-01T00:00:00Z");
}
