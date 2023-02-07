use anyhow::{anyhow, Context, Result};
use chrono::{Duration, NaiveDate, NaiveTime, Utc};
use clap::{crate_name, Arg, Command};
use futures::future;
use ics::{
    properties::{DtEnd, DtStart, Location, Organizer, RRule, Summary, TzName},
    Daylight, ICalendar, TimeZone,
};
use log::{debug, error};
use once_cell::sync::Lazy;
use scraper::{ElementRef, Html, Selector};
use std::env;
use std::num::ParseIntError;
use tide::{log::info, Request, Response};
use url::Url;

macro_rules! selector {
    ($name:ident, $query:expr) => {
        static $name: Lazy<Selector> = Lazy::new(|| Selector::parse($query).unwrap());
    };
}

const BASE_URL: &str = "https://rapla.dhbw-stuttgart.de/rapla";

selector!(DIV_SELECTOR, "div");
selector!(TD_SELECTOR, "td");
selector!(STRONG_SELECTOR, "strong");
selector!(CALENDAR_SELECTOR, "#calendar");
selector!(BLOCK_SELECTOR, "td.week_block");
selector!(TABLE_SELECTOR, "table.infotable");
selector!(RESOURCE_SELECTOR, "span.resource");
selector!(YEAR_SELECTOR, "select[name=year] > option[selected]");
selector!(WEEK_HEADER_SELECTOR, "td.week_header > nobr");
selector!(PERSON_SELECTOR, "span.person");

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Event {
    date: NaiveDate,
    start: NaiveTime,
    end: NaiveTime,
    title: String,
    location: String,
    lecturers: String,
}

impl<'a> From<Event> for ics::Event<'a> {
    fn from(event: Event) -> ics::Event<'a> {
        let start = format!(
            "{}T{}00",
            event.date.format("%Y%m%d"),
            event.start.format("%H%M")
        );

        let end = format!(
            "{}T{}00",
            event.date.format("%Y%m%d"),
            event.end.format("%H%M")
        );

        let id = format!("{}_{}", start, event.title.replace(' ', "-"));

        let mut ics_event = ics::Event::new(id, start.clone());

        ics_event.push(DtStart::new(start));
        ics_event.push(DtEnd::new(end));

        ics_event.push(Summary::new(event.title));
        ics_event.push(Organizer::new(event.lecturers));
        ics_event.push(Location::new(event.location));

        ics_event
    }
}

struct Page {
    html: Html,
}

impl Page {
    pub async fn fetch<S: AsRef<str>>(url: S) -> Result<Page> {
        Url::parse(url.as_ref())?;

        debug!("GET {}", url.as_ref());
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

        Ok(Page {
            html: Html::parse_document(body.as_str()),
        })
    }

    pub fn extract_events(self) -> Result<Vec<Event>> {
        self.html
            .select(&CALENDAR_SELECTOR)
            .next()
            .ok_or_else(|| anyhow!("Page does not contain a calendar"))?;

        debug!("Cleared calendar selection");

        let year = self.parse_year()?;
        debug!("Year: {year}");

        let (day, month) = self.parse_week_start()?;
        debug!("Day: {day}, Month: {month}");

        let week_start = NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| anyhow!("Failed to construct week start date"))?;

        let mut events = Vec::new();

        for block in self.html.select(&BLOCK_SELECTOR) {
            if let Some(event) = Self::parse_block(block, &week_start)
                .context(format!("Week start: {week_start}"))?
            {
                debug!("Successfully parsed block");
                events.push(event);
            } else {
                debug!("Skipped block");
            }
        }

        debug!("Parsed all blocks");

