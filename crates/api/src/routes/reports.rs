use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use chrono::{Datelike, NaiveDate};
use netchronicle_analytics::{AnalyticsEngine, DailyAnalyticsInput};
use netchronicle_db::{
    session_row_to_common, AnalyticsRepository, NetworkRepository, ReportRepository,
    SessionRepository,
};
use serde::{Deserialize, Serialize};

use crate::error::{ApiError, ApiResult};
use crate::params::{day_bounds, DateRangeParams};
use crate::query::UserQuery;
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportResponse {
    pub report_type: String,
    pub period_start: String,
    pub period_end: String,
    pub summary: serde_json::Value,
    pub cached: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportsListResponse {
    pub reports: Vec<ReportListItem>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportListItem {
    pub report_type: String,
    pub period_start: String,
    pub period_end: String,
    pub summary: serde_json::Value,
    pub created_at: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportQuery {
    pub format: Option<String>,
    pub report_type: Option<String>,
    pub date: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub from: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub to: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub user_id: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/reports/daily", get(reports_daily))
        .route("/reports/weekly", get(reports_weekly))
        .route("/reports/monthly", get(reports_monthly))
        .route("/reports/export", get(reports_export))
        .route("/reports", get(list_reports))
}

async fn reports_daily(
    State(state): State<AppState>,
    user: UserQuery,
    range: DateRangeParams,
) -> ApiResult<Json<ReportResponse>> {
    let day = range.from.date_naive();
    let (summary, cached) = load_or_compute_daily(&state, user.user_id, day).await?;
    Ok(Json(ReportResponse {
        report_type: "daily".into(),
        period_start: day.to_string(),
        period_end: day.to_string(),
        summary,
        cached,
    }))
}

async fn reports_weekly(
    State(state): State<AppState>,
    user: UserQuery,
    range: DateRangeParams,
) -> ApiResult<Json<ReportResponse>> {
    let (week_start, week_end) = week_bounds(range.from.date_naive());
    let (summary, cached) =
        load_or_compute_period(&state, user.user_id, "weekly", week_start, week_end).await?;
    Ok(Json(ReportResponse {
        report_type: "weekly".into(),
        period_start: week_start.to_string(),
        period_end: week_end.to_string(),
        summary,
        cached,
    }))
}

async fn reports_monthly(
    State(state): State<AppState>,
    user: UserQuery,
    range: DateRangeParams,
) -> ApiResult<Json<ReportResponse>> {
    let (month_start, month_end) = month_bounds(range.from.date_naive());
    let (summary, cached) =
        load_or_compute_period(&state, user.user_id, "monthly", month_start, month_end).await?;
    Ok(Json(ReportResponse {
        report_type: "monthly".into(),
        period_start: month_start.to_string(),
        period_end: month_end.to_string(),
        summary,
        cached,
    }))
}

async fn list_reports(
    State(state): State<AppState>,
    user: UserQuery,
    Query(query): Query<ExportQuery>,
) -> ApiResult<Json<ReportsListResponse>> {
    let report_type = query.report_type.as_deref();
    let rows = ReportRepository::new(&state.db)
        .list(user.user_id, report_type, 50)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(Json(ReportsListResponse {
        reports: rows
            .into_iter()
            .map(|row| ReportListItem {
                report_type: row.report_type,
                period_start: row.period_start.to_string(),
                period_end: row.period_end.to_string(),
                summary: row.summary,
                created_at: row.created_at.to_rfc3339(),
            })
            .collect(),
    }))
}

async fn reports_export(
    State(state): State<AppState>,
    user: UserQuery,
    Query(query): Query<ExportQuery>,
) -> Result<Response, ApiError> {
    let format = query
        .format
        .as_deref()
        .unwrap_or("json")
        .to_ascii_lowercase();
    let report_type = query.report_type.as_deref().unwrap_or("daily");

    let day = query
        .date
        .as_deref()
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    let (period_start, period_end, summary) = match report_type {
        "weekly" => {
            let (start, end) = week_bounds(day);
            let (summary, _) =
                load_or_compute_period(&state, user.user_id, "weekly", start, end).await?;
            (start, end, summary)
        }
        "monthly" => {
            let (start, end) = month_bounds(day);
            let (summary, _) =
                load_or_compute_period(&state, user.user_id, "monthly", start, end).await?;
            (start, end, summary)
        }
        _ => {
            let (summary, _) = load_or_compute_daily(&state, user.user_id, day).await?;
            (day, day, summary)
        }
    };

    if format == "csv" {
        let csv = summary_to_csv(report_type, period_start, period_end, &summary);
        return Ok((
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/csv; charset=utf-8")],
            csv,
        )
            .into_response());
    }

    let body = serde_json::json!({
        "reportType": report_type,
        "periodStart": period_start.to_string(),
        "periodEnd": period_end.to_string(),
        "summary": summary,
    });
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        body.to_string(),
    )
        .into_response())
}

fn summary_to_csv(
    report_type: &str,
    period_start: NaiveDate,
    period_end: NaiveDate,
    summary: &serde_json::Value,
) -> String {
    let mut lines = vec![
        "reportType,periodStart,periodEnd,key,value".to_string(),
        format!(
            "{report_type},{period_start},{period_end},totalOnlineMinutes,{}",
            summary["totalOnlineMinutes"].as_i64().unwrap_or(0)
        ),
        format!(
            "{report_type},{period_start},{period_end},productiveMinutes,{}",
            summary["productiveMinutes"]
                .as_i64()
                .or_else(|| summary["focusMinutes"].as_i64())
                .unwrap_or(0)
        ),
        format!(
            "{report_type},{period_start},{period_end},productivityScore,{}",
            summary["productivityScore"]
                .as_f64()
                .or_else(|| summary["averageProductivityScore"].as_f64())
                .unwrap_or(0.0)
        ),
        format!(
            "{report_type},{period_start},{period_end},distractionImpactPct,{}",
            summary["distractionImpactPct"]
                .as_f64()
                .or_else(|| summary["distractionRatio"].as_f64().map(|r| r * 100.0))
                .unwrap_or(0.0)
        ),
    ];

    if let Some(apps) = summary["topApps"].as_array() {
        for app in apps {
            lines.push(format!(
                "{report_type},{period_start},{period_end},topApp:{},{}",
                app["app"].as_str().unwrap_or(""),
                app["minutes"].as_i64().unwrap_or(0)
            ));
        }
    }

    lines.join("\n") + "\n"
}

async fn load_or_compute_daily(
    state: &AppState,
    user_id: uuid::Uuid,
    day: NaiveDate,
) -> ApiResult<(serde_json::Value, bool)> {
    let reports = ReportRepository::new(&state.db);
    if let Some(cached) = reports
        .get(user_id, "daily", day, day)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    {
        return Ok((cached.summary, true));
    }

    let (from, to) = day_bounds(day);
    let sessions: Vec<_> = SessionRepository::new(&state.db)
        .list(user_id, from, to, 1000, 0)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        .into_iter()
        .map(session_row_to_common)
        .collect();

    let network_score = NetworkRepository::new(&state.db)
        .stability_score(user_id, from)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    let summary = AnalyticsEngine::daily_summary(&DailyAnalyticsInput {
        date: day,
        sessions: sessions.clone(),
        network_health_score: network_score,
    });

    let payload = serde_json::json!({
        "productivityScore": summary.productivity_score,
        "totalOnlineMinutes": summary.total_online_minutes,
        "networkHealthScore": summary.network_health_score,
        "distractionRatio": summary.distraction_ratio,
        "distractionImpactPct": AnalyticsEngine::distraction_impact_pct(&sessions),
        "focusMinutes": summary.focus_minutes,
        "timeOfDay": AnalyticsEngine::time_of_day_patterns(&sessions),
    });

    reports
        .upsert(user_id, "daily", day, day, payload.clone())
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok((payload, false))
}

async fn load_or_compute_period(
    state: &AppState,
    user_id: uuid::Uuid,
    report_type: &str,
    period_start: NaiveDate,
    period_end: NaiveDate,
) -> ApiResult<(serde_json::Value, bool)> {
    let reports = ReportRepository::new(&state.db);
    if let Some(cached) = reports
        .get(user_id, report_type, period_start, period_end)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    {
        return Ok((cached.summary, true));
    }

    let (from, _) = day_bounds(period_start);
    let (_, to) = day_bounds(period_end);
    let sessions: Vec<_> = SessionRepository::new(&state.db)
        .list(user_id, from, to, 10_000, 0)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        .into_iter()
        .map(session_row_to_common)
        .collect();

    let period = if report_type == "monthly" {
        AnalyticsEngine::monthly_summary(&sessions)
    } else {
        let weekly = AnalyticsEngine::weekly_summary(&sessions);
        netchronicle_analytics::PeriodSummary {
            total_online_minutes: weekly.total_online_minutes,
            productive_minutes: weekly.productive_minutes,
            session_count: weekly.session_count,
            average_productivity_score: weekly.average_productivity_score,
            category_minutes: weekly.category_minutes,
            distraction_impact_pct: AnalyticsEngine::distraction_impact_pct(&sessions),
            time_of_day: AnalyticsEngine::time_of_day_patterns(&sessions),
        }
    };

    let analytics = AnalyticsRepository::new(&state.db);
    let top_apps = analytics
        .top_apps(user_id, from, to, 10)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let top_domains = analytics
        .top_domains(user_id, from, to, 10)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    let summary = serde_json::json!({
        "totalOnlineMinutes": period.total_online_minutes,
        "productiveMinutes": period.productive_minutes,
        "sessionCount": period.session_count,
        "averageProductivityScore": period.average_productivity_score,
        "distractionImpactPct": period.distraction_impact_pct,
        "categoryMinutes": period.category_minutes,
        "timeOfDay": period.time_of_day,
        "topApps": top_apps.into_iter().map(|(name, secs)| serde_json::json!({"app": name, "minutes": secs / 60})).collect::<Vec<_>>(),
        "topDomains": top_domains.into_iter().map(|(domain, secs)| serde_json::json!({"domain": domain, "minutes": secs / 60})).collect::<Vec<_>>(),
    });

    reports
        .upsert(
            user_id,
            report_type,
            period_start,
            period_end,
            summary.clone(),
        )
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok((summary, false))
}

fn week_bounds(day: NaiveDate) -> (NaiveDate, NaiveDate) {
    let weekday = day.weekday().num_days_from_monday() as i64;
    let week_start = day - chrono::Duration::days(weekday);
    let week_end = week_start + chrono::Duration::days(6);
    (week_start, week_end)
}

fn month_bounds(day: NaiveDate) -> (NaiveDate, NaiveDate) {
    let month_start = NaiveDate::from_ymd_opt(day.year(), day.month(), 1).unwrap();
    let month_end = if day.month() == 12 {
        NaiveDate::from_ymd_opt(day.year() + 1, 1, 1).unwrap() - chrono::Duration::days(1)
    } else {
        NaiveDate::from_ymd_opt(day.year(), day.month() + 1, 1).unwrap() - chrono::Duration::days(1)
    };
    (month_start, month_end)
}
