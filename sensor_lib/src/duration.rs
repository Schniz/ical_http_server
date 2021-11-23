use chrono::{NaiveDateTime, TimeZone};
use chrono_tz::Tz;
use ical::property::Property;

#[derive(Debug)]
pub struct Duration {
    pub start: chrono::DateTime<Tz>,
    end: chrono::DateTime<Tz>,
}

impl Duration {
    pub fn new(start: chrono::DateTime<Tz>, end: chrono::DateTime<Tz>) -> Self {
        Self { start, end }
    }

    pub fn contains(&self, time: &chrono::DateTime<Tz>) -> bool {
        (self.start..self.end).contains(time)
    }

    pub fn try_from_properties(properties: &[Property], tz: &Tz) -> Option<Self> {
        let mut start = None::<&str>;
        let mut end = None::<&str>;
        for prop in properties.iter() {
            if prop.name.eq_ignore_ascii_case("dtstart") {
                start = prop.value.as_ref().map(|x| &x[..]);
            } else if prop.name.eq_ignore_ascii_case("dtend") {
                end = prop.value.as_ref().map(|x| &x[..]);
            }
        }

        let start = parse_ical_time_to_tz(tz, start?)?;
        let end = parse_ical_time_to_tz(tz, end?)?;

        Some(Duration::new(start, end))
    }
}

fn parse_ical_time_to_tz(tz: &Tz, date_str: &str) -> Option<chrono::DateTime<Tz>> {
    let fmt = "%Y%m%dT%H%M%S";
    let date_str = date_str.trim_end_matches('Z');
    let date = NaiveDateTime::parse_from_str(date_str, fmt).ok()?;
    let date = chrono::Utc.from_utc_datetime(&date).with_timezone(tz);
    Some(date)
}
