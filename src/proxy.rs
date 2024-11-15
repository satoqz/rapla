use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use chrono::{Datelike, Duration, Utc};
use serde::Deserialize;

use crate::parser::parse_calendar;
use crate::structs::Calendar;

const UPSTREAM: &str = "https://rapla.dhbw.de";

pub fn router(cache_config: Option<crate::cache::Config>) -> Router {
    let router = Router::new().route("/:calendar_path", get(handle_calendar));
    if let Some(cache_config) = cache_config {
        crate::cache::apply_middleware(router, cache_config)
    } else {
        router
    }
}

#[derive(Deserialize)]
struct CalendarQuery {
    key: String,
    salt: String,
}

async fn handle_calendar(
    Path(calendar_path): Path<String>,
    Query(CalendarQuery { key, salt }): Query<CalendarQuery>,
) -> Response {
    let calendar = fetch_calendar(calendar_path, key, salt).await;

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

async fn fetch_calendar(calendar_path: String, key: String, salt: String) -> Option<Calendar> {
    let url = generate_upstream_url(&calendar_path, &key, &salt);
    eprintln!("{url}");
    let html = reqwest::get(url).await.ok()?.text().await.ok()?;
    parse_calendar(&html)
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
