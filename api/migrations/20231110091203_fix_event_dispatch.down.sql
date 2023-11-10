create or replace function event.dispatch()
    returns trigger
    language plpgsql
as
$$
declare
    key text;
    value text;
    subscription_id uuid;
begin
    for key, value in select * from jsonb_each_text(new.labels) limit 50
        loop
            for subscription_id in
                select subscription__id
                from webhook.subscription
                where is_enabled
                  and deleted_at is null
                  and label_key = key
                  and label_value = value
                loop
                    raise notice '[event %] matching subscription: %', new.event__id, subscription_id;
                    insert into webhook.request_attempt (event__id, subscription__id)
                    values (new.event__id, subscription_id);
                end loop;
        end loop;
    update event.event set dispatched_at = statement_timestamp() where event__id = new.event__id;
    return new;
end;
$$;
