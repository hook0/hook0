//! The Zig client against a Hook0 that is really running.
//!
//! Two things happen here, and the second is the reason the first is worth having.
//!
//! The control: whether an application secret the API minted is accepted, whether a second send
//! under an identifier already ingested is reported as the conflict it is, and whether a signature
//! the output worker computed verifies. Those are the three questions no loopback suite can ask
//! itself, because a suite that signs and verifies with the same sources only proves the sources
//! agree with themselves.
//!
//! The surface: every operation the API document declares, driven through the generated layer
//! against the same instance, and every model type it decodes out of a real answer.
//! `clients/zig/tests` already drives all of them — against an API the suite itself writes, out of
//! the same document the client was generated from. That proves the client matches the document. It
//! cannot prove the document matches Hook0, and a field the API really answers under another name
//! passes there and fails on a consumer's first call.

const std = @import("std");
const hook0 = @import("hook0");

/// The conflict the API answers a duplicated ingestion with.
const already_ingested = "EventAlreadyIngested";

/// What this smoke labels everything it creates with, so that the subscription it makes and the
/// event it sends find each other.
const language = "zig";

/// Where the subscription this smoke creates points. Nothing listens there, deliberately: what a
/// delivery proves is proved once, by the webhook the harness catches and every language verifies.
const nowhere = "http://127.0.0.1:1/";

/// The most bytes of one part of the delivery read back. Every one of them is written by the
/// harness a moment earlier and measured in hundreds of bytes.
const max_part_bytes = 1024 * 1024;

/// The most bytes one setting may be spelled with.
const max_setting_bytes = 4096;

/// What a paced instance answers.
const too_many_requests: u16 = 429;

/// The most times one request is sent again after that answer.
const paced_again: usize = 8;

/// The shortest this waits between two tries, and the longest whatever the answer asked for.
const shortest_pause_ms: u64 = 200;
const longest_pause_ms: u64 = 10_000;

/// The most digits a `Retry-After` may carry before it is read as the floor above rather than as a
/// number. A header is written by a server this smoke does not control.
const most_digits: usize = 9;

/// What the harness passes, taken from the process rather than from a clock or a global: the
/// entry point is handed the environment, the allocator and the `Io` this program runs on.
pub fn main(init: std.process.Init) !void {
    const allocator = init.gpa;
    const io = init.io;

    var said: [4096]u8 = undefined;
    var out = std.Io.File.stdout().writerStreaming(io, &said);

    smoke(allocator, io, init.environ_map, &out.interface) catch |refused| {
        out.interface.flush() catch {};
        std.debug.print("the zig smoke was refused: {t}\n", .{refused});
        std.process.exit(1);
    };
}

fn smoke(
    allocator: std.mem.Allocator,
    io: std.Io,
    environment: *std.process.Environ.Map,
    out: *std.Io.Writer,
) !void {
    try sendTwice(allocator, io, environment, out);
    try surface(allocator, io, environment, out);

    // Last, and on purpose: it needs no instance at all, so it still answers after the flow above
    // has deleted the application it was run against.
    try verify(allocator, io, try setting(environment, "HOOK0_DELIVERY"));
    try out.print("the signature the instance produced verifies\n", .{});
    try out.flush();
}

/// The same event, twice, under the identifier the API minted for the first of them.
fn sendTwice(
    allocator: std.mem.Allocator,
    io: std.Io,
    environment: *std.process.Environ.Map,
    out: *std.Io.Writer,
) !void {
    const event_type = try setting(environment, "HOOK0_EVENT_TYPE");

    var client: hook0.Client = .init(
        io,
        try setting(environment, "HOOK0_API_URL"),
        try setting(environment, "HOOK0_APPLICATION_ID"),
        try setting(environment, "HOOK0_TOKEN"),
        .{},
    );

    const sent = try client.sendEvent(allocator, event(event_type, null));
    defer sent.deinit();
    try out.print("ingested {s}\n", .{sent.value});

    const identifier = try allocator.dupe(u8, sent.value);
    defer allocator.free(identifier);

    if (client.sendEvent(allocator, event(event_type, identifier))) |accepted| {
        accepted.deinit();
        std.debug.print("sending the same event twice was accepted twice\n", .{});
        return error.DuplicateAccepted;
    } else |_| {
        if (std.mem.indexOf(u8, client.detail, already_ingested) == null) {
            std.debug.print(
                "the second send failed without naming {s}: {s}\n",
                .{ already_ingested, client.detail },
            );
            return error.ConflictNotReported;
        }
    }
    try out.print("the second send reported {s}\n", .{already_ingested});
    try out.flush();
}