        Ok(events)
    }

    fn parse_year(&self) -> Result<i32> {
        let year_raw = self
            .html
            .select(&YEAR_SELECTOR)
            .next()
            .ok_or_else(|| anyhow!("No selected year element"))?
            .inner_html();

        debug!("Raw year: {year_raw}");

        year_raw.parse().context("Parse year")
    }

    fn parse_week_start(&self) -> Result<(u32, u32)> {
        let mut day_month = self
            .html
            .select(&WEEK_HEADER_SELECTOR)
            .next()
            .ok_or_else(|| anyhow!("No week header found"))?
            .inner_html()
            .split(' ')
            .nth(1)
            .ok_or_else(|| anyhow!("Week header does not have second part"))?
            .trim_end_matches('.')
            .split('.')
            .map(|item| item.parse().map_err(|err: ParseIntError| anyhow!(err)))
            .collect::<Result<Vec<_>>>()
            .context("Week start parts did not parse to numbers")?
            .into_iter();

        let day = day_month
            .next()
            .ok_or_else(|| anyhow!("Week start does not contain day"))?;

        let month = day_month
            .next()
            .ok_or_else(|| anyhow!("Week start does not contain month"))?;

        Ok((day, month))
    }

    fn parse_block(block: ElementRef, week_start: &NaiveDate) -> Result<Option<Event>> {
        let table = block
            .select(&TABLE_SELECTOR)
            .next()
            .ok_or_else(|| anyhow!("No table inside block"))?;

        let event_type = block
            .select(&STRONG_SELECTOR)
            .next()
            .ok_or_else(|| anyhow!("No event type section"))?
            .inner_html()
            .to_lowercase();

        debug!("Event type: {event_type}");

        let title = table
            .select(&TD_SELECTOR)
            .nth(1)
            .ok_or_else(|| anyhow!("No second td element (title string) in table"))?
            .inner_html()
            // TODO: properly unescape html in the future
            .replace("&amp;", "&");

        debug!("Title: {title}");

        let times_raw = block
            .select(&DIV_SELECTOR)
            .nth(1)
            .ok_or_else(|| anyhow!("No second div element (time info string) in block"))?
            .inner_html();

        debug!("Raw times: {times_raw}");

        // `times_split` can follow three formats:
        // 1. "Mo 01.01.2000 00:00-00:00"
        // 2. "Mo 00:00-00:00 wöchentlich"
        // 3. "00:00-00:00 täglich"
        let mut times_split = times_raw.split(' ');

        let weekday_raw = times_split
            .next()
            .ok_or_else(|| anyhow!("No weekday element in times split"))?;

        debug!("Raw weekday {weekday_raw}");

        let maybe_weekday = match weekday_raw {
            "Mo" => Some(0),
            "Di" => Some(1),
            "Mi" => Some(2),
            "Do" => Some(3),
            "Fr" => Some(4),
            "Sa" => Some(5),
            // Sunday doesn't exist!!!
            _ => None,
        };

        let weekday = if let Some(weekday) = maybe_weekday {
            weekday
        } else {
            // Someone had a bad day and used the 3rd format. We don't care.
            return Ok(None);
        };

        let mut hours = times_split
            .find_map(|item| item.contains(':').then_some(item.splitn(2, '-')))
            .ok_or_else(|| anyhow!("No hours element in times split"))?;

        let start = NaiveTime::parse_from_str(
            hours
                .next()
                .ok_or_else(|| anyhow!("No first element in hours"))?,
            "%H:%M",
        )
        .context("Parse start time")?;

        debug!("Start time {start}");

        let end = NaiveTime::parse_from_str(
            hours
                .next()
                .ok_or_else(|| anyhow!("No second element in hours"))?,
            "%H:%M",
        )
        .context("Parse end time")?;

        debug!("End time {end}");

        let date = *week_start + Duration::days(weekday);

        debug!("Date {date}");

        let lecturers = match block
            .select(&PERSON_SELECTOR)
            .map(|lecturer| lecturer.inner_html().trim_end_matches(',').into())
            .collect::<Vec<String>>()
            .join(" & ")
        {
            lecturers if lecturers.is_empty() => "?".into(),
            lecturers => lecturers,
        };

        let location = if event_type.contains("online") {
            "Online".into()
        } else {
            match block
                .select(&RESOURCE_SELECTOR)
                .map(|resource| resource.inner_html())
                .collect::<Vec<String>>()
                .join(", ")
            {
                location if location.is_empty() => "?".into(),
                location => location,
            }
        };

        Ok(Some(Event {
            title,
            date,
            start,
            end,
            lecturers,
            location,
        }))
    }
}

fn create_ics_base(key: &'_ str) -> ICalendar<'_> {
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

    ics
}

async fn fetch_range_and_create_ics(
    key: &str,
    start: NaiveDate,
    end: NaiveDate,
) -> Result<ICalendar> {
    let handles = start
        .iter_weeks()
        .take_while(|date| *date < end)
        .map(|date| async move {
            Page::fetch(format!(
                "{}?key={}{}",
                BASE_URL,
                key,
                date.format("&day=%d&month=%m&year=%Y"),
            ))
            .await?
            .extract_events()
        });

    let weeks = future::join_all(handles)
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?;

    let mut ics = create_ics_base(key);

    for events in weeks {
        for event in events {
            ics.add_event(event.into());
        }
    }

    Ok(ics)
}

fn print_events(events: &mut [Event]) {
    events.sort();

    for (idx, event) in events.iter().enumerate() {
        let prev = if idx != 0 { events.get(idx - 1) } else { None };
        let next = events.get(idx + 1);

        if prev.is_none() || prev.unwrap().date != event.date {
            println!("{}", event.date);
        }

        println!("{} - {} {}", event.start, event.end, event.title);

        println!(
            "Location: {}, Lecturers: {}",
            event.location, event.lecturers
        );

        if next.is_none() || next.unwrap().date != event.date {
            println!();
        }
    }
}

async fn serve_ics() -> Result<()> {
    let port = env::var("PORT").unwrap_or_else(|_| "8080".into());
    let mut app = tide::new();

    app.at("/:key").get(|req: Request<()>| async move {
        let key = req.url().path().trim().trim_matches('/');

        // browsers are so annoying
        if key == "favicon.ico" {
            return Ok(Response::new(404));
        }

        info!("Incoming request for '${key}'");

        let now = Utc::now().date_naive();

        let response = match fetch_range_and_create_ics(
            key,
            now - Duration::weeks(25),
            now + Duration::weeks(25),
        )
        .await
        {
            Ok(ics) => {
                info!("Successfully scraped result for '{key}'");
                Response::builder(200).body(ics.to_string())
            }
            Err(err) => {
                error!("Failed to scrape result for '{key}': {err}");
                Response::builder(400).body(err.to_string())
            }
        }
        .build();

        info!("Sending response status: {}", response.status());
        Ok(response)
    });

    let url = format!("[::]:{port}");
    info!("Listening on {url}");
    app.listen(url).await?;

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
        .subcommand(Command::new("serve-ics"))
        .subcommand(Command::new("parse-url").arg(Arg::new("url").required(true).num_args(1)))
        .get_matches();

    setup_logging();

    match matches.subcommand() {
        Some(("serve-ics", _)) => serve_ics().await,
        Some(("parse-url", pmatches)) => {
            let url = pmatches.get_one::<String>("url").unwrap();
            let mut events = Page::fetch(url).await?.extract_events()?;
            print_events(&mut events);
            Ok(())
        }

        _ => unreachable!(),
    }
}
