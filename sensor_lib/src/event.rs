use crate::duration::Duration;
use chrono_tz::Tz;
use ical::property::Property;

#[derive(Debug)]
pub struct Event {
    pub duration: Duration,
    pub name: Option<String>,
}

impl Event {
    pub fn try_from_properties(properties: &[Property], tz: &Tz) -> Option<Self> {
        let name = properties
            .iter()
            .find(|x| x.name.eq_ignore_ascii_case("summary"))
            .and_then(|x| x.value.clone());

        Some(Self {
            name,
            duration: Duration::try_from_properties(&properties, &tz)?,
        })
    }
}
