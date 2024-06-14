create collation if not exists case_insensitive (provider = icu, locale = 'und-u-ks-level2', deterministic = false);

alter table iam.user alter column email set data type text collate case_insensitive;
