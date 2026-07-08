//! Database access via SQLx (PostgreSQL).

mod models;
mod pool;
mod repository;

pub use models::{
    category_to_db, parse_category, parse_stability, session_row_to_common, ActivitySnapshotRow,
    AppActivityRow, CategoryBreakdownRow, CategoryRuleRow, DailyActivityStats, NetworkLogRow,
    ReportRow, SessionRow, WebsiteLogRow,
};
pub use pool::{create_pool, run_migrations, DbPool};
pub use repository::{
    ActivityRepository, AnalyticsRepository, CategoryRuleRepository, NetworkRepository,
    ReportRepository, SessionRepository, UserRepository,
};
