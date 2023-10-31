use log::trace;
use reqwest::Url;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
struct HearbeatBody<'a> {
    worker_name: &'a str,
    worker_version: &'a str,
}

pub async fn send_heartbeat(
    url: &Url,
    worker_name: &str,
    worker_version: &str,
) -> anyhow::Result<()> {
    trace!("Sending monitoring heartbeat...");
    let res = reqwest::Client::new()
        .post(url.as_ref())
        .json(&HearbeatBody {
            worker_name,
            worker_version,
        })
        .send()
        .await?
        .error_for_status()?;
    trace!("Monitoring heartbeat got a {} response", res.status());
    Ok(())
}
