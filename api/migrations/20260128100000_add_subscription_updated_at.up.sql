alter table webhook.subscription add column updated_at timestamptz not null default statement_timestamp();
update webhook.subscription set updated_at = created_at;
