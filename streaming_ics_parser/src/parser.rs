use std::{convert::TryFrom, str::FromStr};

use crate::line::Line;
use chrono::{DateTime, NaiveDateTime, TimeZone};
use chrono_tz::Tz;

#[derive(Debug, PartialEq, Eq)]
pub struct Event {
    pub name: String,
    pub timezone: Tz,
    pub start: DateTime<Tz>,
    pub end: DateTime<Tz>,
}

#[derive(Default, Debug)]
struct PartialEvent {
    pub name: Option<String>,
    pub timezone: Option<Tz>,
    pub start: Option<DateTime<Tz>>,
    pub end: Option<DateTime<Tz>>,
}

impl<'a> TryFrom<&'a PartialEvent> for Event {
    type Error = &'static str;

    fn try_from(event: &'a PartialEvent) -> Result<Self, Self::Error> {
        match event {
            PartialEvent {
                name: Some(name),
                timezone: Some(timezone),
                start: Some(start),
                end: Some(end),
            } => Ok(Event {
                name: name.clone(),
                timezone: timezone.clone(),
                start: start.clone(),
                end: end.clone(),
            }),
            _ => Err("Event is missing required fields"),
        }
    }
}

#[derive(Default, Debug)]
pub struct Parser {
    current_partial_event: Option<PartialEvent>,
    current_timezone: Option<Tz>,
}

impl Parser {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn process_line(&mut self, line: &Line) -> Option<Event> {
        match (&line.key[..], &line.value) {
            ("x-wr-timezone", Some(tz)) => {
                self.current_timezone = Tz::from_str(&tz).ok();
            }
            ("begin", Some(value)) if value.eq_ignore_ascii_case("vevent") => {
                self.current_partial_event = Some(PartialEvent {
                    timezone: self.current_timezone.clone(),
                    ..Default::default()
                });
            }
            ("end", Some(value)) if value.eq_ignore_ascii_case("vevent") => {
                if let Some(ref event) = self.current_partial_event {
                    return Event::try_from(event).ok();
                }
            }
            ("dtstart", Some(dtstart)) => {
                if let Some(event) = self.current_partial_event.as_mut() {
                    if let Some(tz) = event.timezone {
                        if let Some(time) = parse_ical_time_to_tz(&tz, &dtstart[..]) {
                            event.start = Some(time);
                        }
                    }
                }
            }
            ("dtend", Some(dtend)) => {
                if let Some(event) = self.current_partial_event.as_mut() {
                    if let Some(tz) = event.timezone {
                        if let Some(time) = parse_ical_time_to_tz(&tz, &dtend[..]) {
                            event.end = Some(time);
                        }
                    }
                }
            }
            ("summary", Some(summary)) => {
                if let Some(current_event) = self.current_partial_event.as_mut() {
                    current_event.name = Some(summary.to_string());
                }
            }
            _ => {}
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    #[test]
    fn test_to_calendar_iterator() {
        use super::*;
        use std::io::{BufRead, BufReader};

        let bytes = include_bytes!("../fixtures/multiple.ics");
        let mut parser = Parser::new();
        let calendars: Vec<_> = BufReader::new(&bytes[..])
            .lines()
            .flatten()
            .flat_map(|x| Line::from_str(&x).ok())
            .filter_map(|x| parser.process_line(&x))
            .collect();

        assert_eq!(calendars.len(), 2);

        let tuples: Vec<_> = calendars
            .iter()
            .map(|event| {
                (
                    event.name.to_string(),
                    event.timezone.to_string(),
                    event.start.to_string(),
                    event.end.to_string(),
                )
            })
            .collect();

        assert_eq!(
            tuples,
            vec![
                (
                    "kaki".to_string(),
                    "Asia/Jerusalem".to_string(),
                    "2021-11-22 11:15:00 IST".to_string(),
                    "2021-11-22 12:15:00 IST".to_string(),
                ),
                (
                    "kaki2".to_string(),
                    "Europe/Berlin".to_string(),
                    "2021-11-22 10:15:00 CET".to_string(),
                    "2021-11-22 11:15:00 CET".to_string(),
                )
            ]
        );
    }
}

fn parse_ical_time_to_tz(tz: &Tz, date_str: &str) -> Option<chrono::DateTime<Tz>> {
    let fmt = "%Y%m%dT%H%M%S";
    let date_str = date_str.trim_end_matches('Z');
    let date = NaiveDateTime::parse_from_str(date_str, fmt).ok()?;
    let date = chrono::Utc.from_utc_datetime(&date).with_timezone(tz);
    Some(date)
}
