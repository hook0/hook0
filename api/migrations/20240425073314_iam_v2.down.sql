alter table event.event alter column application_secret__token set not null;
drop table iam.token;
drop table iam.user__organization;
drop table iam.user;
