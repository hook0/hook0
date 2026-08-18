// The C# client against a Hook0 that is really running.
//
// Two things happen here, and the second is the reason the first is worth having.
//
// The control: whether an application secret the API minted is accepted, whether a second send
// under an identifier already ingested is reported as the conflict it is, and whether a signature
// the output worker computed verifies. Those are the three questions no loopback suite can ask
// itself, because a suite that signs and verifies with the same assembly only proves the assembly
// agrees with itself.
//
// The surface: every operation the API document declares, driven through the generated layer
// against the same instance, and every model type it decodes out of a real answer.
// clients/csharp/tests already drives all of them — against an API the suite itself writes, out of
// the same document the client was generated from. That proves the client matches the document. It
// cannot prove the document matches Hook0, and a field the API really answers under another name
// passes there and fails on a consumer's first call.

using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Text.Json.Nodes;
using System.Threading;
using Hook0;
using Generated = Hook0.Generated;

namespace Hook0.Smoke;

internal static class Program
{
    /// <summary>The conflict the API answers a duplicated ingestion with.</summary>
    private const string AlreadyIngested = "EventAlreadyIngested";

    /// <summary>
    /// What this smoke labels everything it creates with, so that the subscription it makes and the
    /// event it sends find each other.
    /// </summary>
    private const string Language = "csharp";

    /// <summary>
    /// Where the subscription this smoke creates points. Nothing listens there, deliberately: what a
    /// delivery proves is proved once, by the webhook the harness catches and every language
    /// verifies.
    /// </summary>
    private const string Nowhere = "http://127.0.0.1:1/";

    /// <summary>What a paced instance answers.</summary>
    private const int TooManyRequests = 429;

    /// <summary>The most times one request is sent again after that answer.</summary>
    private const int PacedAgain = 8;

    /// <summary>The shortest this waits between two tries.</summary>
    private static readonly TimeSpan ShortestPause = TimeSpan.FromMilliseconds(200);

    /// <summary>The longest it waits, whatever the answer asked for.</summary>
    private static readonly TimeSpan LongestPause = TimeSpan.FromSeconds(10);

    private static int Main()
    {
        try
        {
            Smoke();
            return 0;
        }
        catch (Exception refused)
        {
            Console.Error.WriteLine(refused);
            return 1;
        }
    }

    private static void Smoke()
    {
        SendTwice();
        Surface();

        // Last, and on purpose: it needs no instance at all, so it still answers after the flow
        // above has deleted the application it was run against.
        Verify(Setting("HOOK0_DELIVERY"));
        Console.WriteLine("the signature the instance produced verifies");
    }

    /// <summary>The same event, twice, under the identifier the API minted for the first of them.</summary>
    private static void SendTwice()
    {
        string eventType = Setting("HOOK0_EVENT_TYPE");

        using Hook0Client client = new(
            apiUrl: Setting("HOOK0_API_URL"),
            applicationId: Setting("HOOK0_APPLICATION_ID"),
            token: Setting("HOOK0_TOKEN"));

        string sent = client.SendEvent(Sending(eventType, null));
        Console.WriteLine($"ingested {sent}");

        string said;
        try
        {
            client.SendEvent(Sending(eventType, Guid.Parse(sent)));
            throw new InvalidOperationException("sending the same event twice was accepted twice");
        }
        catch (SendException refused)
        {
            said = refused.Message;
        }

        if (!said.Contains(AlreadyIngested, StringComparison.Ordinal))
        {
            throw new InvalidOperationException(
                $"the second send failed without naming {AlreadyIngested}: {said}");
        }
        Console.WriteLine($"the second send reported {AlreadyIngested}");
    }

