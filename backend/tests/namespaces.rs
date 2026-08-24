mod helpers;

use axum_test::TestServer;
use helpers::*;
use serde_json::{json, Value};

async fn create_game_in_ns(server: &TestServer, ns: &str, names: &[&str]) -> Value {
    let body = json!({
        "player_names": names,
        "roles": [],
        "modules": [],
        "namespace": ns,
    });
    server.post("/api/games").json(&body).await.json::<Value>()
}

/// Games created in different namespaces are isolated in the lobby listing.
#[tokio::test]
async fn games_are_isolated_by_namespace() {
    let server = test_server();
    create_game_in_ns(&server, "alpha", &["Alice"]).await;
    create_game_in_ns(&server, "beta", &["Bob"]).await;

    let alpha = server.get("/api/games?namespace=alpha").await.json::<Value>();
    assert_eq!(alpha.as_array().unwrap().len(), 1);
    assert_eq!(alpha[0]["player_names"][0], "Alice");

    let beta = server.get("/api/games?namespace=beta").await.json::<Value>();
    assert_eq!(beta.as_array().unwrap().len(), 1);
    assert_eq!(beta[0]["player_names"][0], "Bob");

    let empty = server.get("/api/games?namespace=gamma").await.json::<Value>();
    assert_eq!(empty.as_array().unwrap().len(), 0);
}

/// The created game carries its namespace, and namespaces are case-sensitive.
#[tokio::test]
async fn namespace_is_stored_and_case_sensitive() {
    let server = test_server();
    let state = create_game_in_ns(&server, "SGW", &["A"]).await;
    assert_eq!(state["game"]["namespace"], "SGW");

    // lowercase is a DIFFERENT namespace
    let lower = server.get("/api/games?namespace=sgw").await.json::<Value>();
    assert_eq!(lower.as_array().unwrap().len(), 0);
    let upper = server.get("/api/games?namespace=SGW").await.json::<Value>();
    assert_eq!(upper.as_array().unwrap().len(), 1);
}

/// The known-player roster is scoped per namespace.
#[tokio::test]
async fn known_player_roster_is_isolated() {
    let server = test_server();
    create_game_in_ns(&server, "alpha", &["Alice"]).await;
    create_game_in_ns(&server, "beta", &["Bob"]).await;

    let alpha = server.get("/api/known-players?namespace=alpha").await.json::<Value>();
    let alpha_names: Vec<&str> = alpha.as_array().unwrap().iter().map(|p| p["name"].as_str().unwrap()).collect();
    assert!(alpha_names.contains(&"Alice"));
    assert!(!alpha_names.contains(&"Bob"));

    let beta = server.get("/api/known-players?namespace=beta").await.json::<Value>();
    let beta_names: Vec<&str> = beta.as_array().unwrap().iter().map(|p| p["name"].as_str().unwrap()).collect();
    assert!(beta_names.contains(&"Bob"));
    assert!(!beta_names.contains(&"Alice"));
}

/// The same player name can exist independently in two namespaces.
#[tokio::test]
async fn same_name_allowed_across_namespaces() {
    let server = test_server();
    let a = create_game_in_ns(&server, "alpha", &["Sam"]).await;
    let b = create_game_in_ns(&server, "beta", &["Sam"]).await;

    // Both games created successfully with their own "Sam" known-player.
    let a_kp = a["known_players"][0]["id"].as_str().unwrap();
    let b_kp = b["known_players"][0]["id"].as_str().unwrap();
    assert_ne!(a_kp, b_kp);
    assert_eq!(a["known_players"][0]["namespace"], "alpha");
    assert_eq!(b["known_players"][0]["namespace"], "beta");
}

/// add_player creates the known-player in the game's namespace, not a global one.
#[tokio::test]
async fn add_player_uses_game_namespace() {
    let server = test_server();
    let state = create_game_in_ns(&server, "xray", &[]).await;
    let game_id = state["game"]["id"].as_str().unwrap();

    server
        .post(&format!("/api/games/{game_id}/players"))
        .json(&json!({ "name": "Zoe" }))
        .await;

    let xray = server.get("/api/known-players?namespace=xray").await.json::<Value>();
    let xray_names: Vec<&str> = xray.as_array().unwrap().iter().map(|p| p["name"].as_str().unwrap()).collect();
    assert!(xray_names.contains(&"Zoe"));

    let sgw = server.get("/api/known-players?namespace=SGW").await.json::<Value>();
    let sgw_names: Vec<&str> = sgw.as_array().unwrap().iter().map(|p| p["name"].as_str().unwrap()).collect();
    assert!(!sgw_names.contains(&"Zoe"));
}

/// The namespaces endpoint lists the distinct namespaces that have games, sorted.
#[tokio::test]
async fn list_namespaces_returns_distinct_sorted() {
    let server = test_server();
    create_game_in_ns(&server, "beta", &["A"]).await;
    create_game_in_ns(&server, "alpha", &["B"]).await;
    create_game_in_ns(&server, "alpha", &["C"]).await; // duplicate namespace

    let ns = server.get("/api/namespaces").await.json::<Value>();
    let names: Vec<&str> = ns.as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect();
    assert_eq!(names, vec!["alpha", "beta"]);
}

/// Omitting the namespace on create defaults to SGW (back-compat for old clients).
#[tokio::test]
async fn create_defaults_to_sgw_when_omitted() {
    let server = test_server();
    let body = json!({ "player_names": ["A"], "roles": [], "modules": [] });
    let state = server.post("/api/games").json(&body).await.json::<Value>();
    assert_eq!(state["game"]["namespace"], "SGW");
}
