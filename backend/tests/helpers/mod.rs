use axum_test::TestServer;
use avalon_notes::{create_app, db};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU32, Ordering};

static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

pub fn test_server() -> TestServer {
    let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let db_path = format!("/tmp/anh_test_{n}_{}.db", std::process::id());
    let pool = db::init_pool(&db_path);
    let app = create_app(pool);
    TestServer::new(app)
}

pub async fn create_game_with_players(server: &TestServer, names: &[&str], roles: &[&str]) -> Value {
    let body = json!({
        "player_names": names,
        "roles": roles,
        "modules": [],
    });
    server.post("/api/games").json(&body).await.json::<Value>()
}
