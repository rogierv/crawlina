use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::{DateTime, Utc};
use feed_rs::parser;
use serde::Serialize;
use serde_json::json;

use crate::router::AppState;

#[derive(Serialize)]
pub struct OutputEntry {
    pub id: i32,
    pub title: String,
    pub link: String,
    pub published: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub read: bool,
}

pub async fn get_feed(
    Path(channel_id): Path<i32>,
    data: State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let query = sqlx::query_as!(
        OutputEntry,
        "SELECT id, title, link, published, updated, read 
         FROM entries 
         WHERE channel_id = $1 
         ORDER BY published DESC",
        channel_id
    )
    .fetch_all(&data.db)
    .await;

    match query {
        Ok(result) => Ok(Json(json!(result))),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Database error".to_string(),
        )),
    }
}

pub async fn update_feed(
    Path(channel_id): Path<i32>,
    data: State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let record = sqlx::query!(
        "SELECT link 
         FROM channels 
         WHERE id = $1",
        channel_id
    )
    .fetch_one(&data.db)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Channel does not exist".to_string(),
        )
    })?;

    let content = reqwest::get(record.link.to_string())
        .await
        .expect("Failed to get feed")
        .text()
        .await
        .expect("Failed to parse content");

    let feed: Feed = parser::parse(content.as_bytes())
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to parse feed".to_string(),
            )
        })?
        .into();

    for entry in &feed.entries {
        sqlx::query!(
            "INSERT INTO entries (entry_id, channel_id, title, link, published, updated) 
             VALUES ($1, $2, $3, $4, $5, $6) 
             ON CONFLICT (entry_id) DO NOTHING",
            entry.entry_id,
            channel_id,
            entry.title,
            entry.link,
            entry.published,
            entry.updated
        )
        .execute(&data.db)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to insert entry".to_string(),
            )
        })?;
    }

    update_unread_items(channel_id, data).await?;

    Ok(Json(json!(feed)))
}

pub async fn toggle_read(
    Path(id): Path<i32>,
    data: State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let record = sqlx::query!(
        "UPDATE entries 
         SET read = NOT read 
         WHERE id = $1
         RETURNING channel_id",
        id
    )
    .fetch_one(&data.db)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to set entry to read".to_string(),
        )
    })?;

    update_unread_items(record.channel_id, data).await?;

    Ok(())
}

async fn update_unread_items(
    channel_id: i32,
    data: State<Arc<AppState>>,
) -> Result<(), (StatusCode, String)> {
    sqlx::query!(
        r#"UPDATE channels SET unread = (SELECT COUNT(*) FROM entries WHERE channel_id = $1 AND read = FALSE) WHERE id = $1"#,
        channel_id
    )
    .execute(&data.db)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to fetch the unread items".to_string(),
        )
    })?;
    Ok(())
}

#[derive(Clone, Serialize)]
pub struct Feed {
    pub id: String,
    pub title: Option<String>,
    pub entries: Vec<Entry>,
}

#[derive(Clone, Serialize)]
pub struct Entry {
    pub entry_id: String,
    pub title: String,
    pub link: String,
    pub published: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

impl From<feed_rs::model::Feed> for Feed {
    fn from(f: feed_rs::model::Feed) -> Self {
        let id = f.id;
        let title = f.title.map(|text| text.content);
        let entries = f
            .entries
            .into_iter()
            .map(|entry| entry.try_into().unwrap())
            .collect();

        Self { id, title, entries }
    }
}

impl TryFrom<feed_rs::model::Entry> for Entry {
    type Error = String;

    fn try_from(e: feed_rs::model::Entry) -> Result<Self, Self::Error> {
        let title = e.title.map(|text| text.content);
        let link = e.links.first().map(|link| link.clone().href);

        if title.is_none() || link.is_none() || e.published.is_none() || e.updated.is_none() {
            return Err("Missing required fields".to_string());
        }

        Ok(Self {
            entry_id: e.id,
            title: title.unwrap(),
            link: link.unwrap(),
            published: e.published.unwrap(),
            updated: e.updated.unwrap(),
        })
    }
}
