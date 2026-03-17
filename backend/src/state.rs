use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use crate::db::DbPool;

/// Per-game broadcast channel for WebSocket clients
pub type GameChannels = Arc<RwLock<HashMap<String, broadcast::Sender<String>>>>;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub channels: GameChannels,
}

impl AppState {
    pub fn new(db: DbPool) -> Self {
        Self {
            db,
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or create a broadcast channel for a game
    pub async fn get_channel(&self, game_id: &str) -> broadcast::Sender<String> {
        let channels = self.channels.read().await;
        if let Some(tx) = channels.get(game_id) {
            return tx.clone();
        }
        drop(channels);

        let mut channels = self.channels.write().await;
        let (tx, _) = broadcast::channel(64);
        channels.insert(game_id.to_string(), tx.clone());
        tx
    }
}
