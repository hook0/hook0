-- 'system' = automatic (dispatch trigger, worker retry)
-- 'user' = manual retry via API
-- default 'system' handles existing rows and dispatch trigger INSERTs
alter table webhook.request_attempt
    add column source text not null default 'system'
        check (source in ('system', 'user'));

-- NULL = system or service token, NOT NULL = action by this user
alter table webhook.request_attempt
    add column user__id uuid
        references iam.user(user__id)
        on delete set null;

-- source = 'system' must have user__id = NULL
alter table webhook.request_attempt
    add constraint request_attempt_source_user_check
        check (source != 'system' or user__id is null);
