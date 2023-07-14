alter table webhook.request_attempt rename column worker_name to worker_id;

drop table webhook.subscription__worker;

drop table iam.organization__worker;

drop table infrastructure.worker;

drop schema infrastructure;