    /// <summary>
    /// Every operation the API document declares, driven against the instance in the order a
    /// consumer would: what it needs is created, read and listed, updated, and destroyed last.
    /// </summary>
    /// <remarks>
    /// Two credentials, because the API takes two and one of them cannot do everything. An
    /// application secret is scoped to the application it belongs to; what belongs to the
    /// organization — listing its applications, everything about service tokens, its per-day
    /// counts — needs the organization-scoped token beside it.
    /// </remarks>
    private static void Surface()
    {
        string origin = OriginOf(Setting("HOOK0_API_URL"));
        string application = Setting("HOOK0_APPLICATION_ID");
        Guid owning = Guid.Parse(application);
        string organization = Setting("HOOK0_ORGANIZATION_ID");
        Guid owner = Guid.Parse(organization);
        string seeded = Setting("HOOK0_SEEDED_APPLICATION_ID");
        IReadOnlyDictionary<string, string> labels =
            new Dictionary<string, string>(StringComparer.Ordinal) { ["language"] = Language };

        using Paced held = new(new HttpTransport(origin, Setting("HOOK0_TOKEN")));
        using Paced organizationWide = new(new HttpTransport(origin, Setting("HOOK0_SERVICE_TOKEN")));

        Generated.ApplicationsApi applications = new(held);
        Generated.ApplicationSecretsApi secrets = new(held);
        Generated.EventTypesApi eventTypes = new(held);
        Generated.SubscriptionsApi subscriptions = new(held);
        Generated.EventsApi events = new(held);
        Generated.EventsPerDayApi eventsPerDay = new(held);
        Generated.InstanceApi instance = new(held);
        Generated.QuotasApi quotas = new(held);
        Generated.PayloadContentTypesApi payloadContentTypes = new(held);
        Generated.ErrorsApi errorCatalogue = new(held);

        Generated.ApplicationsApi organizationApplications = new(organizationWide);
        Generated.EventsPerDayApi organizationEventsPerDay = new(organizationWide);
        Generated.RequestAttemptsApi requestAttempts = new(organizationWide);
        Generated.ResponseApi responses = new(organizationWide);
        Generated.ServiceTokenApi serviceTokens = new(organizationWide);

        // What the instance says about itself, which is what an application asks before it has
        // anything of its own: how it is configured, what it will let this account do, what a
        // payload may be, and every problem it can report.
        Decoded("InstanceConfig", Read("instance.get", instance.Get));

        Generated.QuotasResponse allowed = Read("quotas.get", quotas.Get);
        Decoded("QuotasResponseLimits", allowed.Limits);
        Decoded("QuotasResponse", allowed);

        Exercised("payload_content_types.list", () => payloadContentTypes.List());

        IReadOnlyList<Generated.Problem> catalogue = Read("errors.list", errorCatalogue.List);
        if (catalogue.Count == 0)
        {
            throw new InvalidOperationException(
                "the instance published an empty catalogue of the problems it can report");
        }

        DecodedOneOf("ProblemId", Generated.ProblemId.Contains, catalogue[0].Id);
        Decoded("Problem", catalogue[0]);

        // The application this smoke owns. One per language, so that the three deletions at the end
        // of this flow are real deletions rather than something eleven other smokes have to live
        // with.
        Generated.ApplicationInfo info = Read("applications.get", () => applications.Get(application));
        Decoded("ApplicationInfoConsumption", info.Consumption);
        Decoded("ApplicationInfoQuotas", info.Quotas);
        Decoded("ApplicationInfoOnboardingStepsEvent", info.OnboardingSteps.Event);
        Decoded("ApplicationInfoOnboardingStepsEventType", info.OnboardingSteps.EventType);
        Decoded("ApplicationInfoOnboardingStepsSubscription", info.OnboardingSteps.Subscription);
        Decoded("ApplicationInfoOnboardingSteps", info.OnboardingSteps);
        Decoded("ApplicationInfo", info);

        Decoded(
            "Application",
            Read(
                "applications.update",
                () => applications.Update(
                    application,
                    new Generated.ApplicationPost
                    {
                        Name = "the application the csharp smoke drives",
                        OrganizationId = owner,
                    })));

        // The organization's, so the organization credential. Listing what an account has is the
        // first thing a console does.
        Exercised("applications.list", () => organizationApplications.List(organization));

        // This one is driven with the *application* secret on purpose, and it is the flow's one
        // refusal. Creating an application is the organization's business and an application secret
        // is not the organization's, so the instance answers a problem document and this client
        // reads it — which is the half of the client that nothing else here would exercise.
        Exercised(
            "applications.create",
            () => applications.Create(new Generated.ApplicationPost
            {
                Name = "an application the csharp smoke's application secret may not create",
                OrganizationId = owner,
            }));

        // A second secret, so that the one this smoke is authenticating with is never the one it
        // revokes. Deleting that one succeeds and then locks the flow out of everything below.
        Generated.ApplicationSecret minted = Read(
            "applicationSecrets.create",
            () => secrets.Create(new Generated.ApplicationSecretPost
            {
                ApplicationId = owning,
                Name = "a secret the csharp smoke minted",
            }));
        Decoded("ApplicationSecret", minted);
        string mintedToken = minted.Token.ToString();

        Exercised("applicationSecrets.list", () => secrets.List(application));
        Exercised(
            "applicationSecrets.update",
            () => secrets.Update(mintedToken, new Generated.ApplicationSecretPost
            {
                ApplicationId = owning,
                Name = "a secret the csharp smoke renamed",
            }));
        Exercised("applicationSecrets.delete", () => secrets.Delete(mintedToken, application));

        // An event type of this smoke's own, rather than the one the harness declared: what is
        // created here is what is subscribed to, sent, replayed and deleted below.
        Generated.EventType declared = Read(
            "eventTypes.create",
            () => eventTypes.Create(new Generated.EventTypePost
            {
                ApplicationId = owning,
                ResourceType = "smoke",
                Service = Language,
                Verb = "ran",
            }));
        Decoded("EventType", declared);

        Exercised("eventTypes.get", () => eventTypes.Get(declared.EventTypeName, application));
        Exercised("eventTypes.list", () => eventTypes.List(application));

        Generated.SubscriptionPostTarget target = new()
        {
            Headers = new JsonObject(),
            Method = "POST",
            Type = "http",
            Url = Nowhere,
        };
        Generated.Subscription subscription = Read(
            "subscriptions.create",
            () => subscriptions.Create(new Generated.SubscriptionPost
            {
                ApplicationId = owning,
                EventTypes = [declared.EventTypeName],
                IsEnabled = true,
                Target = target,
                Description = "what the csharp smoke subscribes to its own events with",
                Labels = labels,
            }));
        Decoded("SubscriptionTarget", subscription.Target);
        Decoded("Subscription", subscription);
        string subscribed = subscription.SubscriptionId.ToString();

        Exercised("subscriptions.get", () => subscriptions.Get(subscribed));
        Exercised("subscriptions.list", () => subscriptions.List(application));
        Exercised(
            "subscriptions.update",
            () => subscriptions.Update(subscribed, new Generated.SubscriptionPost
            {
                ApplicationId = owning,
                EventTypes = [declared.EventTypeName],
                IsEnabled = true,
                Target = target,
                Description = "what the csharp smoke renamed it to",
                Labels = labels,
            }));

        // The event the subscription above selects, sent through the generated layer rather than
        // through SendEvent: the hand-written half has its own three questions above, and this is
        // the operation the document declares.
        Generated.IngestedEvent ingested = Read(
            "events.ingest",
            () => events.Ingest(new Generated.EventPost
            {
                ApplicationId = owning,
                EventType = declared.EventTypeName,
                Labels = labels,
                OccurredAt = DateTimeOffset.UtcNow,
                Payload = """{"from":"the csharp smoke"}""",
                PayloadContentType = "application/json",
                EventId = Hook0Client.NewEventId(),
            }));
        Decoded("IngestedEvent", ingested);
        string sent = ingested.EventId.ToString();

        Decoded("EventWithPayload", Read("events.get", () => events.Get(sent, application)));

        IReadOnlyList<Generated.Event> listed = Read("events.list", () => events.List(application));
        if (listed.Count == 0)
        {
            throw new InvalidOperationException("the instance ingested an event and then listed none");
        }
        Decoded("Event", listed[0]);

        Exercised(
            "events.replay",
            () => events.Replay(sent, new Generated.ReplayEvent { ApplicationId = owning }));

        // This application was created a moment ago and the counts come out of a view the instance
        // refreshes on a cycle of its own, so this answers a list with nothing in it — which is an
        // answer, and one a client has to be able to read.
        Exercised(
            "events_per_day.list_for_application",
            () => eventsPerDay.ListForApplication(application));

        // The organization's counts do have something in them: the harness waited for the instance
        // to refresh them before running any of this, precisely so that the type they are answered
        // with is one a client decodes rather than one nothing ever produces.
        IReadOnlyList<Generated.EventsPerDayEntry> perDay = Read(
            "events_per_day.list_for_organization",
            () => organizationEventsPerDay.ListForOrganization(organization));
        if (perDay.Count == 0)
        {
            throw new InvalidOperationException(
                "the organization has ingested events and its per-day counts are empty");
        }
        Decoded("EventsPerDayEntry", perDay[0]);

        // An attempt and a response exist only once the output worker has finished a delivery. The
        // harness waited for one, in the application it caught the shared delivery from, and handed
        // the ids on — so this reads them back with the organization credential rather than waiting
        // again.
        Exercised("requestAttempts.list", () => requestAttempts.List(seeded));

        Generated.RequestAttempt attempted = Read(
            "requestAttempts.get",
            () => requestAttempts.Get(Setting("HOOK0_REQUEST_ATTEMPT_ID"), seeded));
        Decoded("RequestAttemptEvent", attempted.Event);
        Decoded("RequestAttemptSubscription", attempted.Subscription);
        DecodedOneOf(
            "RequestAttemptStatusType", Generated.RequestAttemptStatusType.Contains, attempted.Status.Type);
        Decoded("RequestAttemptStatus", attempted.Status);
        Decoded("RequestAttempt", attempted);

        Decoded("Response", Read("response.get", () => responses.Get(Setting("HOOK0_RESPONSE_ID"), seeded)));

        // Service tokens belong to the organization, so they are minted, read and revoked with the
        // organization credential. The one revoked below is the one minted here — never the one
        // this half of the flow is authenticating with.
        Generated.ServiceToken issued = Read(
            "serviceToken.create",
            () => serviceTokens.Create(new Generated.ServiceTokenPost
            {
                Name = "a token the csharp smoke minted",
                OrganizationId = owner,
            }));
        Decoded("ServiceToken", issued);
        string issuedId = issued.TokenId.ToString();

        Exercised("serviceToken.list", () => serviceTokens.List(organization));
        Exercised("serviceToken.get", () => serviceTokens.Get(issuedId, organization));
        Exercised(
            "serviceToken.update",
            () => serviceTokens.Update(issuedId, new Generated.ServiceTokenPost
            {
                Name = "a token the csharp smoke renamed",
                OrganizationId = owner,
            }));
        Exercised("serviceToken.delete", () => serviceTokens.Delete(issuedId, organization));

        // Destroyed in the order the instance can accept: the subscription that references the
        // event type, then the event type, then the application — which is last because the secret
        // this whole flow authenticates with stops authenticating the moment its application is
        // gone.
        Exercised("subscriptions.delete", () => subscriptions.Delete(subscribed, application));
        Exercised("eventTypes.delete", () => eventTypes.Delete(declared.EventTypeName, application));
        Exercised("applications.delete", () => applications.Delete(application));
    }