/// Every operation the API document declares, driven against the instance in the order a consumer
/// would: what it needs is created, read and listed, updated, and destroyed last.
///
/// Two credentials, because the API takes two and one of them cannot do everything. An application
/// secret is scoped to the application it belongs to; what belongs to the organization — listing
/// its applications, everything about service tokens, its per-day counts — needs the
/// organization-scoped token beside it.
fn surface(
    allocator: std.mem.Allocator,
    io: std.Io,
    environment: *std.process.Environ.Map,
    out: *std.Io.Writer,
) !void {
    const origin = try originOf(try setting(environment, "HOOK0_API_URL"));
    const application = try setting(environment, "HOOK0_APPLICATION_ID");
    const organization = try setting(environment, "HOOK0_ORGANIZATION_ID");
    const seeded = try setting(environment, "HOOK0_SEEDED_APPLICATION_ID");
    const labels: hook0.runtime.Map([]const u8) = .{
        .entries = &.{.{ .key = "language", .value = language }},
    };

    var reaching: hook0.Transport = .{
        .io = io,
        .base_url = origin,
        .token = try setting(environment, "HOOK0_TOKEN"),
    };
    var held: Paced = .{ .inner = &reaching, .io = io };

    var reaching_organization: hook0.Transport = .{
        .io = io,
        .base_url = origin,
        .token = try setting(environment, "HOOK0_SERVICE_TOKEN"),
    };
    var organization_wide: Paced = .{ .inner = &reaching_organization, .io = io };

    // The allocator each group is built with is where what a failure of it reported is read into,
    // which is why it is freed here rather than by the call that drew it: a call frees everything
    // it allocated on its way out, and what the failure said has to be readable after that.
    var applications: hook0.api.ApplicationsApi = .init(allocator, held.any());
    var secrets: hook0.api.ApplicationSecretsApi = .init(allocator, held.any());
    var event_types: hook0.api.EventTypesApi = .init(allocator, held.any());
    var subscriptions: hook0.api.SubscriptionsApi = .init(allocator, held.any());
    var events: hook0.api.EventsApi = .init(allocator, held.any());
    var events_per_day: hook0.api.EventsPerDayApi = .init(allocator, held.any());
    var instance: hook0.api.InstanceApi = .init(allocator, held.any());
    var quotas: hook0.api.QuotasApi = .init(allocator, held.any());
    var payload_content_types: hook0.api.PayloadContentTypesApi = .init(allocator, held.any());
    var error_catalogue: hook0.api.ErrorsApi = .init(allocator, held.any());

    var organization_applications: hook0.api.ApplicationsApi = .init(allocator, organization_wide.any());
    var organization_events_per_day: hook0.api.EventsPerDayApi = .init(allocator, organization_wide.any());
    var request_attempts: hook0.api.RequestAttemptsApi = .init(allocator, organization_wide.any());
    var responses: hook0.api.ResponseApi = .init(allocator, organization_wide.any());
    var service_tokens: hook0.api.ServiceTokenApi = .init(allocator, organization_wide.any());

    defer {
        applications.deinit();
        secrets.deinit();
        event_types.deinit();
        subscriptions.deinit();
        events.deinit();
        events_per_day.deinit();
        instance.deinit();
        quotas.deinit();
        payload_content_types.deinit();
        error_catalogue.deinit();
        organization_applications.deinit();
        organization_events_per_day.deinit();
        request_attempts.deinit();
        responses.deinit();
        service_tokens.deinit();
    }

    // What the instance says about itself, which is what an application asks before it has anything
    // of its own: how it is configured, what it will let this account do, what a payload may be,
    // and every problem it can report.
    const configured = try read(out, "instance.get", instance.get(allocator));
    defer configured.deinit();
    try decoded(out, "InstanceConfig", configured.value);

    const allowed = try read(out, "quotas.get", quotas.get(allocator));
    defer allowed.deinit();
    try decoded(out, "QuotasResponseLimits", allowed.value.limits);
    try decoded(out, "QuotasResponse", allowed.value);

    try exercised(out, "payload_content_types.list", payload_content_types.list(allocator));

    const catalogue = try read(out, "errors.list", error_catalogue.list(allocator));
    defer catalogue.deinit();
    if (catalogue.value.len == 0) {
        std.debug.print(
            "the instance published an empty catalogue of the problems it can report\n",
            .{},
        );
        return error.NoProblemsPublished;
    }
    try decoded(out, "ProblemId", catalogue.value[0].id);
    try decoded(out, "Problem", catalogue.value[0]);

    // The application this smoke owns. One per language, so that the three deletions at the end of
    // this flow are real deletions rather than something eleven other smokes have to live with.
    const info = try read(out, "applications.get", applications.get(allocator, application));
    defer info.deinit();
    try decoded(out, "ApplicationInfoConsumption", info.value.consumption);
    try decoded(out, "ApplicationInfoQuotas", info.value.quotas);
    try decoded(out, "ApplicationInfoOnboardingStepsEvent", info.value.onboarding_steps.event);
    try decoded(
        out,
        "ApplicationInfoOnboardingStepsEventType",
        info.value.onboarding_steps.event_type,
    );
    try decoded(
        out,
        "ApplicationInfoOnboardingStepsSubscription",
        info.value.onboarding_steps.subscription,
    );
    try decoded(out, "ApplicationInfoOnboardingSteps", info.value.onboarding_steps);
    try decoded(out, "ApplicationInfo", info.value);

    const renamed = try read(out, "applications.update", applications.update(
        allocator,
        application,
        .{ .name = "the application the zig smoke drives", .organization_id = organization },
    ));
    defer renamed.deinit();
    try decoded(out, "Application", renamed.value);

    // The organization's, so the organization credential. Listing what an account has is the first
    // thing a console does.
    try exercised(
        out,
        "applications.list",
        organization_applications.list(allocator, organization),
    );

    // This one is driven with the *application* secret on purpose, and it is the flow's one
    // refusal. Creating an application is the organization's business and an application secret is
    // not the organization's, so the instance answers a problem document and this client reads it —
    // which is the half of the client that nothing else here would exercise.
    try exercised(out, "applications.create", applications.create(allocator, .{
        .name = "an application the zig smoke's application secret may not create",
        .organization_id = organization,
    }));

    // A second secret, so that the one this smoke is authenticating with is never the one it
    // revokes. Deleting that one succeeds and then locks the flow out of everything below.
    const minted = try read(out, "applicationSecrets.create", secrets.create(allocator, .{
        .application_id = application,
        .name = "a secret the zig smoke minted",
    }));
    defer minted.deinit();
    try decoded(out, "ApplicationSecret", minted.value);

    try exercised(out, "applicationSecrets.read", secrets.read(allocator, application));
    try exercised(out, "applicationSecrets.update", secrets.update(
        allocator,
        minted.value.token,
        .{ .application_id = application, .name = "a secret the zig smoke renamed" },
    ));
    try exercised(
        out,
        "applicationSecrets.delete",
        secrets.delete(allocator, minted.value.token, application),
    );

    // An event type of this smoke's own, rather than the one the harness declared: what is created
    // here is what is subscribed to, sent, replayed and deleted below.
    const declared = try read(out, "eventTypes.create", event_types.create(allocator, .{
        .application_id = application,
        .resource_type = "smoke",
        .service = language,
        .verb = "ran",
    }));
    defer declared.deinit();
    try decoded(out, "EventType", declared.value);

    try exercised(
        out,
        "eventTypes.get",
        event_types.get(allocator, declared.value.event_type_name, application),
    );
    try exercised(out, "eventTypes.list", event_types.list(allocator, application));

    // The headers of a target are a mapping the API reads, and an empty one still has to go out as
    // an object rather than as nothing.
    const target: hook0.models.SubscriptionPostTarget = .{
        .headers = .{ .object = .empty },
        .method = "POST",
        .type = "http",
        .url = nowhere,
    };
    const subscription = try read(out, "subscriptions.create", subscriptions.create(allocator, .{
        .application_id = application,
        .event_types = &.{declared.value.event_type_name},
        .is_enabled = true,
        .target = target,
        .dedicated_workers = null,
        .description = "what the zig smoke subscribes to its own events with",
        .label_key = null,
        .label_value = null,
        .labels = labels,
        .metadata = null,
    }));
    defer subscription.deinit();
    try decoded(out, "SubscriptionTarget", subscription.value.target);
    try decoded(out, "Subscription", subscription.value);

    try exercised(
        out,
        "subscriptions.get",
        subscriptions.get(allocator, subscription.value.subscription_id),
    );
    try exercised(out, "subscriptions.list", subscriptions.list(allocator, application));
    try exercised(out, "subscriptions.update", subscriptions.update(
        allocator,
        subscription.value.subscription_id,
        .{
            .application_id = application,
            .event_types = &.{declared.value.event_type_name},
            .is_enabled = true,
            .target = target,
            .dedicated_workers = null,
            .description = "what the zig smoke renamed it to",
            .label_key = null,
            .label_value = null,
            .labels = labels,
            .metadata = null,
        },
    ));

    // The event the subscription above selects, sent through the generated layer rather than
    // through sendEvent: the hand-written half has its own three questions above, and this is the
    // operation the document declares.
    var minted_id: [36]u8 = undefined;
    hook0.generateEventId(io, &minted_id);
    var occurred: [20]u8 = undefined;
    const ingested = try read(out, "events.ingest", events.ingest(allocator, .{
        .application_id = application,
        .event_type = declared.value.event_type_name,
        .labels = labels,
        .occurred_at = try moment(io, &occurred),
        .payload = "{\"from\":\"the zig smoke\"}",
        .payload_content_type = "application/json",
        .event_id = &minted_id,
        .metadata = null,
    }));
    defer ingested.deinit();
    try decoded(out, "IngestedEvent", ingested.value);

    const whole = try read(
        out,
        "events.get",
        events.get(allocator, ingested.value.event_id, application),
    );
    defer whole.deinit();
    try decoded(out, "EventWithPayload", whole.value);

    const listed = try read(out, "events.list", events.list(allocator, application));
    defer listed.deinit();
    if (listed.value.len == 0) {
        std.debug.print("the instance ingested an event and then listed none\n", .{});
        return error.NothingListed;
    }
    try decoded(out, "Event", listed.value[0]);

    try exercised(out, "events.replay", events.replay(
        allocator,
        ingested.value.event_id,
        .{ .application_id = application },
    ));

    // This application was created a moment ago and the counts come out of a view the instance
    // refreshes on a cycle of its own, so this answers a list with nothing in it — which is an
    // answer, and one a client has to be able to read.
    try exercised(out, "events_per_day.list_for_application", events_per_day.listForApplication(
        allocator,
        application,
        null,
        null,
    ));

    // The organization's counts do have something in them: the harness waited for the instance to
    // refresh them before running any of this, precisely so that the type they are answered with is
    // one a client decodes rather than one nothing ever produces.
    const per_day = try read(
        out,
        "events_per_day.list_for_organization",
        organization_events_per_day.listForOrganization(allocator, organization, null, null),
    );
    defer per_day.deinit();
    if (per_day.value.len == 0) {
        std.debug.print(
            "the organization has ingested events and its per-day counts are empty\n",
            .{},
        );
        return error.NothingCounted;
    }
    try decoded(out, "EventsPerDayEntry", per_day.value[0]);

    // An attempt and a response exist only once the output worker has finished a delivery. The
    // harness waited for one, in the application it caught the shared delivery from, and handed the
    // ids on — so this reads them back with the organization credential rather than waiting again.
    try exercised(out, "requestAttempts.read", request_attempts.read(
        allocator,
        seeded,
        null,
        null,
        null,
        null,
        null,
        null,
    ));

    const attempted = try read(out, "requestAttempts.get", request_attempts.get(
        allocator,
        try setting(environment, "HOOK0_REQUEST_ATTEMPT_ID"),
        seeded,
    ));
    defer attempted.deinit();
    try decoded(out, "RequestAttemptEvent", attempted.value.event);
    try decoded(out, "RequestAttemptSubscription", attempted.value.subscription);
    try decoded(out, "RequestAttemptStatusType", attempted.value.status.type);
    try decoded(out, "RequestAttemptStatus", attempted.value.status);
    try decoded(out, "RequestAttempt", attempted.value);

    const answered = try read(out, "response.get", responses.get(
        allocator,
        try setting(environment, "HOOK0_RESPONSE_ID"),
        seeded,
    ));
    defer answered.deinit();
    try decoded(out, "Response", answered.value);

    // Service tokens belong to the organization, so they are minted, read and revoked with the
    // organization credential. The one revoked below is the one minted here — never the one this
    // half of the flow is authenticating with.
    const issued = try read(out, "serviceToken.create", service_tokens.create(allocator, .{
        .name = "a token the zig smoke minted",
        .organization_id = organization,
    }));
    defer issued.deinit();
    try decoded(out, "ServiceToken", issued.value);

    try exercised(out, "serviceToken.list", service_tokens.list(allocator, organization));
    try exercised(
        out,
        "serviceToken.get",
        service_tokens.get(allocator, issued.value.token_id, organization),
    );
    try exercised(out, "serviceToken.edit", service_tokens.edit(
        allocator,
        issued.value.token_id,
        .{ .name = "a token the zig smoke renamed", .organization_id = organization },
    ));
    try exercised(
        out,
        "serviceToken.delete",
        service_tokens.delete(allocator, issued.value.token_id, organization),
    );

    // Destroyed in the order the instance can accept: the subscription that references the event
    // type, then the event type, then the application — which is last because the secret this whole
    // flow authenticates with stops authenticating the moment its application is gone.
    try exercised(out, "subscriptions.delete", subscriptions.delete(
        allocator,
        subscription.value.subscription_id,
        application,
    ));
    try exercised(out, "eventTypes.delete", event_types.delete(
        allocator,
        declared.value.event_type_name,
        application,
    ));
    try exercised(out, "applications.delete", applications.delete(allocator, application));
}

