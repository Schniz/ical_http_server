mod duration;

use crate::duration::Duration;
use chrono::{DateTime, Local, TimeZone};
use chrono_tz::Tz;
use ical::IcalParser;
use reqwest::IntoUrl;
use std::io::BufReader;
use tracing::{info, info_span};

pub async fn is_calendar_busy(into_url: impl IntoUrl) -> Result<bool, Box<dyn std::error::Error>> {
    let url = into_url.into_url()?;
    let url_s = url.to_string();
    let span = info_span!("is_calendar_busy", url = &url_s[..]);
    let _ = span.enter();

    let content = {
        let span = info_span!("drain_response_body_bytes");
        let _ = span.enter();
        let response = reqwest::get(url).await?;
        response.bytes().await?
    };

    let bufreader = BufReader::new(content.as_ref());
    let parser = IcalParser::new(bufreader);
    for mut line in parser.flatten() {
        let span = info_span!("line");
        let _ = span.enter();

        info!(
            event_count = line.events.len(),
            "Found {} events",
            line.events.len()
        );

        if let Some(timezone) = get_timezone(&mut line.properties) {
            let current_time = current_time_at(&timezone)?;

            for event in line.events {
                if let Some(duration) = Duration::try_from_properties(&event.properties, &timezone)
                {
                    // Checks if current_time_in_tz is within the duration of the event
                    if duration.contains(&current_time) {
                        dbg!((duration, current_time));
                        return Ok(true);
                    }
                }
            }
        }
    }

    Ok(false)
}

fn current_time_at(tz: &Tz) -> Result<DateTime<Tz>, Box<dyn std::error::Error>> {
    let current_time = Local::now();
    let current_time_in_tz = tz
        .from_local_datetime(&current_time.naive_local())
        .latest()
        .ok_or_else(|| "welp")?;
    Ok(current_time_in_tz)
}

fn get_timezone(calendar: &mut Vec<ical::property::Property>) -> Option<Tz> {
    let mut timezone = None::<String>;
    for prop in calendar.drain(..) {
        if prop.name.eq_ignore_ascii_case("x-wr-timezone") {
            timezone = prop.value;
        }
    }

    let timezone = timezone?.parse::<Tz>().ok()?;
    Some(timezone)
}
