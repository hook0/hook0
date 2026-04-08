use actix_web::rt::time::sleep;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::timeout::TimeoutConfig;
use aws_sdk_s3::error::{DisplayErrorContext, SdkError};
use aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Error;
use aws_sdk_s3::types::{Delete, ObjectIdentifier};
use chrono::NaiveDate;
use futures_util::stream::{self, StreamExt as _};
use sqlx::{PgPool, query, query_as};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tracing::{error, info, trace, warn};
use uuid::Uuid;

use crate::ObjectStorageConfig;
use crate::opentelemetry::report_cleaned_up_objects;
use hook0_sentry_integration::log_object_storage_error_with_context;

const STARTUP_GRACE_PERIOD: Duration = Duration::from_secs(2 * 60);
/// Delay between processing each application to reduce burst load on the S3 backend.
const INTER_APP_DELAY: Duration = Duration::from_millis(100);

#[allow(clippy::too_many_arguments)]
pub async fn periodically_clean_up_object_storage(
    db: &PgPool,
    object_storage: &ObjectStorageConfig,
    period: Duration,
    delete: bool,
    collect_concurrency: u8,
    delete_concurrency: u8,
    operation_attempt_timeout: Duration,
    operation_timeout: Duration,
) {
    sleep(STARTUP_GRACE_PERIOD).await;

    loop {
        if let Err(e) = delete_dangling_objects_from_object_storage(
            db,
            object_storage,
            delete,
            collect_concurrency,
            delete_concurrency,
            operation_attempt_timeout,
            operation_timeout,
        )
        .await
        {
            error!("Could not clean up object storage: {e}");
        }

        sleep(period).await;
    }
}

