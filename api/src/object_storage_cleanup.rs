use actix_web::rt::time::sleep;
use aws_sdk_s3::types::{Delete, ObjectIdentifier};
use chrono::NaiveDate;
use log::{error, info, trace};
use sqlx::{PgPool, query_as};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::ObjectStorageConfig;

const STARTUP_GRACE_PERIOD: Duration = Duration::from_secs(2 * 60);

pub async fn periodically_clean_up_object_storage(
    db: &PgPool,
    object_storage: &ObjectStorageConfig,
    period: Duration,
    delete: bool,
) {
    sleep(STARTUP_GRACE_PERIOD).await;

    loop {
        if let Err(e) =
            delete_dangling_objects_from_object_storage(db, object_storage, delete).await
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
) -> anyhow::Result<()> {
    trace!("Start cleaning up object storage...");
    let start = Instant::now();

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
                MIN(epd.date) AS oldest_event_date
            FROM event.application AS a
            LEFT JOIN event.events_per_day AS epd ON epd.application__id = a.application__id
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

    let (applications_in_object_storage, lost_applications) = {
        let mut applications = Vec::new();
        let mut continuation_token = Some(String::new());

        while let Some(ct) = continuation_token {
            let applications_list = object_storage
                .client
                .list_objects_v2()
                .bucket(&object_storage.bucket)
                .delimiter("/")
                .max_keys(1)
                .set_continuation_token(if ct.is_empty() { None } else { Some(ct) })
                .send()
                .await?;
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
            Some(kv) => acc.push(kv),
            None => lost_acc.push(cur.to_string()),
        };
        (acc, lost_acc)
    });

    if !lost_applications.is_empty() {
        error!(
            "Some applications exist in object storage but not in database (you should investigate and maybe remove them from object storage manually): {}",
            lost_applications.join(", ")
        );
    }

    trace!("Listing object storage prefixes that should be deleted...");
    let mut prefixes_to_delete = Vec::new();
    for (application_id, oldest_event_date) in applications_in_object_storage {
        let mut event_prefixes = {
            let mut dates = Vec::new();
            let mut continuation_token = Some(String::new());

            while let Some(ct) = continuation_token {
                let dates_list = object_storage
                    .client
                    .list_objects_v2()
                    .bucket(&object_storage.bucket)
                    .delimiter("/")
                    .prefix(format!("{application_id}/event/"))
                    .max_keys(1)
                    .set_continuation_token(if ct.is_empty() { None } else { Some(ct) })
                    .send()
                    .await?;
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

            dates
        }
        .into_iter()
        .filter(|d| d < oldest_event_date)
        .map(|d| format!("{application_id}/event/{d}/"))
        .collect::<Vec<_>>();

        prefixes_to_delete.append(&mut event_prefixes);
    }

    if !prefixes_to_delete.is_empty() {
        info!(
            "The following object storage prefixes are out of retention period: {}",
            prefixes_to_delete.join(", ")
        );
    }

    if delete {
        let mut total_deleted_objects = 0;

        for prefix in &prefixes_to_delete {
            trace!("Deleting prefix '{prefix}' from object storage");
            let mut deleted_objects_for_current_prefix = 0;
            let mut continuation_token = Some(String::new());

            while let Some(ct) = continuation_token {
                let objects = object_storage
                    .client
                    .list_objects_v2()
                    .bucket(&object_storage.bucket)
                    .delimiter("/")
                    .prefix(prefix)
                    .max_keys(2)
                    .set_continuation_token(if ct.is_empty() { None } else { Some(ct) })
                    .send()
                    .await?;
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
                    total_deleted_objects += del.objects().len();
                    deleted_objects_for_current_prefix += del.objects().len();

                    object_storage
                        .client
                        .delete_objects()
                        .bucket(&object_storage.bucket)
                        .delete(del)
                        .send()
                        .await?;
                };
                continuation_token = objects.next_continuation_token().map(|ct| ct.to_owned());
            }

            if deleted_objects_for_current_prefix > 0 {
                trace!(
                    "Deleted {deleted_objects_for_current_prefix} objects for prefix '{prefix}'"
                );
            }
        }

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
