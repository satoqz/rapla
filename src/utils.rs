use crate::scraper::RaplaEvent;
use chrono::{Date, Duration, Utc};
use ics::{
    properties::{DtEnd, DtStart, Location, Organizer, Summary, TzID},
    Event,
};
use std::mem;

pub struct WeekRange(Date<Utc>, Date<Utc>);

impl WeekRange {
    pub fn new(start: Date<Utc>, end: Date<Utc>) -> Self {
        Self(start, end)
    }
}

impl Iterator for WeekRange {
    type Item = Date<Utc>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0 <= self.1 {
            let next = self.0 + Duration::days(7);
            Some(mem::replace(&mut self.0, next))
        } else {
            None
        }
    }
}

pub fn rapla_event_to_ics<'a>(event: RaplaEvent) -> Event<'a> {
    let date = format!("20{}", {
        let mut date_vec = event.date.split('.').collect::<Vec<&str>>();
        date_vec.reverse();
        date_vec.join("")
    });

    let start = format!("{}T{}00", date, event.start.replace(':', ""));
    let end = format!("{}T{}00", date, event.end.replace(':', ""));

    let id = format!("{}_{}", start, event.title.replace(' ', "-"));

    let mut ics_event = Event::new(id, start.clone());
    ics_event.push(Summary::new(event.title));
    ics_event.push(DtStart::new(start));
    ics_event.push(DtEnd::new(end));
    ics_event.push(Organizer::new(event.lecturers));
    ics_event.push(Location::new(event.location));
    ics_event.push(TzID::new("Europe/Berlin"));

    ics_event
}
