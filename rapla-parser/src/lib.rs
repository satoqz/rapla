mod parser;
mod structs;

#[cfg(feature = "ics")]
mod ics;

pub use parser::parse_calendar;
pub use structs::{Calendar, Event};
