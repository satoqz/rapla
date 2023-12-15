#![cfg(feature = "proxy")]

use chrono::{DateTime, Datelike, Duration, Utc};
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use rapla::Calendar;
use tokio::sync::Mutex;

use std::{collections::HashMap, convert::Infallible, env, net, process, sync::Arc};

#[tokio::main]
async fn main() {
    let Ok(port) = env::var("PORT").map_or_else(|_| Ok(8080), |port| port.parse::<u16>()) else {
        eprintln!("`PORT` environment variable is invalid");
        process::exit(1);
    };

    let Ok(ip) = env::var("IP").map_or_else(
        |_| Ok(net::IpAddr::V4(net::Ipv4Addr::from([127, 0, 0, 1]))),
        |ip| ip.parse::<net::IpAddr>(),
    ) else {
        eprintln!("`IP` environment variable is invalid");
        process::exit(1);
    };

        let cache = Arc::new(Mutex::new(HashMap::new()));

    let make_service = make_service_fn(|_conn| {
        let cache_clone = Arc::clone(&cache);
        async move {
        Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                handle_request(req, Arc::clone(&cache_clone))
        }))
        }
    });

    let addr = net::SocketAddr::from((ip, port));
    let server = Server::bind(&addr).serve(make_service);
    eprintln!("listening on http://{addr}");

    if let Err(err) = server.await {
        eprintln!("server error: {}", err);
    }
}

async fn fetch_calendar(url: &str) -> Option<Calendar> {
    let res = reqwest::get(url).await.ok()?;
    let html = res.text().await.ok()?;
    Calendar::from_html(html.as_str())
}

async fn handle_request(
    req: Request<Body>,
    cache: Arc<Mutex<HashMap<String, (DateTime<Utc>, Calendar)>>>,
) -> Result<Response<String>, Infallible> {
    let builder = Response::builder();

    let query_pairs =
        form_urlencoded::parse(req.uri().query().unwrap_or("").as_bytes()).collect::<Vec<_>>();

    let Some((_, key)) = query_pairs.iter().find(|pair| pair.0 == "key") else {
        return Ok(builder
            .status(400)
            .body("missing `key` query parameter".into())
            .unwrap());
    };

    let Some((_, salt)) = query_pairs.iter().find(|pair| pair.0 == "salt") else {
        return Ok(builder
            .status(400)
            .body("missing `salt` query parameter".into())
            .unwrap());
    };

    let return_json = query_pairs
        .iter()
        .find(|pair| pair.0 == "json" && pair.1 == "true")
        .is_some();

    let now = Utc::now();
    let year_ago = now - Duration::days(365);

    let url = format!(
        "https://rapla.dhbw.de/rapla/calendar?key={key}&salt={salt}&day={}&month={}&year={}&pages=104",
        year_ago.day(), year_ago.month(), year_ago.year()
    );

    let cached_calendar = match cache.lock().await.get(&url) {
        Some((expiry_time, _)) if *expiry_time < now => {
            cache.lock().await.remove(&url);
            None
        }
        Some((_, calendar)) => Some(calendar.clone()),
        None => None,
    };

    let calendar = match cached_calendar {
        Some(calendar) => calendar,
        None => match fetch_calendar(url.as_str()).await {
            Some(calendar) => {
                cache
                    .lock()
                    .await
                    .insert(url, (now + Duration::minutes(10), calendar.clone()));
                calendar
            }
            None => return Ok(builder.status(500).body("no events".into()).unwrap()),
        },
    };

    Ok(if return_json {
        builder
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&calendar).unwrap())
            .unwrap()
    } else {
        builder
            .header("Content-Type", "text/calendar")
            .body(calendar.to_ics().to_string())
            .unwrap()
    })
}