/// What every generated method is issued through, waiting out a paced instance.
///
/// Hook0 paces callers per credential, and a flow driving three dozen operations one after another
/// is exactly what that is for. The answer says the request was not processed and is safe to send
/// again after the delay it names, so this waits and sends it again rather than handing the caller
/// a problem that says nothing about the operation it was asking about.
///
/// It wraps the transport the package ships rather than replacing it: `deliver` is what that
/// transport offers a caller who needs what the answer carried beside its body, which is precisely
/// the delay.
const Paced = struct {
    inner: *hook0.Transport,
    io: std.Io,

    /// What the generated half issues its requests through.
    fn any(self: *Paced) hook0.runtime.Transport {
        return .{ .context = self, .requestFn = issue };
    }

    fn issue(
        context: *anyopaque,
        allocator: std.mem.Allocator,
        asked: hook0.runtime.Request,
    ) anyerror!hook0.runtime.Answer {
        const self: *Paced = @ptrCast(@alignCast(context));

        var sent: usize = 1;
        while (true) : (sent += 1) {
            const delivered = try self.inner.deliver(allocator, asked);
            if (delivered.status != too_many_requests or sent > paced_again) {
                return .{ .status = delivered.status, .payload = delivered.payload };
            }

            const waiting: i96 = @intCast(pause(delivered) * std.time.ns_per_ms);
            std.Io.sleep(self.io, .fromNanoseconds(waiting), .awake) catch {};
        }
    }
};