async fn delete_dangling_objects_from_object_storage(
    db: &PgPool,
    object_storage: &ObjectStorageConfig,
    delete: bool,
    collect_concurrency: u8,
    delete_concurrency: u8,
    operation_attempt_timeout: Duration,
    operation_timeout: Duration,
) -> anyhow::Result<()> {
    trace!("Start cleaning up object storage...");
    let start = Instant::now();

    // Build a dedicated S3 client with relaxed timeouts for the cleanup task.
    // Listing many objects can be slow; this should not be constrained by the
    // tight timeouts used for latency-sensitive API operations.
    // We clone the existing config and only override timeout settings.
    let mut cleanup_timeout_builder = TimeoutConfig::builder()
        .operation_attempt_timeout(operation_attempt_timeout)
        .operation_timeout(operation_timeout);
    // Preserve connect and read timeouts from the existing client.
    if let Some(existing) = object_storage.client.config().timeout_config() {
        if let Some(ct) = existing.connect_timeout() {
            cleanup_timeout_builder = cleanup_timeout_builder.connect_timeout(ct);
        }
        if let Some(rt) = existing.read_timeout() {
            cleanup_timeout_builder = cleanup_timeout_builder.read_timeout(rt);
        }
    }
    let cleanup_config = object_storage
        .client
        .config()
        .to_builder()
        .timeout_config(cleanup_timeout_builder.build())
        .build();
    let client = Client::from_conf(cleanup_config);

    trace!("Listing applications with their oldest event date...");
    struct ApplicationWithOldestEventDate {
        application_id: Uuid,
        oldest_event_date: Option<NaiveDate>,
    }
    let applications_with_oldest_event_date = query_as!(
        ApplicationWithOldestEventDate,
        "
            SELECT
                a.application__id AS application_id,
                MIN(e.received_at)::date AS oldest_event_date
            FROM event.application AS a
            INNER JOIN event.event AS e ON e.application__id = a.application__id
            GROUP BY a.application__id
        "
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|r| {
        (
            r.application_id,
            r.oldest_event_date.unwrap_or(NaiveDate::MIN),
        )
    })
    .collect::<HashMap<_, _>>();

    let (mut applications_in_object_storage, lost_applications) = {
        let mut applications = Vec::new();
        let mut continuation_token = Some(String::new());
        let mut page = 0u64;

        while let Some(ct) = continuation_token {
            let page_start = Instant::now();
            let applications_list = client
                .list_objects_v2()
                .bucket(&object_storage.bucket)
                .delimiter("/")
                .set_continuation_token(if ct.is_empty() { None } else { Some(ct) })
                .send()
                .await
                .inspect_err(|e| {
                    log_object_storage_error_with_context!(
                        "S3 LIST OBJECTS v2 failed while listing applications",
                        error_chain = DisplayErrorContext(e).to_string(),
                    );
                })?;
            let prefixes_count = applications_list.common_prefixes().len();
            trace!(
                "Listed applications page {page}: {prefixes_count} prefixes in {:?}",
                page_start.elapsed()
            );
            page += 1;
            applications.append(
                &mut applications_list
                    .common_prefixes()
                    .iter()
                    .filter_map(|cp| {
                        cp.prefix
                            .as_deref()
                            .and_then(|p| p.strip_suffix("/"))
                            .and_then(|p| Uuid::parse_str(p).ok())
                    })
                    .collect::<Vec<_>>(),
            );
            continuation_token = applications_list
                .next_continuation_token()
                .map(|ct| ct.to_owned());
        }

        applications
    }
    .into_iter()
    .fold((Vec::new(), Vec::new()), |(mut acc, mut lost_acc), cur| {
        match applications_with_oldest_event_date.get_key_value(&cur) {
            Some((k, v)) => acc.push((*k, *v)),
            None => lost_acc.push(cur),
        };
        (acc, lost_acc)
    });

    let mut unknown_applications = Vec::new();
    for a in lost_applications {
        let has_existed = query!(
            "
                SELECT true AS whatever
                FROM event.all_time_events_per_day
                WHERE application__id = $1
                LIMIT 1
            ",
            a
        )
        .fetch_optional(db)
        .await?
        .is_some();

        if has_existed {
            applications_in_object_storage.push((a, NaiveDate::MAX));
        } else {
            unknown_applications.push(a.to_string());
        }
    }

    if !unknown_applications.is_empty() {
        error!(
            "Some applications exist in object storage but not in database (you should investigate and maybe remove them from object storage manually): {}",
            unknown_applications.join(", ")
        );
    }

    trace!(
        "Listing object storage prefixes that should be deleted (concurrency: {collect_concurrency})..."
    );
    let failed_applications = Arc::new(AtomicU64::new(0));

    let prefixes_to_delete: Vec<String> = stream::iter(applications_in_object_storage)
        .map(|(application_id, oldest_event_date)| {
            let client = client.clone();
            let bucket = object_storage.bucket.clone();
            let failed_apps = Arc::clone(&failed_applications);
            async move {
                sleep(INTER_APP_DELAY).await;
                match collect_prefixes_for_application(
                    &client,
                    &bucket,
                    application_id,
                    oldest_event_date,
                )
                .await
                {
                    Ok(app_prefixes) => app_prefixes,
                    Err(e) => {
                        failed_apps.fetch_add(1, Ordering::Relaxed);
                        error!("Failed to list prefixes for application {application_id}, skipping: {e}");
                        let app_id_str = application_id.to_string();
                        log_object_storage_error_with_context!(
                            "S3 LIST OBJECTS v2 failed while listing prefixes for application",
                            error_chain = DisplayErrorContext(&e).to_string(),
                            prefix = app_id_str.as_str(),
                        );
                        Vec::new()
                    }
                }
            }
        })
        .buffer_unordered(usize::from(collect_concurrency))
        .collect::<Vec<Vec<String>>>()
        .await
        .into_iter()
        .flatten()
        .collect();

    let failed_applications = failed_applications.load(Ordering::Relaxed);
    if failed_applications > 0 {
        warn!("Skipped {failed_applications} applications due to errors");
    }

    if !prefixes_to_delete.is_empty() {
        info!(
            "The following object storage prefixes are out of retention period: {}",
            prefixes_to_delete.join(", ")
        );
    }

    if delete {
        trace!("Deleting object storage prefixes (concurrency: {delete_concurrency})...");
        let total_deleted_objects = Arc::new(AtomicU64::new(0));

        stream::iter(&prefixes_to_delete)
            .for_each_concurrent(usize::from(delete_concurrency), |prefix| {
                let client = client.clone();
                let bucket = object_storage.bucket.clone();
                let total = Arc::clone(&total_deleted_objects);
                async move {
                    match delete_prefix_objects(&client, &bucket, prefix).await {
                        Ok(deleted_count) => {
                            total.fetch_add(deleted_count, Ordering::Relaxed);
                        }
                        Err(e) => {
                            error!("Failed to delete objects for prefix '{prefix}': {e}");
                        }
                    }
                }
            })
            .await;

        let total_deleted_objects = total_deleted_objects.load(Ordering::Relaxed);
        info!(
            "Cleaned up {total_deleted_objects} dangling objects from {} object storage prefixes in {:?}",
            prefixes_to_delete.len(),
            start.elapsed()
        );
    } else {
        info!(
            "Could clean up dangling objects from {} object storage prefixes but actual cleaning is not enabled (scan done in {:?})",
            prefixes_to_delete.len(),
            start.elapsed()
        )
    }

    Ok(())
}

