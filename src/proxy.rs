use std::fmt::{self, Display};

use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use chrono::{Datelike, Duration, Utc};
use serde::Deserialize;

use crate::parser::parse_calendar;
use crate::structs::Calendar;

#[derive(Deserialize)]
struct CalendarQuery {
    key: String,
    salt: String,
}

enum Error {
    UpstreamConnection(reqwest::Error),
    UpstreamStatus(reqwest::Url, reqwest::StatusCode),
    UpstreamBody(reqwest::Url, reqwest::StatusCode),
    Parse(reqwest::Url, reqwest::StatusCode),
}

impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::UpstreamConnection(_) => StatusCode::BAD_GATEWAY,
            Self::UpstreamStatus(_, status) => {
                StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY)
            }
            Self::UpstreamBody(_, _) => StatusCode::BAD_GATEWAY,
            Self::Parse(_, _) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UpstreamConnection(err) => write!(
                f,
                "Could not connect to upstream at {}\n",
                err.url()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| String::from("<No URL found?>"))
            ),
            Error::UpstreamStatus(url, status) => {
                write!(
                    f,
                    "Upstream (status {status}) returned unsuccessful status code at {url}",
                )
            }
            Error::UpstreamBody(url, status) => {
                write!(
                    f,
                    "Upstream (status {status}) returned invalid body at {url}"
                )
            }
            Error::Parse(url, status) => write!(
                f,
                "Upstream (status {status}) returned HTML that did not parse at {url}"
            ),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (
            self.status_code(),
            [("content-type", "text/plain")],
            self.to_string(),
        )
            .into_response()
    }
}

pub fn router(cache_config: Option<crate::cache::Config>) -> Router {
    let router = Router::new().route("/:calendar_path", get(handle_calendar));
    if let Some(cache_config) = cache_config {
        crate::cache::apply_middleware(router, cache_config)
    } else {
        router
    }
}

async fn handle_calendar(
    Path(calendar_path): Path<String>,
    Query(CalendarQuery { key, salt }): Query<CalendarQuery>,
) -> Response {
    let calendar = fetch_calendar(calendar_path, key, salt).await;

    match calendar {
        Ok(calendar) => (
            [("content-type", "text/calendar")],
            calendar.to_ics().to_string(),
        )
            .into_response(),
        Err(err) => err.into_response(),
    }
}

async fn fetch_calendar(
    calendar_path: String,
    key: String,
    salt: String,
) -> Result<Calendar, Error> {
    let url = generate_upstream_url(&calendar_path, &key, &salt);
    eprintln!("{url}");

    let response = reqwest::get(url)
        .await
        .map_err(|err| Error::UpstreamConnection(err))?;

    let response_url = response.url().clone();
    let response_status = response.status();

    if !response_status.is_success() {
        return Err(Error::UpstreamStatus(response_url, response_status));
    }

    let html = match response.text().await {
        Ok(html) => html,
        Err(_) => return Err(Error::UpstreamBody(response_url, response_status)),
    };

    parse_calendar(&html).ok_or(Error::Parse(response_url, response_status))
}

fn generate_upstream_url(calendar_path: &str, key: &str, salt: &str) -> String {
    // these don't need to be 100% accurate
    const WEEKS_TWO_YEARS: usize = 104;
    const DAYS_ONE_YEAR: i64 = 365;

    let now = Utc::now();
    let year_ago = now - Duration::try_days(DAYS_ONE_YEAR).unwrap();

    const UPSTREAM: &str = "https://rapla.dhbw.de";

    format!(
        "{UPSTREAM}/rapla/{calendar_path}?key={key}&salt={salt}&day={}&month={}&year={}&pages={WEEKS_TWO_YEARS}",
        year_ago.day(),
        year_ago.month(),
        year_ago.year()
    )
}
