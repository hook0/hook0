alter table infrastructure.worker add column queue_type text not null default 'pg';

alter table infrastructure.worker add constraint queue_type_chk check (queue_type in ('pg', 'pulsar'));
