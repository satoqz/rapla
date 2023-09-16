## Introduction

`rapla-proxy` is a web service that converts the HTML calendar shown by DHBW's class schedule page ([rapla.dhbw.de](https://rapla.dhbw.de)) into the universally accepted [iCalendar](https://icalendar.org/) format on the fly.
Rapla does not natively offer a method to integrate with internet calendar providers (think Outlook, Google Calendar, etc.), thus a 3rd party service like this is needed to bridge the gap.

I host a public instance of `rapla-proxy` at `rapla.satoqz.net`, powered by [fly.io](https://fly.io) servers in the Netherlands.
Alternatively, you can easily self-host the service, e.g. using the provided `Dockerfile`.

## Getting Started

To get started synchronizing your schedule to a calendar provider of your choice, follow below steps:

1. Get your Rapla URL ready.
   This should be a very long URL with `rapla.dhbw.de` as its host, provided to you by DHBW.
   The proxy will mirror exactly the events that are shown under this URL.

2. Replace the `dhbw.de` part of the URL with `satoqz.net`.

3. Remove all query parameters from the URL that indicate a specific date, i.e. `&day=...`, `&month=...` and `&year=...`.
   While this is optional, this way the returned Rapla page will always start with the current week, continuously progressing through time.

4. Ensure that the URL has the `&pages=...` query parameter set to a large value, e.g. `&pages=20`.
   This is also optional and ensures that the returned iCalendar file includes several weeks of your schedule.

5. Paste the resulting URL into the "New calendar subscription" field of your calendar provider. The name of this feature varies based on your provider.

## Service Configuration

The IP address and port used by the service are configured via the `IP` and `PORT` environment variables. The defaults are `127.0.0.1` and `8080` respectively.
