use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use sqlx::{Pool, Postgres};

use crate::routes::{add_channel, get_channels, get_feed, health_check, update_feed};

pub struct AppState {
    pub db: Pool<Postgres>,
}

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/health_check", get(health_check))
        .route("/api/channels", get(get_channels).post(add_channel))
        .route("/api/feeds", get(get_feed).post(update_feed))
        .with_state(app_state)
}
