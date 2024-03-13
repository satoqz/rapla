# 📆 rapla-proxy

- [Introduction](#introduction)
- [Usage as Calendar Synchronizer](#usage-as-calendar-synchronizer)
- [Usage as JSON API](#usage-as-json-api)
- [Self-Hosting](#self-hosting)

## Introduction

`rapla-proxy` is a web service that converts the HTML calendar shown by DHBW's class schedule page ([rapla.dhbw.de](https://rapla.dhbw.de)) into the universally accepted [iCalendar](https://icalendar.org/) format on the fly.
Rapla does not natively offer a (reliable) method to integrate with internet calendar providers (think Outlook, Google Calendar, etc.), thus a 3rd party service like this is needed to bridge the gap.

I host a public instance of `rapla-proxy` at `rapla.satoqz.net`, powered by [fly.io](https://fly.io).

## Usage as Calendar Synchronizer

To get started synchronizing your schedule to a calendar provider of your choice, follow below steps:

1. Get your Rapla URL ready.
   This should be a very long URL with `rapla.dhbw.de` as its host, provided to you by DHBW.

2. Replace the `dhbw.de` part of the URL with `satoqz.net`, such that it becomes `rapla.satoqz.net`.

3. Paste the resulting URL into the "New calendar subscription" field of your calendar provider. The name of this feature varies based on your provider.

## Usage as JSON API

If you would like to receive the list of events as JSON rather than in the iCalendar format, you can add the `&json=true` query parameter to the Rapla URL. The resulting JSON response looks as follows:

```jsonc
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