async fn collect_prefixes_for_application(
    client: &Client,
    bucket: &str,
    application_id: Uuid,
    oldest_event_date: NaiveDate,
) -> Result<Vec<String>, SdkError<ListObjectsV2Error>> {
    let (mut event_prefixes, response_prefixes) = tokio::try_join!(
        list_prefixes_for_kind(client, bucket, application_id, oldest_event_date, "event"),
        list_prefixes_for_kind(
            client,
            bucket,
            application_id,
            oldest_event_date,
            "response"
        ),
    )?;
    event_prefixes.extend(response_prefixes);
    Ok(event_prefixes)
}

async fn list_prefixes_for_kind(
    client: &Client,
    bucket: &str,
    application_id: Uuid,
    oldest_event_date: NaiveDate,
    kind: &str,
) -> Result<Vec<String>, SdkError<ListObjectsV2Error>> {
    let mut dates = Vec::new();
    let mut continuation_token = Some(String::new());
    let pfx = format!("{application_id}/{kind}/");
    let mut page = 0u64;

    while let Some(ct) = continuation_token {
        let page_start = Instant::now();
        let dates_list = client
            .list_objects_v2()
            .bucket(bucket)
            .delimiter("/")
            .prefix(&pfx)
            .set_continuation_token(if ct.is_empty() { None } else { Some(ct) })
            .send()
            .await?;
        let prefixes_count = dates_list.common_prefixes().len();
        trace!(
            "Listed {kind} date prefixes for {application_id} page {page}: {prefixes_count} prefixes in {:?}",
            page_start.elapsed()
        );
        page += 1;
        dates.append(
            &mut dates_list
                .common_prefixes()
                .iter()
                .filter_map(|cp| {
                    cp.prefix
                        .as_deref()
                        .and_then(|p| p.strip_suffix("/"))
                        .and_then(|p| p.split("/").last())
                        .and_then(|p| NaiveDate::from_str(p).ok())
                })
                .collect::<Vec<_>>(),
        );
        continuation_token = dates_list.next_continuation_token().map(|ct| ct.to_owned());
    }

    Ok(dates
        .into_iter()
        .filter(|d| d < &oldest_event_date)
        .map(|d| format!("{application_id}/{kind}/{d}/"))
        .collect())
}

async fn delete_prefix_objects(client: &Client, bucket: &str, prefix: &str) -> anyhow::Result<u64> {
    trace!("Deleting prefix '{prefix}' from object storage");
    let mut deleted_objects_for_current_prefix = 0u64;
    let mut continuation_token = Some(String::new());
    let mut page = 0u64;

    while let Some(ct) = continuation_token {
        let page_start = Instant::now();
        let objects = client
            .list_objects_v2()
            .bucket(bucket)
            .delimiter("/")
            .prefix(prefix)
            .set_continuation_token(if ct.is_empty() { None } else { Some(ct) })
            .send()
            .await
            .inspect_err(|e| {
                log_object_storage_error_with_context!(
                    "S3 LIST OBJECTS v2 failed while listing objects to delete",
                    error_chain = DisplayErrorContext(e).to_string(),
                    prefix = prefix,
                );
            })?;
        let contents_count = objects.contents().len();
        trace!(
            "Listed objects to delete for prefix '{prefix}' page {page}: {contents_count} objects in {:?}",
            page_start.elapsed()
        );
        page += 1;
        let delete = {
            let mut d = Delete::builder();
            for oi in objects
                .contents()
                .iter()
                .filter_map(|o| o.key())
                .filter_map(|k| ObjectIdentifier::builder().key(k).build().ok())
            {
                d = d.objects(oi);
            }
            d.build().ok()
        };
        if let Some(del) = delete {
            let resp = client
                .delete_objects()
                .bucket(bucket)
                .delete(del)
                .send()
                .await
                .inspect_err(|e| {
                    log_object_storage_error_with_context!(
                        "S3 DELETE OBJECTS failed",
                        error_chain = DisplayErrorContext(e).to_string(),
                        prefix = prefix,
                    );
                })?;

            let actual_deleted: u64 = resp.deleted().len().try_into().unwrap_or(0);
            deleted_objects_for_current_prefix += actual_deleted;
            report_cleaned_up_objects(actual_deleted);

            let errors = resp.errors();
            if !errors.is_empty() {
                warn!(
                    "S3 DELETE OBJECTS had {} per-object errors for prefix '{prefix}'",
                    errors.len(),
                );
                for err in errors {
                    warn!(
                        "  S3 delete error: key={:?} code={:?} message={:?}",
                        err.key().unwrap_or("?"),
                        err.code().unwrap_or("?"),
                        err.message().unwrap_or("?"),
                    );
                }
            }
        };
        continuation_token = objects.next_continuation_token().map(|ct| ct.to_owned());
    }

    if deleted_objects_for_current_prefix > 0 {
        trace!("Deleted {deleted_objects_for_current_prefix} objects for prefix '{prefix}'");
    }

    Ok(deleted_objects_for_current_prefix)
}
