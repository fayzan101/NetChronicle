use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{ActivityCategory, NetworkStability};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDraft {
    pub user_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub primary_apps: Vec<String>,
    pub category: ActivityCategory,
    pub network_stability: Option<NetworkStability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub category: ActivityCategory,
    pub productivity_score: Option<f32>,
    pub network_stability: Option<NetworkStability>,
    pub primary_apps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySummary {
    pub date: chrono::NaiveDate,
    pub productivity_score: f32,
    pub total_online_minutes: u32,
    pub network_health_score: f32,
    pub distraction_ratio: f32,
    pub focus_minutes: u32,
}
