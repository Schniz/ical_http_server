mod duration;
mod event;

use crate::event::Event;
use chrono::{DateTime, Local};
use chrono_tz::Tz;
use futures::TryStreamExt;
use ical::IcalParser;
use reqwest::IntoUrl;
use std::str::FromStr;
use streaming_ics_parser::{Line, Parser};
use tokio::io::AsyncBufReadExt;
use tokio_util::io::StreamReader;
use tracing::{error, info, info_span};

pub async fn sync_is_calendar_busy(
    into_url: impl IntoUrl,
) -> Result<bool, Box<dyn std::error::Error>> {
    use std::io::BufReader;

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
            let current_time_s = format!("{:?}", current_time);
            info!(current_time = &current_time_s[..], "Found the given event");

            for event in line.events {
                if let Some(event) = Event::try_from_properties(&event.properties, &timezone) {
                    // if event.duration.start.year() == 2021 {
                    //     let event_s = format!("{:?}", event);
                    //     let current_time_s = format!("{:?}", current_time);
                    //     info!(event = &event_s[..], current_time = &current_time_s[..], "Found the given event");
                    // }
                    // Checks if current_time_in_tz is within the duration of the event
                    if event.duration.contains(&current_time) {
                        return Ok(true);
                    }
                }
            }
        }
    }

    Ok(false)
}

pub async fn async_is_calendar_busy(
    into_url: impl IntoUrl,
) -> Result<bool, Box<dyn std::error::Error>> {
    let url = into_url.into_url()?;
    let url_s = url.to_string();
    let span = info_span!("is_calendar_busy", url = &url_s[..]);
    let _ = span.enter();

    let stream = reqwest::get(url)
        .await?
        .bytes_stream()
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err));
    let stream = StreamReader::new(stream);
    let mut lines = stream.lines();
    let mut parser = Parser::new();

    while let Ok(Some(line)) = lines.next_line().await {
        if let Ok(line) = Line::from_str(&line[..]) {
            if let Some(event) = parser.process_line(&line) {
                let current_time = current_time_at(&event.timezone)?;
                if ((event.start)..(event.end)).contains(&current_time) {
                    return Ok(true);
                }
            }
        }
    }

    Ok(false)
}

fn current_time_at(tz: &Tz) -> Result<DateTime<Tz>, Box<dyn std::error::Error>> {
    Ok(Local::now().with_timezone(tz))
}

fn get_timezone(calendar: &Vec<ical::property::Property>) -> Option<Tz> {
    let mut timezone = None::<&str>;
    for prop in calendar.iter() {
        if prop.name.eq_ignore_ascii_case("x-wr-timezone") {
            timezone = prop.value.as_ref().map(|x| &x[..]);
        }
    }

    if timezone.is_none() {
        let properties = format!("{:?}", calendar);
        error!(
            properties = &properties[..],
            "Can't get timezone from calendar properties"
        );
    }

    let timezone = timezone?.parse::<Tz>();
    if let Err(err) = &timezone {
        error!("Can't parse timezone: {}", err)
    }

    match timezone {
        Ok(timezone) => {
            info!("Found timezone {}", timezone);
            Some(timezone)
        }
        Err(err) => {
            error!("Can't parse timezone: {}", err);
            None
        }
    }
}
