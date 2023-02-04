use crate::ics::scrape_ics;

use async_std::{sync::Mutex, task};
use chrono::{DateTime, Duration, Utc};
use std::{collections::HashMap, env, sync::Arc};
use tide::{log, Request, Response, Server};

mod ics;
mod scraper;
mod utils;

struct CacheItem {
    ttl: DateTime<Utc>,
    ics: String,
}

type State = Arc<Mutex<HashMap<String, CacheItem>>>;

async fn handle_request(req: Request<State>) -> tide::Result {
    let key = req
        .url()
        .path()
        .trim()
        .trim_matches('/')
        .trim_end_matches(".ics");

    let mut cache = req.state().lock().await;

    let now = Utc::now();

    if let Some(cache_hit) = cache.get(key) {
        if cache_hit.ttl > now {
            return Ok(Response::builder(200)
                .content_type("text/calendar")
                .body(cache_hit.ics.as_str())
                .build());
        }
    }

    if let Some(ics) = scrape_ics(key).await {
        let cache_item = CacheItem {
            ttl: now + Duration::hours(1),
            ics,
        };

        let response = Response::builder(200)
            .content_type("text/calendar")
            .body(cache_item.ics.as_str())
            .build();

        cache.insert(key.into(), cache_item);

        Ok(response)
    } else {
        Ok(Response::builder(400)
            .body(format!("Error: Invalid Key ({key})\n"))
            .build())
    }
}

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    json_env_logger::init();

    let port = env::var("PORT").unwrap_or_else(|_| "8080".into());
    let cache = Arc::new(Mutex::new(HashMap::new()));
    let mut app: Server<State> = tide::with_state(cache.clone());

    app.with(log::LogMiddleware::new());
    app.at("/:key").get(handle_request);
    app.at("/").get(|_| async {
        Ok(Response::builder(301)
            .header("Location", env!("CARGO_PKG_REPOSITORY"))
            .build())
    });

    let app_handle = app.listen(format!("0.0.0.0:{port}"));
    let sweeper_handle = task::spawn(async move {
        let hour = Duration::hours(1).to_std().unwrap();
        loop {
            task::sleep(hour).await;
            let now = Utc::now();
            cache.lock().await.retain(|_, value| value.ttl > now);
        }
    });

    futures::join!(app_handle, sweeper_handle).0
}
