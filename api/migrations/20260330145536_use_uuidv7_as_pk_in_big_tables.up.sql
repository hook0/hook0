alter table event.event alter column event__id set default uuidv7();
alter table webhook.request_attempt alter column request_attempt__id set default uuidv7();
alter table webhook.response alter column response__id set default uuidv7();
