package rapla

import (
	ics "github.com/arran4/golang-ical"
)

func Extract(url string) *ics.Calendar {
	cal := ics.NewCalendar()
	return cal
}