    /// <summary>Reports one operation the flow goes on to use the answer of, which has to be a success.</summary>
    private static T Read<T>(string operation, Func<T> asking)
    {
        T answered;
        try
        {
            answered = asking();
        }
        catch (Exception failed)
        {
            throw new InvalidOperationException(
                $"{operation}: the flow needs what it answers, and it answered {failed.Message}", failed);
        }

        Console.WriteLine($"exercised {operation} accepted");
        return answered;
    }

    /// <summary>
    /// Reports one operation driven for its own sake, whichever way the instance answered it.
    /// </summary>
    /// <remarks>
    /// A success and a problem are both complete round trips through the generated layer: the
    /// request was composed, the instance answered, and this client read the answer. What is
    /// neither — the API not reached, a body this client cannot read, a problem it does not know —
    /// stops the smoke, because none of those say the client and the instance agree on anything.
    /// </remarks>
    private static void Exercised(string operation, Action asking)
    {
        try
        {
            asking();
        }
        catch (Generated.ProblemException refused)
        {
            Generated.Problem? problem = refused.Problem;
            if (problem is null)
            {
                throw new InvalidOperationException(
                    $"{operation}: what came back names no problem this client knows: {refused.Message}", refused);
            }

            Console.WriteLine($"exercised {operation} refused:{problem.Id}");
            return;
        }
        catch (Exception failed)
        {
            throw new InvalidOperationException($"{operation}: {failed.Message}", failed);
        }

        Console.WriteLine($"exercised {operation} accepted");
    }

