mod db;
mod models;
mod queries;
mod routes;
mod schema;
mod state;
mod types;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // TEMP: deliberate startup panic to demo CI auto-rollback. Revert me.
    panic!("intentional rollback demo");

    let pool = db::init_pool("avalon.db");
    let state = state::AppState::new(pool);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .nest("/api", routes::api_routes())
        .with_state(state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8337").await.unwrap();
    tracing::info!("Listening on http://localhost:8337");
    axum::serve(listener, app).await.unwrap();
}
