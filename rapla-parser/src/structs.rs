use chrono::{NaiveDate, NaiveTime};

#[derive(Debug, Clone)]
pub struct Calendar {
    pub name: String,
    pub events: Vec<Event>,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub date: NaiveDate,
    pub start: NaiveTime,
    pub end: NaiveTime,
    pub title: String,
    pub location: Option<String>,
    pub organizer: Option<String>,
    pub description: Option<String>,
}
