use anyhow::bail;
use chrono::Utc;
use log::{trace, warn};
use reqwest::Url;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::spawn;
use tokio::sync::mpsc::Receiver;

#[derive(Debug, Clone, Serialize)]
struct HearbeatBody<'a> {
    worker_name: &'a str,
    worker_version: &'a str,
}

pub async fn heartbeat_sender(
    heartbeat_min_period: Duration,
    url: &Url,
    rx: &mut Receiver<u16>,
    worker_name: &str,
    worker_version: &str,
) -> anyhow::Result<()> {
    let mut last_heartbeat = Utc::now() - heartbeat_min_period;
    let url = Arc::new(url.to_owned());
    let worker_name = Arc::new(worker_name.to_owned());
    let worker_version = Arc::new(worker_version.to_owned());

    while let Some(unit_id) = rx.recv().await {
        trace!("Ping received from unit {unit_id}");

        if last_heartbeat + heartbeat_min_period <= Utc::now() {
            trace!("Sending heartbeat...");
            let url = url.clone();
            let worker_name = worker_name.clone();
            let worker_version = worker_version.clone();
            spawn(async move {
                if let Err(e) = send_heartbeat(&url, &worker_name, &worker_version).await {
                    warn!("Monitoring heartbeat failed: {e}");
                }
            });
            last_heartbeat = Utc::now();
        }
    }

    bail!("Heartbeat sender task ended whereas it should never end");
}

async fn send_heartbeat(url: &Url, worker_name: &str, worker_version: &str) -> anyhow::Result<()> {
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
