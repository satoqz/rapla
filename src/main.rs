use rapla_proxy::*;

use anyhow::Result;
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

    let addr = net::SocketAddr::from(([127, 0, 0, 1], port));

    let make_service =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_request)) });

    let server = Server::bind(&addr).serve(make_service);
    eprintln!("listening on http://{addr}");

    if let Err(err) = server.await {
        eprintln!("server error: {}", err);
    }
}

async fn get_ics<'a>(url: String) -> Result<ics::ICalendar<'a>> {
    let res = reqwest::get(&url).await?;
    let html = res.text().await?;

    let mut ics = ics_base(url);
    for event in extract_html(html)? {
        ics.add_event(event.into());
    }

    Ok(ics)
}

async fn handle_request(req: Request<Body>) -> Result<Response<String>, Infallible> {
    let path_and_query = req.uri().path_and_query().unwrap();

    let ics = match get_ics(format!("https://rapla.dhbw.de/{path_and_query}")).await {
        Ok(ics) => ics,
        Err(err) => return Ok(Response::builder().body(format!("Error: {err}")).unwrap()),
    };

    Ok(Response::builder()
        .header("Content-Type", "text/calendar")
        .body(ics.to_string())
        .unwrap())
}
