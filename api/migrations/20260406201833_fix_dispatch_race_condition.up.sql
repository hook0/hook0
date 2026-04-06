create or replace function event.dispatch()
    returns trigger
    language plpgsql
as
$$
begin
    if new.dispatched_at is not null then
        return new;
    end if;

    insert into webhook.request_attempt (event__id, subscription__id, application__id)
    select new.event__id, s.subscription__id, s.application__id
    from webhook.subscription as s
    inner join webhook.subscription__event_type as set on set.subscription__id = s.subscription__id
    where s.is_enabled
      and s.application__id = new.application__id
      and s.deleted_at is null
      and set.event_type__name = new.event_type__name
      and new.labels @> s.labels
    for share of s;

    update event.event set dispatched_at = statement_timestamp() where event__id = new.event__id;
    return new;
end;
$$;