/// How long the answer says to wait, in milliseconds, held between a floor and a ceiling of this
/// smoke's own.
///
/// The floor is there because the header counts in whole seconds and the delay being waited out is
/// a fraction of one, so a truthful `Retry-After: 0` would otherwise mean sending the same request
/// again immediately, forever. The ceiling is there because a header is written by a server this
/// smoke does not control.
fn pause(delivered: hook0.transport.Transport.Delivered) u64 {
    const written = delivered.get("retry-after") orelse return shortest_pause_ms;
    const trimmed = std.mem.trim(u8, written, " \t");
    if (trimmed.len == 0 or trimmed.len > most_digits) return shortest_pause_ms;

    const seconds = std.fmt.parseInt(u32, trimmed, 10) catch return shortest_pause_ms;
    return std.math.clamp(
        @as(u64, seconds) * std.time.ms_per_s,
        shortest_pause_ms,
        longest_pause_ms,
    );
}

/// The problem the API named, when what was raised is one of the ones it can name.
///
/// Read off the table the generator writes rather than off the error's own name: the two agree
/// today, and only one of them is the document's.
fn problemOf(raised: anyerror) ?[]const u8 {
    for (hook0.errors.problems) |entry| {
        if (raised == @as(anyerror, entry.raised)) return entry.id;
    }
    return null;
}

