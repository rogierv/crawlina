use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::router::AppState;

#[derive(Serialize)]
struct Channel {
    id: i32,
    link: String,
}

#[derive(Deserialize)]
pub struct AddChannel {
    link: String,
}

pub async fn get_channels(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as!(Channel, "SELECT id, link FROM channels")
        .fetch_all(&data.db)
        .await;

    let channels = query_result.unwrap();
    let json_response = json!(channels);
    Ok(Json(json_response))
}

pub async fn add_channel(
    State(data): State<Arc<AppState>>,
    channel: Query<AddChannel>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let query_result = sqlx::query_as!(
        Channel,
        "INSERT INTO channels (link) VALUES ($1) RETURNING *",
        channel.link.to_string()
    )
    .fetch_one(&data.db)
    .await;

    match query_result {
        Ok(channel) => Ok(Json(json!(channel))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}
