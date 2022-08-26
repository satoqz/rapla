mod ics;
mod scraper;
mod server;
mod utils;

use server::{server, State};

#[shuttle_service::main]
async fn tide() -> shuttle_service::ShuttleTide<State> {
    Ok(server())
}
