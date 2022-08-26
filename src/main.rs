mod ics;
mod scraper;
mod server;
mod utils;

use server::server;
use std::env;

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    let port = env::var("PORT").unwrap_or_else(|_| "8000".into());
    let app = server();
    app.listen(format!("0.0.0.0:{port}")).await
}
