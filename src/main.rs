#![allow(unused)]

use anyhow::{anyhow, Result};
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use clap::{crate_name, Arg, Command};
use ics::{
    properties::{RRule, TzName},
    Daylight, ICalendar, TimeZone,
};
use log::{debug, error, info};
use once_cell::sync::Lazy;
use scraper::{Html, Selector};
use std::env;
use url::Url;

macro_rules! selector {
    ($name:ident, $query:expr) => {
        const $name: Lazy<Selector> = Lazy::new(|| Selector::parse($query).unwrap());
    };
}

selector!(DIV_SELECTOR, "div");
selector!(TD_SELECTOR, "td");
selector!(STRONG_SELECTOR, "strong");
selector!(CALENDAR_SELECTOR, "#calendar");
selector!(BLOCK_SELECTOR, "td.week_block");
selector!(INFO_TABLE_SELECTOR, "table.infotable");
selector!(RESOURCE_SELECTOR, "span.resource");

struct Event {
    title: String,
    date: NaiveDate,
    start: NaiveTime,
    end: NaiveTime,
    location: String,
    lecturer: String,
}

struct Page(Html);

impl Event {
    pub fn to_ics_event<'a>(self) -> ics::Event<'a> {
        let ics_event = ics::Event::new("", "");

        ics_event
    }
}

impl Page {
    pub async fn fetch(url: &String) -> Result<Page> {
        Url::parse(&url)?;

        debug!("Sending HTTP request");
        let mut resp = surf::get(url).await.map_err(|err| anyhow!(err))?;

        let status = resp.status();
        if status != 200 {
            return Err(anyhow!("Got response status {status}"));
        }

        debug!("Reading response body");
        let body = resp
            .take_body()
            .into_string()
            .await
            .map_err(|err| anyhow!(err))?;

        Ok(Page(Html::parse_document(body.as_str())))
    }

    pub fn extract_events(self) -> Result<Vec<Event>> {
        self.0
            .select(&CALENDAR_SELECTOR)
            .next()
            .ok_or(anyhow!("Page does not contain a calendar"))?;

        Ok(vec![])
    }
}

fn build_ics<'a>(events: Vec<Event>, key: &'a str) -> ICalendar<'a> {
    let mut cest = Daylight::new("19700329T020000", "+0100", "+0200");
    cest.push(TzName::new("CEST"));
    cest.push(RRule::new("FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU"));

    let mut cet = ics::Standard::new("19701025T030000", "+0200", "+0100");
    cet.push(TzName::new("CET"));
    cet.push(RRule::new("FREQ=YEARLY;BYMONTH=10;BYDAY=-1SU"));

    let mut timezone = TimeZone::daylight("Europe/Berlin", cest);
    timezone.add_standard(cet);

    let mut ics = ICalendar::new("2.0", key);
    ics.add_timezone(timezone);

    for event in events {
        ics.add_event(event.to_ics_event());
    }

    ics
}

fn print_events(events: &Vec<Event>) {}

async fn serve() -> Result<()> {
    Ok(())
}

fn setup_logging() {
    if env::var("LOG").is_err() {
        env::set_var("LOG", "rapla_to_ics=info");
    }

    pretty_env_logger::init_custom_env("LOG");
}

#[async_std::main]
async fn main() -> Result<()> {
    let matches = Command::new(crate_name!())
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand_required(true)
        .subcommand(Command::new("serve"))
        .subcommand(Command::new("fetch").arg(Arg::new("url").required(true).num_args(1)))
        .get_matches();

    setup_logging();

    match matches.subcommand() {
        Some(("serve", _)) => serve().await,

        Some(("fetch", fetch_matches)) => {
            let url = fetch_matches.get_one::<String>("url").unwrap();
            let events = Page::fetch(url).await?.extract_events()?;
            print_events(&events);
            Ok(())
        }

        _ => unreachable!(),
    }
}