/// What an operation answers, once the error it may raise instead is set aside.
fn Answered(comptime T: type) type {
    return @typeInfo(T).error_union.payload;
}

/// Reports one operation the flow goes on to use the answer of, which has to be a success.
fn read(
    out: *std.Io.Writer,
    operation: []const u8,
    answered: anytype,
) !Answered(@TypeOf(answered)) {
    const value = answered catch |raised| {
        out.flush() catch {};
        std.debug.print(
            "{s}: the flow needs what it answers, and it answered {t}\n",
            .{ operation, raised },
        );
        return raised;
    };

    try out.print("exercised {s} accepted\n", .{operation});
    try out.flush();
    return value;
}

/// Reports one operation driven for its own sake, whichever way the instance answered it.
///
/// A success and a problem are both complete round trips through the generated layer: the request
/// was composed, the instance answered, and this client read the answer. What is neither — the API
/// not reached, a body this client cannot read, a problem it does not know — stops the smoke,
/// because none of those say the client and the instance agree on anything.
fn exercised(out: *std.Io.Writer, operation: []const u8, answered: anytype) !void {
    if (answered) |value| {
        released(value);
        try out.print("exercised {s} accepted\n", .{operation});
    } else |raised| {
        const problem = problemOf(raised) orelse {
            out.flush() catch {};
            std.debug.print(
                "{s}: what came back names no problem this client knows: {t}\n",
                .{ operation, raised },
            );
            return raised;
        };
        try out.print("exercised {s} refused:{s}\n", .{ operation, problem });
    }
    try out.flush();
}

