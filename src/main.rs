pub struct Event {}

impl Event {
    pub fn new() -> Self {
        Event {}
    }
}

pub struct Page {}

impl Page {
    pub fn new(html: String) -> Self {
        Page {}
    }

    pub fn get_events() -> Vec<Event> {
        vec![]
    }
}

#[async_std::main]
async fn main() {
    let url = "";
    let html = surf::get(url);
}
