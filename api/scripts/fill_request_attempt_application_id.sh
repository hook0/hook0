#!/usr/bin/env bash
set -euo pipefail

# This script can be used to fill the new application__id (default null) column that was added to webhook.request_attempt in migration 20251204171458_add_application_id_to_request_attempt_table.
# A following migration will SET NOT NULL on this column, so existing data must be prepared first.
#
# The script must be provided with a PostgreSQL connection string as first and only argument; for example: postgres://user:password@host:port/database?application_name=script_fill_request_attempt_application_id
# It updates existing rows in webhook.request_attempt by batches of 100,000 in order to avoid long locks.
# It also runs VACUUM between every batch so that table does not waste disk space.

db="$1"
completion_status="0"

update_query="update webhook.request_attempt as ra
set application__id = batch.application__id
from (
  select ra.request_attempt__id, s.application__id
  from webhook.request_attempt as ra
  inner join webhook.subscription as s on s.subscription__id = ra.subscription__id
  where ra.application__id is null
  limit 100000
) as batch
where batch.request_attempt__id = ra.request_attempt__id;"

function check() {
  completion_status="$(psql -d "$db" -qtAX -c "select count(request_attempt__id) filter (where application__id is not null)::numeric * 100 / count(request_attempt__id) from webhook.request_attempt;")"
}

function display() {
  printf "$(date --rfc-3339 seconds) %.1f%% done\n" "$completion_status"
}

check
display
while [ "$(echo "$completion_status < 100.0" | bc)" -eq 1  ]; do
  psql -d "$db" -c "$update_query" -c "vacuum webhook.request_attempt;"
  check
  display
done

psql -d "$db" -c "vacuum analyze webhook.request_attempt;"
echo "Done"
