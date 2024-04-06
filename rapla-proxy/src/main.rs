use std::{collections::HashMap, env, io, net::SocketAddr, process, sync::Arc};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Json, Router,
};
use chrono::{Datelike, Duration, Utc};
use serde::Deserialize;
use tokio::{net::TcpListener, sync::RwLock, task, time};

use rapla_parser::{parse_calendar, Calendar};

type Cache = Arc<RwLock<HashMap<String, Arc<Calendar>>>>;

const UPSTREAM: &str = "https://rapla.dhbw.de";
const CALENDAR_PATH: &str = "/rapla/calendar";

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
        .route(CALENDAR_PATH, get(handle_calendar))
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
    Query(query): Query<CalendarQuery>,
) -> Response {
    let Some(calendar) = fetch_calendar(query.key, query.salt, cache).await else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed proxy calendar").into_response();
    };

    if query.json {
        return Json(calendar.as_ref()).into_response();
    }

    return (
        [("content-type", "text/calendar")],
        calendar.to_ics().to_string(),
    )
        .into_response();
}

async fn fetch_calendar<'a>(key: String, salt: String, cache: Cache) -> Option<Arc<Calendar>> {
    let now = Utc::now();
    let year_ago = now - Duration::try_days(365).unwrap();

    let url = format!(
        "{UPSTREAM}/{CALENDAR_PATH}?key={key}&salt={salt}&day={}&month={}&year={}&pages=104",
        year_ago.day(),
        year_ago.month(),
        year_ago.year()
    );

    if let Some(calendar) = cache.read().await.get(&key) {
        return Some(Arc::clone(calendar));
    }

    let html = reqwest::get(&url).await.ok()?.text().await.ok()?;
    let calendar = Arc::new(parse_calendar(&html)?);

    cache
        .write()
        .await
        .insert(key.clone(), Arc::clone(&calendar));

    task::spawn(async move {
        time::sleep(time::Duration::from_secs(60 * 60)).await;
        cache.write().await.remove(&key);
    });

    Some(calendar)
}
