use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::{DateTime, Utc};
use feed_rs::parser;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::router::AppState;

#[derive(Deserialize)]
pub struct GetFeed {
    channel_id: i32,
    link: String,
}

pub async fn get_feed(
    get_feed: Query<GetFeed>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let query_result = sqlx::query_as!(
        Entry,
        "SELECT id, title, link, published, updated FROM entries WHERE channel_id = $1 ORDER BY published ASC",
        get_feed.channel_id
    )
    .fetch_all(&data.db)
    .await;

    Ok(Json(json!(query_result.ok())))
}

pub async fn update_feed(
    get_feed: Query<GetFeed>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let content = reqwest::get(get_feed.link.to_string())
        .await
        .expect("Failed to get feed")
        .text()
        .await
        .expect("Failed to parse content");

    let feed: Feed = parser::parse(content.as_bytes()).unwrap().into();

    for entry in &feed.entries {
        let _ = sqlx::query_as!(
            Entry,
            "INSERT INTO entries (id, channel_id, title, link, published, updated) VALUES ($1, $2, $3, $4, $5, $6)",
            entry.id,
            get_feed.channel_id,
            entry.title,
            entry.link,
            entry.published,
            entry.updated
        ).execute(&data.db).await;
    }

    Ok(Json(json!(feed)))
}

#[derive(Clone, Serialize)]
pub struct Feed {
    pub id: String,
    pub title: Option<String>,
    pub entries: Vec<Entry>,
}

#[derive(Clone, Serialize)]
pub struct Entry {
    pub id: String,
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
        let id = e.id;
        let title = e.title.map(|text| text.content);
        let link = e.links.first().map(|link| link.clone().href);

        if title.is_none() || link.is_none() || e.published.is_none() || e.updated.is_none() {
            return Err("Missing required fields".to_string());
        }

        Ok(Self {
            id,
            title: title.unwrap(),
            link: link.unwrap(),
            published: e.published.unwrap(),
            updated: e.updated.unwrap(),
        })
    }
}