/// Frees what an operation answered, for the ones the flow drives for their own sake.
fn released(value: anytype) void {
    if (@TypeOf(value) == void) return;
    value.deinit();
}

/// Reports one generated model type as decoded out of a real answer.
///
/// The value is taken rather than only named, so the line cannot outlive what it is about: a field
/// that stops being part of an answer stops this compiling.
fn decoded(out: *std.Io.Writer, model: []const u8, value: anytype) !void {
    _ = value;
    try out.print("decoded {s}\n", .{model});
    try out.flush();
}

/// The instance without the path the hand-written half is built with.
///
/// The generated half composes paths that already carry `/api/v1`, since the API document's own
/// server URL is the bare origin. Handing this transport the whole of `HOOK0_API_URL` happens to
/// reach the same request, because a path of its own replaces the base's — but that is how one
/// language joins two URLs rather than a contract, and the TypeScript client was posting to
/// `/api/event` until the first live run found it. So this points at the origin, which is what the
/// contract says.
fn originOf(api_url: []const u8) ![]const u8 {
    const scheme = std.mem.indexOf(u8, api_url, "://") orelse return error.UnusableApiUrl;
    const authority = api_url[scheme + 3 ..];
    const path = std.mem.indexOfScalar(u8, authority, '/') orelse return api_url;
    if (path == 0) return error.UnusableApiUrl;

    return api_url[0 .. scheme + 3 + path];
}

