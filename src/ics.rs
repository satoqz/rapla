use crate::scraper::RaplaScraper;
use crate::utils::{rapla_event_to_ics, WeekRange};

use async_std::sync::Mutex;
use chrono::{Date, Datelike, Duration, Utc};
use futures::future;
use ics::{Event, ICalendar};
use std::sync::Arc;

async fn process_week<'a>(
    rapla: &RaplaScraper,
    ics: &Arc<Mutex<ICalendar<'a>>>,
    week: Date<Utc>,
) -> Result<(), String> {
    let events = rapla
        .scrape_page(week.year(), week.month(), week.day())
        .await?
        .iter()
        .map(rapla_event_to_ics)
        .collect::<Vec<Event>>();

    let mut l = ics.lock().await;

    for event in events {
        l.add_event(event);
    }

    Ok(())
}

pub async fn get_ics(key: &str) -> Result<String, String> {
    let rapla = RaplaScraper::new(format!("https://rapla.dhbw-stuttgart.de/rapla?key={key}"));

    let mut ics = ICalendar::new("2.0", key);
    ics.add_timezone(ics::TimeZone::standard(
        "Europe/Berlin",
        ics::Standard::new("18930401T000632", "+0053", "+0100"),
    ));

    let ics = Arc::new(Mutex::new(ics));

    let now = Utc::now().date();
    let start = now - Duration::weeks(25);
    let end = now + Duration::weeks(25);

    match future::join_all(WeekRange::new(start, end).map(|week| process_week(&rapla, &ics, week)))
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(_) => Ok(ics.lock().await.to_string()),
        Err(err) => Err(err),
    }
}