    /// <summary>Reports one generated model type as decoded out of a real answer.</summary>
    /// <remarks>
    /// The value is taken rather than only named, so the line cannot outlive what it is about: a
    /// field that stops being part of an answer stops compiling here.
    /// </remarks>
    private static void Decoded<T>(string model, T value) => Console.WriteLine($"decoded {model}");

    /// <summary>
    /// Reports one of the document's closed lists of strings as decoded, and holds the value to the
    /// list.
    /// </summary>
    /// <remarks>
    /// The other clients write such a list as a type of their own, so a value outside it fails to
    /// decode and the <c>decoded</c> line means the reader refused everything else. This one writes
    /// it as constants beside a field typed as the string itself, so the same line would pass on any
    /// string at all — including the one an instance answering a value this client has never heard
    /// of would send. <c>Contains</c> is what the generated class offers for exactly that question,
    /// and asking it here is what makes this language's line say what the other six say.
    /// </remarks>
    private static void DecodedOneOf(string model, Func<string, bool> declares, string value)
    {
        if (!declares(value))
        {
            throw new InvalidOperationException(
                $"the API answered `{value}`, and {model} declares no such value");
        }

        Decoded(model, value);
    }

    /// <summary>The instance without the path the hand-written half is built with.</summary>
    /// <remarks>
    /// The generated half composes paths that already carry <c>/api/v1</c>, since the API document's
    /// own server URL is the bare origin. Handing this transport the whole of <c>HOOK0_API_URL</c>
    /// happens to reach the same request: <see cref="Uri"/> resolution lets an absolute path replace
    /// the base's, as RFC 3986 says, and what the base carried is discarded whichever of the two it
    /// was given. That is how one language joins two URLs rather than a contract — the TypeScript
    /// client resolves with <c>new URL</c> and was posting to <c>/api/event</c> until the first live
    /// run found it — so this points at the origin, which is what the contract says.
    /// </remarks>
    private static string OriginOf(string apiUrl)
    {
        Uri parsed = new(apiUrl, UriKind.Absolute);
        return $"{parsed.Scheme}://{parsed.Authority}";
    }