/// The moment an event says it happened, as RFC 3339 spells one.
fn moment(io: std.Io, buffer: *[20]u8) ![]const u8 {
    const seconds: u64 = @intCast(@max(std.Io.Timestamp.now(io, .real).toSeconds(), 0));
    const epoch: std.time.epoch.EpochSeconds = .{ .secs = seconds };
    const year_day = epoch.getEpochDay().calculateYearDay();
    const month_day = year_day.calculateMonthDay();
    const time = epoch.getDaySeconds();

    return std.fmt.bufPrint(buffer, "{d:0>4}-{d:0>2}-{d:0>2}T{d:0>2}:{d:0>2}:{d:0>2}Z", .{
        year_day.year,
        month_day.month.numeric(),
        month_day.day_index + 1,
        time.getHoursIntoDay(),
        time.getMinutesIntoHour(),
        time.getSecondsIntoMinute(),
    });
}

/// The event both sends carry, under the identifier the caller names.
fn event(event_type: []const u8, event_id: ?[]const u8) hook0.Event {
    return .{
        .event_type = event_type,
        .payload = "{\"from\":\"the zig smoke\"}",
        .payload_content_type = "application/json",
        .labels = &.{.{ .key = "language", .value = language }},
        .event_id = event_id,
    };
}

/// Verifies what the output worker really delivered, with this client's own verification.
fn verify(allocator: std.mem.Allocator, io: std.Io, delivery: []const u8) !void {
    const signature = try part(allocator, io, delivery, "signature");
    defer allocator.free(signature);
    const secret = try part(allocator, io, delivery, "secret");
    defer allocator.free(secret);
    const body = try part(allocator, io, delivery, "body");
    defer allocator.free(body);
    const tolerance = try part(allocator, io, delivery, "tolerance");
    defer allocator.free(tolerance);
    const lines = try part(allocator, io, delivery, "headers");
    defer allocator.free(lines);

    var headers: std.ArrayList(hook0.signature.Header) = .empty;
    defer headers.deinit(allocator);
    var walking = std.mem.splitScalar(u8, lines, '\n');
    while (walking.next()) |line| {
        const at = std.mem.indexOf(u8, line, ": ") orelse continue;
        try headers.append(allocator, .{ .name = line[0..at], .value = line[at + 2 ..] });
    }

    try hook0.verifyWebhookSignature(
        io,
        std.mem.trim(u8, signature, " \t\r\n"),
        body,
        headers.items,
        std.mem.trim(u8, secret, " \t\r\n"),
        try std.fmt.parseInt(i64, std.mem.trim(u8, tolerance, " \t\r\n"), 10),
    );
}

/// One part of the delivery, as the harness wrote it down.
fn part(allocator: std.mem.Allocator, io: std.Io, delivery: []const u8, name: []const u8) ![]u8 {
    const path = try std.fs.path.join(allocator, &.{ delivery, name });
    defer allocator.free(path);
    return std.Io.Dir.cwd().readFileAlloc(io, path, allocator, .limited(max_part_bytes));
}

/// A setting the harness passes, or a refusal naming it: a smoke that ran without one would report
/// a failure of the client for something the harness never handed it.
fn setting(environment: *std.process.Environ.Map, name: []const u8) ![]const u8 {
    const value = environment.get(name) orelse return error.SettingNotSet;
    if (value.len == 0 or value.len > max_setting_bytes) return error.SettingNotSet;
    return value;
}
