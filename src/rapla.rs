use std::borrow::Cow;

use chrono::{Duration, NaiveDate, NaiveTime};
use ics::{
    properties::{DtEnd, DtStart, Location, RRule, Summary, TzName},
    Daylight, ICalendar, Standard, TimeZone,
};
use once_cell::sync::Lazy;
use scraper::{Html, Selector};

macro_rules! selector {
    ($name:ident, $query:expr) => {
        static $name: Lazy<Selector> = Lazy::new(|| Selector::parse($query).unwrap());
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub date: NaiveDate,
    pub start: NaiveTime,
    pub end: NaiveTime,
    pub title: String,
    pub location: Option<String>,
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

        if let Some(location) = event.location {
            ics_event.push(Location::new(location));
        }

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

pub fn extract_events<S: AsRef<str>>(html: S) -> Option<Vec<Event>> {
    selector!(START_YEAR, "select[name=year] > option[selected]");
    selector!(WEEK_NUMBER, "th.week_number");
    selector!(WEEKS, "div.calendar > table.week_table > tbody");
    selector!(START_DATE, "tr > td.week_header > nobr");
    selector!(ROWS, "tr");
    selector!(COLUMNS, "td");
    selector!(RESOURCE, "span.resource");
    selector!(ANCHOR, "a");

    let html = Html::parse_document(html.as_ref());
    let mut start_year = html
        .select(&START_YEAR)
        .next()?
        .inner_html()
        .parse::<i32>()
        .ok()?;

    let mut events = Vec::new();
    for week in html.select(&WEEKS) {
        let week_number = week
            .select(&WEEK_NUMBER)
            .next()?
            .inner_html()
            .split(' ')
            .nth(1)?
            .parse::<usize>()
            .ok()?;

        if week_number == 1 {
            start_year += 1;
        }

        let start_date_raw = week.select(&START_DATE).next()?.inner_html();

        let mut day_month = start_date_raw
            .split(' ')
            .nth(1)?
            .trim_end_matches('.')
            .split('.');

        let start_day = day_month.next()?.parse::<u32>().ok()?;
        let start_month = day_month.next()?.parse::<u32>().ok()?;

        for row in week.select(&ROWS).skip(1) {
            let monday = NaiveDate::from_ymd_opt(start_year, start_month, start_day)?;
            let mut day_index = 0;

            for column in row.select(&COLUMNS) {
                let class = column.value().classes().next()?;

                if class.starts_with("week_separatorcell") {
                    day_index += 1;
                }

                if class != "week_block" {
                    continue;
                }

                let date = monday + Duration::days(day_index);

                let details = column.select(&ANCHOR).next()?.inner_html();
                let mut details_split = details.split("<br>");

                let times_raw = details_split.next()?;
                let mut times_raw_split = times_raw.split("&nbsp;-");

                let start = NaiveTime::parse_from_str(times_raw_split.next()?, "%H:%M").ok()?;
                let end = NaiveTime::parse_from_str(times_raw_split.next()?, "%H:%M").ok()?;

                let title = details_split.next()?.replace("&amp;", "&");

                let location = column
                    .select(&RESOURCE)
                    .nth(1)
                    .map(|location| location.inner_html());

                events.push(Event {
                    date,
                    start,
                    end,
                    title,
                    location,
                })
            }
        }
    }

    Some(events)
}
