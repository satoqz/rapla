use scraper::{Html, Selector};
use surf::get;

struct Selectors {
    calendar: Selector,
    week_block: Selector,
    infotable: Selector,
    person: Selector,
    resource: Selector,
    div: Selector,
    td: Selector,
    strong: Selector,
}

impl Selectors {
    pub fn parse() -> Self {
        Self {
            calendar: Selector::parse("#calendar").unwrap(),
            week_block: Selector::parse("td.week_block").unwrap(),
            infotable: Selector::parse("table.infotable").unwrap(),
            person: Selector::parse("span.person").unwrap(),
            resource: Selector::parse("span.resource").unwrap(),
            div: Selector::parse("div").unwrap(),
            td: Selector::parse("td").unwrap(),
            strong: Selector::parse("strong").unwrap(),
        }
    }
}

pub struct RaplaEvent {
    pub title: String,
    pub lecturers: String,
    pub date: String,
    pub start: String,
    pub end: String,
    pub location: String,
}

pub struct RaplaScraper {
    url: String,
    selectors: Selectors,
}

impl RaplaScraper {
    pub fn new(url: String) -> Self {
        Self {
            url,
            selectors: Selectors::parse(),
        }
    }

    fn format_url(&self, year: i32, month: u32, day: u32) -> String {
        format!("{}&day={}&month={}&year={}", self.url, day, month, year)
    }

    pub async fn scrape_page(&self, year: i32, month: u32, day: u32) -> Option<Vec<RaplaEvent>> {
        let url = self.format_url(year, month, day);
        let html = get(url).recv_string().await.ok()?;
        let doc = Html::parse_document(html.as_str());
        let mut events = Vec::new();

        doc.select(&self.selectors.calendar).next()?;

        for week_block in doc.select(&self.selectors.week_block) {
            let tp = week_block
                .select(&self.selectors.strong)
                .next()?
                .inner_html();

            if !["Vorlesung", "Online-Format", "Klausur"]
                .map(|val| tp.starts_with(val))
                .contains(&true)
            {
                continue;
            }

            let infotable = week_block.select(&self.selectors.infotable).next()?;
            let title = infotable
                .select(&self.selectors.td)
                .nth(1)?
                .inner_html()
                .replace("&amp;", "&");

            /* a title starting with "Belegung" or "Raum belegt" indicates an event irrelevant to the course
            a title that is only "Klausur" instead of e.g. "Klausur Mathematik" indicates that
            the event is only a blocker but does not contain specific times and names */
            if ["Belegung", "Raum belegt"]
                .map(|val| title.starts_with(val))
                .contains(&true)
                || title == "Klausur"
            {
                continue;
            }

            let lecturers = week_block
                .select(&self.selectors.person)
                .map(|lecturer| lecturer.inner_html().trim_end_matches(',').into())
                .collect::<Vec<String>>()
                .join(" & ");

            let time_info_string = week_block.select(&self.selectors.div).nth(1)?.inner_html();
            let time_info_vec = time_info_string.split(' ').collect::<Vec<&str>>();
            let date = time_info_vec.get(1)?.to_string();

            let mut times = time_info_vec.get(2)?.split('-');

            let start = times.next()?.into();
            let end = times.next()?.into();

            let location = if tp.starts_with("Online-Format") {
                "Online".into()
            } else {
                week_block
                    .select(&self.selectors.resource)
                    .map(|resource| resource.inner_html())
                    .collect::<Vec<String>>()
                    .join(", ")
            };

            events.push(RaplaEvent {
                title,
                lecturers,
                date,
                start,
                end,
                location,
            });
        }

        Some(events)
    }
}
