use chrono::{DateTime, Utc};
use netchronicle_common::{ActivityCategory, NetworkStability};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AppActivityRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub app_name: String,
    pub window_title: Option<String>,
    pub duration_sec: i32,
    pub category: String,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct WebsiteLogRow {
    pub id: Uuid,
    pub url: String,
    pub domain: String,
    pub time_spent_sec: i32,
    pub category: String,
    pub visited_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct NetworkLogRow {
    pub latency_ms: Option<f32>,
    pub packet_loss_pct: Option<f32>,
    pub bandwidth_mbps: Option<f32>,
    pub stability: Option<String>,
    pub disconnect: bool,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ActivitySnapshotRow {
    pub payload: serde_json::Value,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DailyActivityStats {
    pub total_sec: i64,
    pub productive_sec: i64,
    pub distraction_sec: i64,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct CategoryBreakdownRow {
    pub category: String,
    pub total_sec: i64,
}

pub fn parse_category(value: &str) -> ActivityCategory {
    match value {
        "work" => ActivityCategory::Work,
        "learning" => ActivityCategory::Learning,
        "entertainment" => ActivityCategory::Entertainment,
        "distraction" => ActivityCategory::Distraction,
        "neutral" => ActivityCategory::Neutral,
        _ => ActivityCategory::Unknown,
    }
}

pub fn category_to_db(category: ActivityCategory) -> &'static str {
    match category {
        ActivityCategory::Work => "work",
        ActivityCategory::Learning => "learning",
        ActivityCategory::Entertainment => "entertainment",
        ActivityCategory::Distraction => "distraction",
        ActivityCategory::Neutral => "neutral",
        ActivityCategory::Unknown => "unknown",
    }
}

pub fn stability_to_db(stability: NetworkStability) -> &'static str {
    match stability {
        NetworkStability::Stable => "stable",
        NetworkStability::Degraded => "degraded",
        NetworkStability::Unstable => "unstable",
        NetworkStability::Offline => "offline",
    }
}
