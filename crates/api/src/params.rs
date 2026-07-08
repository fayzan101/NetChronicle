use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use chrono::{DateTime, NaiveDate, Utc};

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct DateRangeParams {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    pub limit: i64,
    pub offset: i64,
}

impl DateRangeParams {
    pub fn parse(query: &str) -> ApiResult<Self> {
        let mut date: Option<NaiveDate> = None;
        let mut from: Option<DateTime<Utc>> = None;
        let mut to: Option<DateTime<Utc>> = None;
        let mut limit = 100_i64;
        let mut offset = 0_i64;

        for pair in query.split('&').filter(|part| !part.is_empty()) {
            let Some((key, value)) = pair.split_once('=') else {
                continue;
            };
            match key {
                "date" => {
                    date = Some(
                        NaiveDate::parse_from_str(value, "%Y-%m-%d")
                            .map_err(|_| ApiError::bad_request("invalid date, use YYYY-MM-DD"))?,
                    );
                }
                "from" => {
                    from = Some(
                        DateTime::parse_from_rfc3339(value)
                            .map(|dt| dt.with_timezone(&Utc))
                            .map_err(|_| ApiError::bad_request("invalid from timestamp"))?,
                    );
                }
                "to" => {
                    to = Some(
                        DateTime::parse_from_rfc3339(value)
                            .map(|dt| dt.with_timezone(&Utc))
                            .map_err(|_| ApiError::bad_request("invalid to timestamp"))?,
                    );
                }
                "limit" => {
                    limit = value
                        .parse()
                        .map_err(|_| ApiError::bad_request("invalid limit"))?;
                    limit = limit.clamp(1, 1000);
                }
                "offset" => {
                    offset = value
                        .parse()
                        .map_err(|_| ApiError::bad_request("invalid offset"))?;
                    offset = offset.max(0);
                }
                _ => {}
            }
        }

        if let Some(day) = date {
            return Ok(Self {
                from: day.and_hms_opt(0, 0, 0).unwrap().and_utc(),
                to: (day + chrono::Duration::days(1))
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc(),
                limit,
                offset,
            });
        }

        let from = from.unwrap_or_else(|| Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc());
        let to = to.unwrap_or(from + chrono::Duration::days(1));

        Ok(Self {
            from,
            to,
            limit,
            offset,
        })
    }
}

impl FromRequestParts<AppState> for DateRangeParams {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &AppState) -> Result<Self, Self::Rejection> {
        Self::parse(parts.uri.query().unwrap_or_default())
    }
}

pub fn day_bounds(day: NaiveDate) -> (DateTime<Utc>, DateTime<Utc>) {
    let start = day.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let end = (day + chrono::Duration::days(1))
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();
    (start, end)
}
