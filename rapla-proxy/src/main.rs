use std::{collections::HashMap, env, io, net::SocketAddr, process, sync::Arc};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Json, Router,
};
use chrono::{Datelike, Duration, Utc};
use serde::Deserialize;
use tokio::{net::TcpListener, sync::RwLock, task, time};

use rapla_parser::{parse_calendar, Calendar};

type Cache = Arc<RwLock<HashMap<(String, String), Arc<Calendar>>>>;

const UPSTREAM: &str = "https://rapla.dhbw.de";

#[tokio::main]
async fn main() -> io::Result<()> {
    const RAPLA_PROXY_ADDR: &str = "RAPLA_PROXY_ADDR";

    let Ok(addr) = env::var(RAPLA_PROXY_ADDR).map_or_else(
        |_| Ok(SocketAddr::from(([127, 0, 0, 1], 8080))),
        |value| value.parse(),
    ) else {
        eprintln!("Failed to parse `{RAPLA_PROXY_ADDR}` environment variable");
        process::exit(1);
    };

    let router = Router::new()
        .route("/rapla/:calendar_path", get(handle_calendar))
        .fallback(|| async { Redirect::permanent(env!("CARGO_PKG_REPOSITORY")) })
        .with_state(Arc::new(RwLock::new(HashMap::new())));

    let listener = TcpListener::bind(addr).await?;
    eprintln!("Listening at http://{addr}");
    axum::serve(listener, router).await
}

#[derive(Deserialize)]
struct CalendarQuery {
    // Forwarded components
    key: String,
    salt: String,
    // Custom components
    #[serde(default)]
    json: bool,
}

async fn handle_calendar(
    State(cache): State<Cache>,
    Path(calendar_path): Path<String>,
    Query(CalendarQuery { key, salt, json }): Query<CalendarQuery>,
) -> Response {
    let Some(calendar) = fetch_calendar_with_cache(calendar_path, key, salt, cache).await else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to proxy calendar",
        )
            .into_response();
    };

    if json {
        return (
            [("content-type", "application/json; charset=utf-8")],
            Json(calendar.as_ref()),
        )
            .into_response();
    }

    return (
        [("content-type", "text/calendar")],
        calendar.to_ics().to_string(),
    )
        .into_response();
}

async fn fetch_calendar_with_cache(
    calendar_path: String,
    key: String,
    salt: String,
    cache: Cache,
) -> Option<Arc<Calendar>> {
    let cache_key = (calendar_path, key);

    if let Some(calendar) = cache.read().await.get(&cache_key) {
        return Some(Arc::clone(calendar));
    }

    let upstream_url = generate_upstream_url(&cache_key.0, &cache_key.1, &salt);
    let html = reqwest::get(upstream_url).await.ok()?.text().await.ok()?;
    let calendar = Arc::new(parse_calendar(&html)?);

    cache
        .write()
        .await
        .insert(cache_key.clone(), Arc::clone(&calendar));

    task::spawn(async move {
        time::sleep(time::Duration::from_secs(60 * 60)).await;
        cache.write().await.remove(&cache_key);
    });

    Some(calendar)
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
