use chrono::Local;
use futures::TryStreamExt;
use reqwest::IntoUrl;
use std::str::FromStr;
use streaming_ics_parser::{Line, Parser};
use tokio::io::AsyncBufReadExt;
use tokio_util::io::StreamReader;
use tracing::info_span;

pub async fn is_calendar_busy(into_url: impl IntoUrl) -> Result<bool, Box<dyn std::error::Error>> {
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
                let current_time = Local::now().with_timezone(&event.timezone);
                if ((event.start)..(event.end)).contains(&current_time) {
                    return Ok(true);
                }
            }
        }
    }

    Ok(false)
}
