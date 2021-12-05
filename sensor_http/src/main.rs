use sensor_lib::is_calendar_busy;
use std::collections::HashMap;
use tracing::error;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{prelude::*, EnvFilter, Registry};
use warp::Filter;

#[derive(serde::Deserialize, Debug)]
struct Input {
    urls: HashMap<String, url::Url>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_tracing();

    let server = warp::path!("by_url")
        .and(warp::filters::method::post())
        .and(warp::body::json())
        .and_then(build_output)
        .with(warp::trace::request());

    let root = warp::path::end().map(|| "Hello!");

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()?;
    warp::serve(server.or(root)).run(([0, 0, 0, 0], port)).await;

    Ok(())
}

async fn build_output(input: Input) -> Result<impl warp::Reply, warp::Rejection> {
    use futures::stream::StreamExt;

    let busy_urls = futures::stream::iter(input.urls.into_iter())
        .map(|(url, url_obj)| {
            let url = url.clone();
            async move {
                let is_busy = is_calendar_busy(url_obj).await;
                if let Err(err) = &is_busy {
                    error!("Can't get data: {}", err);
                }
                is_busy.ok().map(move |is_busy| (url, is_busy))
            }
        })
        .buffer_unordered(10)
        .fold(HashMap::new(), |mut v, result| async {
            if let Some((url, is_busy)) = result {
                v.insert(url, is_busy);
            }

            v
        })
        .await;
    Ok(warp::reply::json(&busy_urls))
}

fn setup_tracing() {
    let subscriber = Registry::default()
        .with(EnvFilter::from_default_env())
        .with(JsonStorageLayer)
        .with(BunyanFormattingLayer::new(
            "ics_sensor_http".to_string(),
            std::io::stdout,
        ));
    set_global_default(subscriber).unwrap();
}
