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

    pub async fn scrape_page(
        &self,
        year: i32,
        month: u32,
        day: u32,
    ) -> Result<Vec<RaplaEvent>, String> {
        let url = self.format_url(year, month, day);
        let html = get(url)
            .recv_string()
            .await
            .map_err(|err| err.to_string())?;
        let doc = Html::parse_document(html.as_str());
        let mut events = Vec::new();

        if doc.select(&self.selectors.calendar).next().is_none() {
            return Err("No calendar page received".into());
        }

        for week_block in doc.select(&self.selectors.week_block) {
            let infotable = week_block
                .select(&self.selectors.infotable)
                .next()
                .ok_or("Unexpected HTML: information_table selector")?;

            let title = infotable
                .select(&self.selectors.td)
                .nth(1)
                .ok_or("Unexpected HTML: event name td selector")?
                .inner_html();

            // exams are only given in form of an all-day event which is quite useless
            if title == "Klausur" {
                continue;
            }

            let lecturers = week_block
                .select(&self.selectors.person)
                .map(|lecturer| lecturer.inner_html().trim_end_matches(',').into())
                .collect::<Vec<String>>()
                .join(" & ");

            let time_info_string = week_block
                .select(&self.selectors.div)
                .nth(1)
                .ok_or("Unexpected HTML: time_info div selector")?
                .inner_html();

            let time_info_vec = time_info_string.split(' ').collect::<Vec<&str>>();

            let date = time_info_vec
                .get(1)
                .ok_or("Unexpected HTML: time_info")?
                .to_string();

            let mut times = time_info_vec
                .get(2)
                .ok_or("Unexpected HTML: time info")?
                .split('-');

            let start = times.next().ok_or("Unexpected HTML: time info")?.into();
            let end = times.next().ok_or("Unexpected HTML: time info")?.into();

            let is_online = week_block.html().contains("background-color:#9999ff");

            let location = if is_online {
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

        Ok(events)
    }
}
