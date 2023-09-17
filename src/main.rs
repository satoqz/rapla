mod rapla;

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};

use std::{convert::Infallible, env, net, process};

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

    let addr = net::SocketAddr::from((ip, port));

    let make_service =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_request)) });

    let server = Server::bind(&addr).serve(make_service);
    eprintln!("listening on http://{addr}");

    if let Err(err) = server.await {
        eprintln!("server error: {}", err);
    }
}

async fn get_ics<'a>(url: String) -> Option<ics::ICalendar<'a>> {
    let res = reqwest::get(&url).await.ok()?;
    let html = res.text().await.ok()?;

    let mut ics = rapla::ics_base(url);
    for event in rapla::extract_events(html)? {
        ics.add_event(event.into());
    }

    Some(ics)
}

async fn handle_request(req: Request<Body>) -> Result<Response<String>, Infallible> {
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

    let url = format!("https://rapla.dhbw.de/rapla/calendar?key={key}&salt={salt}&pages=20");

    let Some(ics) = get_ics(url).await else {
        return Ok(builder.status(500).body("no events".into()).unwrap());
    };

    Ok(builder
        .header("Content-Type", "text/calendar")
        .body(ics.to_string())
        .unwrap())
}