    /// <summary>The event both sends carry, under the identifier the caller names.</summary>
    private static Event Sending(string eventType, Guid? eventId) => new()
    {
        EventType = eventType,
        Payload = """{"from":"the csharp smoke"}""",
        PayloadContentType = "application/json",
        Labels = new Dictionary<string, string>(StringComparer.Ordinal) { ["language"] = Language },
        EventId = eventId,
    };

    /// <summary>
    /// Verifies what the output worker really delivered, with this client's own verification.
    /// </summary>
    private static void Verify(string delivery)
    {
        string Read(string part) => File.ReadAllText(Path.Combine(delivery, part));

        List<KeyValuePair<string, string>> headers = new();
        foreach (string line in Read("headers").Split('\n'))
        {
            int at = line.IndexOf(": ", StringComparison.Ordinal);
            if (at > 0)
            {
                headers.Add(new KeyValuePair<string, string>(line[..at], line[(at + 2)..]));
            }
        }

        Webhooks.VerifyWebhookSignature(
            signature: Read("signature").Trim(),
            payload: File.ReadAllBytes(Path.Combine(delivery, "body")),
            headers: headers,
            subscriptionSecret: Read("secret").Trim(),
            tolerance: TimeSpan.FromSeconds(int.Parse(Read("tolerance").Trim(),
                CultureInfo.InvariantCulture)));
    }

    /// <summary>A setting the harness passes, or a refusal naming it.</summary>
    private static string Setting(string name)
    {
        string? value = Environment.GetEnvironmentVariable(name);
        if (string.IsNullOrEmpty(value))
        {
            throw new InvalidOperationException($"{name} is not set");
        }
        return value;
    }

    /// <summary>
    /// How long the answer says to wait, held between a floor and a ceiling of this smoke's own.
    /// </summary>
    /// <remarks>
    /// The floor is there because the header counts in whole seconds and the delay being waited out
    /// is a fraction of one, so a truthful <c>Retry-After: 0</c> would otherwise mean sending the
    /// same request again immediately, forever. The ceiling is there because a header is written by
    /// a server this smoke does not control.
    /// </remarks>
    private static TimeSpan Pause(TransportDelivery delivered)
    {
        TimeSpan asked = ShortestPause;
        if (delivered.Headers.TryGetValue("retry-after", out string? written)
            && int.TryParse(written.Trim(), NumberStyles.None, CultureInfo.InvariantCulture, out int seconds))
        {
            asked = TimeSpan.FromSeconds(seconds);
        }

        return asked < ShortestPause ? ShortestPause : asked > LongestPause ? LongestPause : asked;
    }

    /// <summary>What every generated method is issued through, waiting out a paced instance.</summary>
    /// <remarks>
    /// Hook0 paces callers per credential, and a flow driving three dozen operations one after
    /// another is exactly what that is for. The answer says the request was not processed and is
    /// safe to send again after the delay it names, so this waits and sends it again rather than
    /// handing the caller a problem that says nothing about the operation it was asking about.
    /// <para>
    /// It wraps the transport the assembly ships rather than replacing it: <c>Deliver</c> is what
    /// that transport offers a caller who needs what the answer carried beside its body, which is
    /// precisely the delay.
    /// </para>
    /// </remarks>
    private sealed class Paced(HttpTransport inner) : ITransport, IDisposable
    {
        public TransportAnswer Request(
            string method,
            string path,
            IReadOnlyList<KeyValuePair<string, string>> query,
            object? body)
        {
            for (int sent = 1; ; sent++)
            {
                TransportDelivery delivered = inner.Deliver(method, path, query, body);
                if (delivered.Status != TooManyRequests || sent > PacedAgain)
                {
                    return new TransportAnswer(delivered.Status, delivered.Payload);
                }

                Thread.Sleep(Pause(delivered));
            }
        }

        public void Dispose() => inner.Dispose();
    }
}
