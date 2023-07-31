## `rapla-sync`

[![built with nix](https://builtwithnix.org/badge.svg)](https://builtwithnix.org)

### What is this?

This is a web service that scrapes the rapla site of [DHBW Stuttgart](https://dhbw-stuttgart.de) hosted at [rapla.dhbw-stuttgart.de](https://rapla.dhbw-stuttgart.de).
It then returns the scraped events in the iCalendar format, a standard supported by almost all calendar providers on the internet.

### Usage

The link that you use to visit your rapla schedule site contains a URL parameter called `key`. This key identifies your calendar. To sync your schedule to another provider, copy the value of the key parameter and append it to a URL that points to a rapla-sync instance.

```sh
# from this:
https://rapla.dhbw-stuttgart.de/rapla?key={YOUR_RAPLA_KEY}
# to this:
https://{RAPLA_SYNC_INSTANCE}/{YOUR_RAPLA_KEY}
```

### Official Instance

I host a public instance on [fly.io](https://fly.io). It is available at https://rapla.fly.dev.

To use it, simply add 

```
https://rapla.fly.dev/<YOUR_RAPLA_KEY>
```

to your calendar subscriptions.

### Docker Image

A docker image built by GitHub Actions is available as `ghcr.io/satoqz/rapla-sync:latest`
