use std::{collections::HashMap, convert::Infallible, env, net::SocketAddr, process, sync::Arc};

use chrono::{DateTime, Datelike, Duration, Utc};
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use tokio::sync::Mutex;

use rapla_parser::Calendar;

#[tokio::main]
async fn main() {
    const RAPLA_PROXY_ADDR: &str = "RAPLA_PROXY_ADDR";

    let Ok(addr) = env::var(RAPLA_PROXY_ADDR).map_or_else(
        |_| Ok(SocketAddr::from(([127, 0, 0, 1], 8080))),
        |value| value.parse(),
    ) else {
        eprintln!("failed to parse `{RAPLA_PROXY_ADDR}` environment variable");
        process::exit(1)
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

    let server = Server::bind(&addr).serve(make_service);
    eprintln!("listening on http://{addr}");

    if let Err(err) = server.await {
        eprintln!("server error: {err}");
    }
}

type Cache = Arc<Mutex<HashMap<String, (DateTime<Utc>, Arc<Calendar>)>>>;

async fn handle_request(req: Request<Body>, cache: Cache) -> Result<Response<String>, Infallible> {
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
        .any(|pair| pair.0 == "json" && pair.1 == "true");

    let now = Utc::now();
    let year_ago = now - Duration::try_days(365).unwrap();

    let url = format!(
        "https://rapla.dhbw.de/rapla/calendar?key={key}&salt={salt}&day={}&month={}&year={}&pages=104",
        year_ago.day(), year_ago.month(), year_ago.year()
    );

    let Some(calendar) = fetch_calendar(url, cache).await else {
        return Ok(builder.status(500).body("no events".into()).unwrap());
    };

    Ok(if return_json {
        builder
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(calendar.as_ref()).unwrap())
            .unwrap()
    } else {
        builder
            .header("Content-Type", "text/calendar")
            .body(calendar.to_ics().to_string())
            .unwrap()
    })
}

async fn fetch_calendar<'a>(url: String, cache: Cache) -> Option<Arc<Calendar>> {
    let now = Utc::now();

    {
        let mut cache = cache.lock().await;
        if let Some((ttl, calendar)) = cache.get(&url) {
            if *ttl > now {
                return Some(Arc::clone(calendar));
            }
            cache.remove(&url);
        }
    }

    let html = reqwest::get(&url).await.ok()?.text().await.ok()?;
    let calendar = Arc::new(Calendar::from_html(html.as_str())?);

    cache.lock().await.insert(
        url,
        (
            now + Duration::try_minutes(10).unwrap(),
            Arc::clone(&calendar),
        ),
    );

    Some(calendar)
}
