use anyhow::{bail, Context, Result};
use axum::{extract::Path, http::StatusCode, response::Response, routing, Router, Server};
use chrono::{Duration, NaiveDate, NaiveTime, Utc};
use futures::future;
use ics::{
    properties::{DtEnd, DtStart, Location, Organizer, RRule, Summary, TzName},
    Daylight, ICalendar, TimeZone,
};
use log::{debug, error, info, warn};
use once_cell::sync::Lazy;
use scraper::{ElementRef, Html, Selector};
use std::{env, process};
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
    pub async fn fetch(url: &String) -> Result<Page> {
        Url::parse(url.as_ref())?;

        debug!("GET {}", url);
        let resp = reqwest::get(url).await.map_err(anyhow::Error::from)?;

        let status = resp.status();
        if status != 200 {
            bail!("Got response status {status}");
        }

        debug!("Reading response body");
        let body = resp.text().await.map_err(anyhow::Error::from)?;

        Ok(Page {
            html: Html::parse_document(body.as_str()),
        })
    }

    pub fn extract_events(self) -> Result<Vec<Event>> {
        self.html
            .select(&CALENDAR_SELECTOR)
            .next()
            .context("Page does not contain a calendar")?;

        debug!("Cleared calendar selection");

        let year = self.parse_year()?;
        debug!("Year: {year}");

        let (day, month) = self.parse_week_start()?;
        debug!("Day: {day}, Month: {month}");

        let week_start = NaiveDate::from_ymd_opt(year, month, day)
            .context("Failed to construct week start date")?;

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
            .context("No selected year element")?
            .inner_html();

        debug!("Raw year: {year_raw}");

        year_raw.parse().context("Parse year")
    }

    fn parse_week_start(&self) -> Result<(u32, u32)> {
        let mut day_month = self
            .html
            .select(&WEEK_HEADER_SELECTOR)
            .next()
            .context("No week header found")?
            .inner_html()
            .split(' ')
            .nth(1)
            .context("Week header does not have second part")?
            .trim_end_matches('.')
            .split('.')
            .map(|item| item.parse().map_err(anyhow::Error::from))
            .collect::<Result<Vec<_>>>()
            .context("Week start parts did not parse to numbers")?
            .into_iter();

        let day = day_month
            .next()
            .context("Week start does not contain day")?;

        let month = day_month
            .next()
            .context("Week start does not contain month")?;

        Ok((day, month))
    }

    fn parse_block(block: ElementRef, week_start: &NaiveDate) -> Result<Option<Event>> {
        let table = block
            .select(&TABLE_SELECTOR)
            .next()
            .context("No table inside block")?;

        let event_type = block
            .select(&STRONG_SELECTOR)
            .next()
            .context("No event type section")?
            .inner_html()
            .to_lowercase();

        debug!("Event type: {event_type}");

        let title = table
            .select(&TD_SELECTOR)
            .nth(1)
            .context("No second td element (title string) in table")?
            .inner_html()
            // TODO: properly unescape html, probably overkill
            .replace("&amp;", "&");

        debug!("Title: {title}");

        let times_raw = block
            .select(&DIV_SELECTOR)
            .nth(1)
            .context("No second div element (time info string) in block")?
            .inner_html();

        debug!("Raw times: {times_raw}");

        // `times_raw` can follow three formats:
        // 1. "Mo 01.01.2000 00:00-00:00"
        // 2. "Mo 00:00-00:00 wöchentlich"
        // 3. "00:00-00:00 täglich"

        let mut times_split = times_raw.split(' ');

        let weekday_raw = times_split
            .next()
            .context("No weekday element in times split")?;

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
            .context("No hours element in times split")?;

        let start =
            NaiveTime::parse_from_str(hours.next().context("No first element in hours")?, "%H:%M")
                .context("Parse start time")?;

        debug!("Start time {start}");

        let end =
            NaiveTime::parse_from_str(hours.next().context("No second element in hours")?, "%H:%M")
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
            Page::fetch(&format!(
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

async fn run_server() -> Result<()> {
    let app = Router::new().route(
        "/:key",
        routing::get(|Path(key): Path<String>| async move {
            let resp = Response::builder();

            if key == "favicon.ico" {
                return resp
                    .status(StatusCode::NOT_FOUND)
                    .body("Not found".into())
                    .unwrap();
            }

            let now = Utc::now().date_naive();

            let ics = fetch_range_and_create_ics(
                key.as_str(),
                now - Duration::weeks(25),
                now + Duration::weeks(25),
            )
            .await;

            match ics {
                Ok(ics) => {
                    info!("Successfully scraped result for '{key}'");
                    resp.status(StatusCode::OK)
                        .header("Content-Type", "text/calendar")
                        .body(ics.to_string())
                        .unwrap()
                }
                Err(err) => {
                    error!("Failed to scrape result for '{key}': {err}");
                    resp.status(StatusCode::BAD_REQUEST)
                        .body("Bad Request".into())
                        .unwrap()
                }
            }
        }),
    );

    let port = env::var("PORT").unwrap_or_else(|_| "8080".into());
    let url = format!("[::]:{port}");
    info!("Listening on {url}");

    Server::bind(&url.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .context("Failed to start server")
}

#[tokio::main]
async fn main() -> Result<()> {
    if env::var("LOG").is_err() {
        env::set_var("LOG", "rapla_to_ics=info");
    }

    pretty_env_logger::init_custom_env("LOG");

    ctrlc::set_handler(|| {
        warn!("Received SIGTERM");
        process::exit(1);
    })
    .unwrap();

    run_server().await
}
