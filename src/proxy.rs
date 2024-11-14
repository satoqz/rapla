use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use chrono::{Datelike, Duration, Utc};
use serde::Deserialize;

use crate::parser::parse_calendar;
use crate::structs::Calendar;

const UPSTREAM: &str = "https://rapla.dhbw.de";

type Cache = Arc<crate::cache::Cache<(String, String), Calendar>>;

pub fn router(cache_config: crate::cache::Config) -> Router {
    let cache = crate::cache::Cache::new(cache_config);
    Router::new()
        .route("/:calendar_path", get(handle_calendar))
        .with_state(cache)
}

#[derive(Deserialize)]
struct CalendarQuery {
    key: String,
    salt: String,
}

async fn handle_calendar(
    State(cache): State<Cache>,
    Path(calendar_path): Path<String>,
    Query(CalendarQuery { key, salt }): Query<CalendarQuery>,
) -> Response {
    let calendar = fetch_calendar(calendar_path, key, salt, cache).await;

    match calendar {
        Some(calendar) => (
            [("content-type", "text/calendar")],
            calendar.to_ics().to_string(),
        )
            .into_response(),
        None => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to proxy calendar",
        )
            .into_response(),
    }
}

async fn fetch_calendar(
    calendar_path: String,
    key: String,
    salt: String,
    cache: Cache,
) -> Option<Arc<Calendar>> {
    let cache_key = (key, salt);
    if let Some(cached) = cache.get(&cache_key).await {
        return Some(cached);
    }

    let url = generate_upstream_url(&calendar_path, &cache_key.0, &cache_key.1);
    eprintln!("{url}");

    let html = reqwest::get(url).await.ok()?.text().await.ok()?;
    let calendar = parse_calendar(&html)?;

    Some(cache.insert(cache_key, calendar).await)
}

fn generate_upstream_url(calendar_path: &str, key: &str, salt: &str) -> String {
    // these don't need to be 100% accurate
    const WEEKS_TWO_YEARS: usize = 104;
    const DAYS_ONE_YEAR: i64 = 365;

    let now = Utc::now();
    let year_ago = now - Duration::try_days(DAYS_ONE_YEAR).unwrap();

    format!(
        "{UPSTREAM}/rapla/{calendar_path}?key={key}&salt={salt}&day={}&month={}&year={}&pages={WEEKS_TWO_YEARS}",
        year_ago.day(),
        year_ago.month(),
        year_ago.year()
    )
}
