create or replace function event.dispatch()
    returns trigger
    language plpgsql
as
$$
declare
    subscription_id uuid;
begin
    if new.dispatched_at is not null then
        return new;
    end if;

    for subscription_id in
        select s.subscription__id
        from webhook.subscription as s
                  inner join webhook.subscription__event_type as set on set.subscription__id = s.subscription__id
        where s.is_enabled
          and s.application__id = new.application__id
          and s.deleted_at is null
          and set.event_type__name = new.event_type__name
          and new.labels @> s.labels
        loop
            raise notice '[event %] matching subscription: %', new.event__id, subscription_id;
            insert into webhook.request_attempt (event__id, subscription__id)
            values (new.event__id, subscription_id);
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

alter table webhook.request_attempt drop column application__id;
