use std::borrow::Cow;

use anyhow::{Context, Result};
use chrono::{Duration, NaiveDate, NaiveTime};
use ics::{
    properties::{DtEnd, DtStart, Location, RRule, Summary, TzName},
    Daylight, ICalendar, Standard, TimeZone,
};
use once_cell::sync::Lazy;
use scraper::{ElementRef, Html, Selector};

macro_rules! selector {
    ($name:ident, $query:expr) => {
        static $name: Lazy<Selector> = Lazy::new(|| Selector::parse($query).unwrap());
    };
}

selector!(START_YEAR, "select[name=year] > option[selected]");
selector!(WEEK_NUMBER, "th.week_number");
selector!(WEEKS, "div.calendar > table.week_table > tbody");
selector!(START_DATE, "tr > td.week_header > nobr");
selector!(ROWS, "tr");
selector!(COLUMNS, "td");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Event<'a> {
    pub date: NaiveDate,
    pub start: NaiveTime,
    pub end: NaiveTime,
    pub title: &'a str,
    pub location: &'a str,
}

impl<'a> From<Event<'a>> for ics::Event<'a> {
    fn from(event: Event<'a>) -> ics::Event<'a> {
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
        ics_event.push(Location::new(event.location));

        ics_event
    }
}

pub fn ics_base<'a, S: Into<Cow<'a, str>>>(name: S) -> ICalendar<'a> {
    let mut cet_standard = Standard::new("19701025T030000", "+0200", "+0100");
    cet_standard.push(TzName::new("CET"));
    cet_standard.push(RRule::new("FREQ=YEARLY;BYMONTH=10;BYDAY=-1SU"));

    let mut cest_daylight = Daylight::new("19700329T020000", "+0100", "+0200");
    cest_daylight.push(TzName::new("CEST"));
    cest_daylight.push(RRule::new("FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU"));

    let mut timezone = TimeZone::daylight("Europe/Berlin", cest_daylight);
    timezone.add_standard(cet_standard);

    let mut ics = ICalendar::new("2.0", name);
    ics.add_timezone(timezone);

    ics
}

pub fn extract_html<'a, S: AsRef<str>>(html: S) -> Result<Vec<Event<'a>>> {
    let html = Html::parse_document(html.as_ref());
    let mut events = Vec::new();

    let mut year = html
        .select(&START_YEAR)
        .next()
        .context("ahh")?
        .inner_html()
        .parse::<i32>()?;

    for week in html.select(&WEEKS) {
        let week_number = week
            .select(&WEEK_NUMBER)
            .next()
            .context("ahh")?
            .inner_html()
            .split(' ')
            .nth(1)
            .context("ahh")?
            .parse::<usize>()?;

        if week_number == 1 {
            year += 1;
        }

        events.append(&mut extract_week(week, year)?);
    }

    Ok(events)
}

fn extract_week<'a>(element: ElementRef, start_year: i32) -> Result<Vec<Event<'a>>> {
    let mut events = Vec::new();

    let start_date_raw = element
        .select(&START_DATE)
        .next()
        .context("ahh")?
        .inner_html();

    let mut day_month = start_date_raw
        .split(' ')
        .nth(1)
        .context("ahh")?
        .trim_end_matches('.')
        .split('.')
        .into_iter();

    let start_day = day_month.next().context("ahh")?.parse::<u32>()?;
    let start_month = day_month.next().context("ahh")?.parse::<u32>()?;

    for row in element.select(&ROWS).skip(1) {
        events.append(&mut extract_row(row, start_year, start_month, start_day)?);
    }

    Ok(events)
}

fn extract_row<'a>(
    element: ElementRef,
    start_year: i32,
    start_month: u32,
    start_day: u32,
) -> Result<Vec<Event<'a>>> {
    let mut events = Vec::new();

    let monday = NaiveDate::from_ymd_opt(start_year, start_month, start_day).context("wahh")?;
    let mut day_index = 0;

    for column in element.select(&COLUMNS) {
        let class = column.value().classes().next().context("wahh")?;

        if class.starts_with("week_separatorcell") {
            day_index += 1;
        }

        if class != "week_block" {
            continue;
        }

        let date = monday + Duration::days(day_index);
        events.push(extract_event(column, date)?)
    }

    Ok(events)
}

fn extract_event<'a>(element: ElementRef, date: NaiveDate) -> Result<Event<'a>> {
    Ok(Event {
        date: date,
        start: NaiveTime::from_hms_opt(10, 10, 10).unwrap(),
        end: NaiveTime::from_hms_opt(11, 11, 11).unwrap(),
        title: "woo",
        location: "woo",
    })
}
