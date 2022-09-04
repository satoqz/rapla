# `rapla-to-ics`

Server to transform DHBW Rapla Timetable links into the iCalendar format.
Import any Rapla timetable into your calendar provider of choice (Outlook, Google, Proton...).

## Usage

Append your Rapla key (included in the parameters of the Rapla URL given to you) to the request URL.

Example:

```
https://rapla.dhbw-stuttgart.de/rapla?key={your_rapla_key}
-> {application_url}/{your_rapla_key}(.ics?)
```

Optionally, you can append the `.ics` file extension after the key.

## Official instance

The official instance is available at https://blade.trench.world.
This is the easiest way to use the project.

Simply use

```
https://blade.trench.world/{your_rapla_key}
```

to your calendar provider

## Docker image

A Docker image automatically built by CI is available here:

```
docker pull ghrc.io/satoqz/rapla-to-ics:latest
```

## Credits

[JulianGroshaupt/dhbw_rapla-to-ics](https://github.com/JulianGroshaupt/dhbw_rapla-to-ics)
