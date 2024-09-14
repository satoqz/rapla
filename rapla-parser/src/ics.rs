use ics::{
    properties::{DtEnd, DtStart, Location, Organizer, RRule, Summary, TzName},
    parameters::TzIDParam,
    Daylight, Standard, TimeZone,
};

use crate::{Calendar, Event};

impl Calendar {
    #[must_use]
    pub fn to_ics(&self) -> ics::ICalendar<'_> {
        let mut cet_standard = Standard::new("19701025T030000", "+0200", "+0100");
        cet_standard.push(TzName::new("CET"));
        cet_standard.push(RRule::new("FREQ=YEARLY;BYMONTH=10;BYDAY=-1SU"));

        let mut cest_daylight = Daylight::new("19700329T020000", "+0100", "+0200");
        cest_daylight.push(TzName::new("CEST"));
        cest_daylight.push(RRule::new("FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU"));

        let mut timezone = TimeZone::daylight("Europe/Berlin", cest_daylight);
        timezone.add_standard(cet_standard);

        let mut icalendar = ics::ICalendar::new("2.0", &self.name);
        icalendar.add_timezone(timezone);

        for event in &self.events {
            icalendar.add_event(event.to_ics());
        }

        icalendar
    }
}

impl Event {
    #[must_use]
    pub fn to_ics(&self) -> ics::Event<'_> {
        let start = format!(
            "{}T{}00",
            self.date.format("%Y%m%d"),
            self.start.format("%H%M")
        );

        let end = format!(
            "{}T{}00",
            self.date.format("%Y%m%d"),
            self.end.format("%H%M")
        );

        let id = format!("{}_{}", start, self.title.replace(' ', "-"));

        let mut ics_event = ics::Event::new(id, start.clone());

        let mut dtstart = DtStart::new(start);
        dtstart.add(TzIDParam::new("Europe/Berlin"));

        let mut dtend = DtEnd::new(end);
        dtend.add(TzIDParam::new("Europe/Berlin"));

        ics_event.push(dtstart);
        ics_event.push(dtend);
        ics_event.push(Summary::new(&self.title));

        if let Some(location) = &self.location {
            ics_event.push(Location::new(location));
        }

        if let Some(organizer) = &self.organizer {
            ics_event.push(Organizer::new(organizer));
        }

        ics_event
    }
}
