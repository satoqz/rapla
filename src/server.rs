use crate::ics::get_ics;

use async_std::sync::Mutex;
use chrono::{DateTime, Duration, Utc};
use std::{collections::HashMap, sync::Arc};
use tide::{log, Response};

pub struct CacheItem {
    ttl: DateTime<Utc>,
    ics: String,
}

pub type State = Arc<Mutex<HashMap<String, CacheItem>>>;

async fn handle_request(req: tide::Request<State>) -> tide::Result {
    let url = req.url();
    let key = url.path().trim().trim_matches('/').trim_end_matches(".ics");

    let mut cache = req.state().lock().await;
    let mut cache_hit = false;
    let now = Utc::now();

    let ics = if let Some(cached) = cache.get(key) {
        if cached.ttl < now {
            get_ics(key).await
        } else {
            cache_hit = true;
            Some(cached.ics.clone())
        }
    } else {
        get_ics(key).await
    };

    if let Some(ics) = ics {
        let response = Response::builder(200)
            .content_type("text/calendar")
            .body(ics.to_string())
            .build();

        if !cache_hit {
            cache.insert(
                key.into(),
                CacheItem {
                    ics,
                    ttl: now + Duration::hours(1),
                },
            );
        }

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

    app
}
