# `📆 rapla-proxy`

- [Introduction](#introduction)
- [Usage as Calendar Synchronizer](#usage-as-calendar-synchronizer)
- [Usage as JSON API](#usage-as-json-api)
- [Self-Hosting](#self-hosting)
- [Usage as Rust Library](#usage-as-rust-library)

## Introduction

`rapla-proxy` is a web service that converts the HTML calendar shown by DHBW's class schedule page ([rapla.dhbw.de](https://rapla.dhbw.de)) into the universally accepted [iCalendar](https://icalendar.org/) format on the fly.
Rapla does not natively offer a (reliable) method to integrate with internet calendar providers (think Outlook, Google Calendar, etc.), thus a 3rd party service like this is needed to bridge the gap.

I host a public instance of `rapla-proxy` at `rapla.satoqz.net`, powered by [fly.io](https://fly.io) servers in the Netherlands.

## Usage as Calendar Synchronizer

To get started synchronizing your schedule to a calendar provider of your choice, follow below steps:

1. Get your Rapla URL ready.
   This should be a very long URL with `rapla.dhbw.de` as its host, provided to you by DHBW.

2. Replace the `dhbw.de` part of the URL with `satoqz.net`.

3. Paste the resulting URL into the "New calendar subscription" field of your calendar provider. The name of this feature varies based on your provider.

## Usage as JSON API

If you would like to receive the list of events as JSON rather than in the iCalendar format, you can add the `&json=true` query parameter to the Rapla URL. The resulting JSON response looks as follows:

```json
{
  "events": [
    {
      "date": "2023-10-12",
      "start": "08:30",
      "end": "11:15",
      "title": "Grundlagen Data Science",
      "location": "C3.01 Vorlesung"
    },
    {
      "date": "",
      "start": "",
      "end": "",
      "title": "Wahlfach",
      "location": null
    }
    // ...
  ]
}
```

## Self-Hosting

The proxy service is configured using the `IP` and `PORT` environment variables,
where `127.0.0.0` and `8080` are the defaults respectively.

A [Dockerfile](./Dockerfile) is included to simplify container deployments. The container binds to the `0.0.0.0` address by default.

## Usage as Rust Library

The core scraping logic is exposed as a Rust library.
To add it to your Rust project, include the following in your `Cargo.toml`:

```toml
[dependencies]
rapla = { git = "https://github.com/satoqz/rapla-proxy.git", default-features = false }
```

To enable conversion of scraped calendars to the iCalendar format, include the `ics` feature:

```toml
[dependencies]
rapla = { git = "https://github.com/satoqz/rapla-proxy.git", default-features = false, features = ["ics"] }
```

Below is a minimal example using `tokio` and `reqwest` showing how to download HTML from Rapla and then parse it into a `rapla::Calendar`:

```rs
const RAPLA_URL: &str = "...";

#[tokio::main]
async fn main() {
   let response = reqwest::get(RAPLA_URL).await.unwrap();
   let html = response.text().await.unwrap();

   let calendar = rapla::Calendar::from_html(html).unwrap();

   for event in &calendar.events {
      // ...
   }

   println!("{}", calendar.to_ics());
}
```

> **Note**
> The `rapla-proxy` service transforms the query parameters the given Rapla URL such that it always returns events ranging from ± 1 year from the current time. The core `rapla` library can parse only the events that are actually included in the HTML that you pass and will not issue any further requests.
