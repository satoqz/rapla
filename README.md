# rapla-ical-proxy

- [Introduction](#introduction)
- [Usage as Calendar Synchronizer](#usage-as-calendar-synchronizer)
- [Self-Hosting](#self-hosting)
- [Configuration](#configuration)

## Introduction

`rapla-ical-proxy` is a web service that converts the HTML calendar shown by DHBW's class schedule page ([rapla.dhbw.de](https://rapla.dhbw.de)) into the universally accepted [iCalendar](https://icalendar.org/) format on the fly.
Rapla does not natively offer a (reliable) method to integrate with internet calendar providers (think Outlook, Google Calendar, etc.), thus a 3rd party service like this is needed to bridge the gap.

I host a public instance at `rapla.satoqz.net`, powered by [fly.io](https://fly.io).

## Usage as Calendar Synchronizer

To get started synchronizing your schedule to a calendar provider of your choice, follow below steps:

1. Get your Rapla URL ready.
   This should be a very long URL with `rapla.dhbw.de` as its host, provided to you by DHBW.

2. Replace the `dhbw.de` part of the URL with `satoqz.net`, such that it becomes `rapla.satoqz.net`.

3. Paste the resulting URL into the "New calendar subscription" field of your calendar provider. The name of this feature varies based on your provider.

## Self-Hosting

A [Dockerfile](./Dockerfile) is included for easy deployment.

## Configuration

You can set the `RAPLA_ICAL_PROXY_ADDR` environment variable to configure the socket address that the service binds to.
The default is `127.0.0.1:8080`.
