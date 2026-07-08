//! Computes productivity score, network stability score, focus time, and insights.

mod engine;
mod insight;

pub use engine::{
    AnalyticsEngine, CategoryMinutes, DailyAnalyticsInput, WeeklySummary,
};
pub use insight::{Insight, InsightSeverity};
