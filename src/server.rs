use crate::ics::get_ics;

use async_std::sync::Mutex;
use chrono::{DateTime, Duration, Utc};
use std::{collections::HashMap, sync::Arc};
use tide::{log, Request, Response};

pub struct CacheItem {
    ttl: DateTime<Utc>,
    ics: String,
}

pub type State = Arc<Mutex<HashMap<String, CacheItem>>>;

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

    if let Some(ics) = get_ics(key).await {
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

pub fn server() -> tide::Server<State> {
    let mut app = tide::with_state(Arc::new(Mutex::new(HashMap::new())));

    app.with(log::LogMiddleware::new());
    app.at("/:key").get(handle_request);
    app.at("/").get(|_| async {
        Ok(Response::builder(301)
            .header("Location", env!("CARGO_PKG_REPOSITORY"))
            .build())
    });

    app
}
