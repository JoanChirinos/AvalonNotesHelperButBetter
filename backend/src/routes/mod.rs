mod games;
mod ws;

use axum::routing::{get, patch, post, put};
use axum::Router;

use crate::state::AppState;

pub fn api_routes() -> Router<AppState> {
    Router::new()
        // Games
        .route("/games", post(games::create_game))
        .route("/games", get(games::list_games))
        .route("/games/{game_id}", get(games::get_game))
        .route("/games/{game_id}", patch(games::update_game))
        .route("/games/{game_id}", axum::routing::delete(games::delete_game))
        // Namespaces (distinct groups, for the landing picker)
        .route("/namespaces", get(games::list_namespaces))
        // Known players (distinct names across all games)
        .route("/players", get(games::list_known_players))
        // Known players (global roster)
        .route("/known-players", get(games::list_known_players))
        // Players
        .route("/games/{game_id}/players", post(games::add_player))
        .route("/games/{game_id}/players/{player_id}", patch(games::update_player))
        .route("/games/{game_id}/players/{player_id}", axum::routing::delete(games::delete_player))
        // Roles & Modules (for setup)
        .route("/games/{game_id}/players/reorder", post(games::reorder_players))
        .route("/games/{game_id}/roles", post(games::add_role))
        .route("/games/{game_id}/roles/{role_id}", axum::routing::delete(games::delete_role))
        .route("/games/{game_id}/modules", post(games::add_module))
        .route("/games/{game_id}/modules/{module_id}", axum::routing::delete(games::delete_module))
        // Quests
        .route("/games/{game_id}/quests/{quest_id}", patch(games::update_quest))
        // Rounds
        .route("/games/{game_id}/quests/{quest_id}/rounds", post(games::create_round))
        .route("/games/{game_id}/rounds/{round_id}", patch(games::update_round))
        // Votes
        .route("/games/{game_id}/rounds/{round_id}/votes", put(games::record_votes))
        // Lady of the Lake
        .route("/games/{game_id}/lady-investigations", post(games::create_lady_investigation))
        // Lancelot switches
        .route("/games/{game_id}/lancelot-switches", post(games::create_lancelot_switch))
        // Plot cards
        .route("/games/{game_id}/plot-cards", post(games::create_plot_card))
        .route("/games/{game_id}/plot-cards/{plot_card_id}", patch(games::update_plot_card))
        // Assassination
        .route("/games/{game_id}/assassination-attempts", post(games::create_assassination_attempt))
        // Notes
        .route("/games/{game_id}/notes", post(games::create_note))
        .route("/games/{game_id}/notes/{note_id}", patch(games::update_note))
        .route("/games/{game_id}/notes/{note_id}", axum::routing::delete(games::delete_note))
        // WebSocket
        .route("/games/{game_id}/ws", get(ws::ws_handler))
}
