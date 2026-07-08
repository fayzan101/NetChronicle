use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityCategory {
    Work,
    Learning,
    Entertainment,
    Distraction,
    Neutral,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkStability {
    Stable,
    Degraded,
    Unstable,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawEvent {
    pub user_id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebsiteVisitEvent {
    pub url: String,
    pub domain: String,
    pub duration_sec: u32,
    pub category: ActivityCategory,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppActivityEvent {
    pub app_name: String,
    pub window_title: Option<String>,
    pub duration_sec: u32,
    pub category: ActivityCategory,
    pub recorded_at: DateTime<Utc>,
}
