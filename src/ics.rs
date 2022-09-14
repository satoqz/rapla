use crate::scraper::RaplaScraper;
use crate::utils::{rapla_event_to_ics, WeekRange};

use async_std::sync::Mutex;
use chrono::{Date, Datelike, Duration, Utc};
use futures::future;
use ics::{
    properties::{RRule, TzName},
    Event, ICalendar,
};
use std::sync::Arc;

async fn process_week<'a>(
    rapla: &RaplaScraper,
    ics: &Arc<Mutex<ICalendar<'a>>>,
    week: Date<Utc>,
) -> Option<()> {
    let events = rapla
        .scrape_page(week.year(), week.month(), week.day())
        .await?
        .into_iter()
        .map(rapla_event_to_ics)
        .collect::<Vec<Event>>();

    let mut l = ics.lock().await;

    for event in events {
        l.add_event(event);
    }

    Some(())
}

pub async fn get_ics(key: &str) -> Option<String> {
    let rapla = RaplaScraper::new(format!("https://rapla.dhbw-stuttgart.de/rapla?key={key}"));

    let mut cest = ics::Daylight::new("19700329T020000", "+0100", "+0200");
    cest.push(TzName::new("CEST"));
    cest.push(RRule::new("FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU"));
    let mut cet = ics::Standard::new("19701025T030000", "+0200", "+0100");
    cet.push(TzName::new("CET"));
    cet.push(RRule::new("FREQ=YEARLY;BYMONTH=10;BYDAY=-1SU"));

    let mut timezone = ics::TimeZone::daylight("Europe/Berlin", cest);
    timezone.add_standard(cet);

    let mut ics = ICalendar::new("2.0", key);
    ics.add_timezone(timezone);

    let ics = Arc::new(Mutex::new(ics));

    let now = Utc::now().date();
    let start = now - Duration::weeks(15);
    let end = now + Duration::weeks(15);

    if future::join_all(WeekRange::new(start, end).map(|week| process_week(&rapla, &ics, week)))
        .await
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .is_some()
    {
        Some(ics.lock().await.to_string())
    } else {
        None
    }
}
