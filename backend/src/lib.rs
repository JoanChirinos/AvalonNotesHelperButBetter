pub mod db;
pub mod models;
pub mod queries;
pub mod routes;
pub mod schema;
pub mod state;
pub mod types;

use axum::Router;

pub fn create_app(pool: db::DbPool) -> Router {
    let state = state::AppState::new(pool);
    Router::new()
        .nest("/api", routes::api_routes())
        .with_state(state)
}
