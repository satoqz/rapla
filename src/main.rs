mod rapla;

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};

use std::{convert::Infallible, env, net, process};

#[cfg(feature = "bind-wildcard")]
const IP: [u8; 4] = [0; 4];
#[cfg(not(feature = "bind-wildcard"))]
const IP: [u8; 4] = [127, 0, 0, 1];

#[tokio::main]
async fn main() {
    let Ok(port) = env::var("PORT").map_or_else(|_| Ok(8080), |port| port.parse::<u16>()) else {
        eprintln!("`PORT` environment variable is invalid");
        process::exit(1);
    };

    let addr = net::SocketAddr::from((IP, port));

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
    let path_and_query = req.uri().path_and_query().unwrap();
    let Some(ics) = get_ics(format!("https://rapla.dhbw.de/{path_and_query}")).await else {
        return Ok(Response::builder().body("error".into()).unwrap());
    };

    Ok(Response::builder()
        .header("Content-Type", "text/calendar")
        .body(ics.to_string())
        .unwrap())
}
