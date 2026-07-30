use avalon_notes::{create_app, db};
use axum_test::TestServer;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU32, Ordering};

static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

fn unique_db_path() -> String {
    let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("/tmp/anh_test_{n}_{}.db", std::process::id())
}

pub fn test_server() -> TestServer {
    let pool = db::init_pool(&unique_db_path());
    TestServer::new(create_app(pool))
}

/// Like `test_server`, but over a real HTTP transport so WebSocket upgrades work.
/// WebSocket tests MUST use this — the default mock transport can't upgrade.
pub fn ws_test_server() -> TestServer {
    let pool = db::init_pool(&unique_db_path());
    TestServer::builder()
        .http_transport()
        .build(create_app(pool))
}

pub async fn create_game_with_players(server: &TestServer, names: &[&str], roles: &[&str]) -> Value {
    let body = json!({
        "player_names": names,
        "roles": roles,
        "modules": [],
    });
    server.post("/api/games").json(&body).await.json::<Value>()
}
