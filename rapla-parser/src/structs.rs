use chrono::{NaiveDate, NaiveTime};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Calendar {
    pub name: String,
    pub events: Vec<Event>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Event {
    pub date: NaiveDate,
    pub start: NaiveTime,
    pub end: NaiveTime,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organizer: Option<String>,
    pub description: Option<String>,
}
