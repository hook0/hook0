alter table webhook.subscription add column label_key text;
alter table webhook.subscription add column label_value text;
update webhook.subscription set
  label_key = (array(select jsonb_object_keys(labels)))[1],
  label_value = labels->>(array(select jsonb_object_keys(labels)))[1];
alter table webhook.subscription alter column label_key set not null;
alter table webhook.subscription alter column label_value set not null;
alter table webhook.subscription drop column labels;

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
    if new.dispatched_at is not null then
        return new;
    end if;

    for key, value in select * from jsonb_each_text(new.labels) limit 50
        loop
            for subscription_id in
                select s.subscription__id
                from webhook.subscription as s
                         inner join webhook.subscription__event_type as set on set.subscription__id = s.subscription__id
                where s.is_enabled
                  and s.application__id = new.application__id
                  and s.deleted_at is null
                  and set.event_type__name = new.event_type__name
                  and s.label_key = key
                  and s.label_value = value
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

drop trigger event_dispatch on event.event;

create trigger event_dispatch
    after insert or update
    on event.event
    for each row
execute function event.dispatch();
