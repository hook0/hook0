// What a send does, over a real socket, on each surface this client carries.
//
// Everything the shared corpus dictates is checked by `ConformanceTests`. What is here is what the
// corpus does not state and this client still has to get right: that a success costs one request,
// that a repeated request carries the identifier the first one carried, that a schedule stops where
// its policy says, and that a payload nobody could send is refused before a socket is opened.

using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Text.Json.Nodes;
using System.Threading.Tasks;
using Xunit;

namespace Hook0.Tests;

/// <summary>What one send does when the API answers each way it can.</summary>
public sealed class ClientTests : ApiCase
{
    private const string IngestedId = "01961234-5678-7abc-8def-0123456789ac";

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task ASendTheApiTakesStraightAwayIssuesOneRequest(Surface surface)
    {
        Api.WillAnswer(Ingested(IngestedId));
        using Hook0Client client = Client();

        string answered = await Send(client, AnEvent(), surface);

        Assert.Equal(IngestedId, answered);
        Assert.Single(Api.Received);
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task ARepeatedRequestCarriesTheIdentifierTheFirstOneCarried(Surface surface)
    {
        // The whole reason a send is safe to repeat: the API keys the event on an identifier this
        // client chose, so the second attempt asks for the same event rather than a second one.
        Api.WillAnswer(ServerError(), Ingested(IngestedId));
        using Hook0Client client = Client();

        await Send(client, AnEvent(), surface);

        Assert.Equal(2, Api.Received.Count);
        Assert.Equal(Sent(0, "event_id"), Sent(1, "event_id"));
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task ATransportFailureThenASuccessCarriesTheSameIdentifier(Surface surface)
    {
        // An attempt that got no answer says nothing about whether the API acted on it, which is
        // exactly the case the client-chosen identifier exists for.
        Api.WillAnswer(
            Ingested(IngestedId) with { HeldFor = TimeSpan.FromSeconds(1) },
            Ingested(IngestedId));
        using Hook0Client client = Client(Options(maxAttempts: 4, requestTimeout: 0.2));

        string answered = await Send(client, AnEvent(), surface);

        Assert.Equal(IngestedId, answered);
        Assert.Equal(2, Api.Received.Count);
        Assert.Equal(Sent(0, "event_id"), Sent(1, "event_id"));
    }

    [Theory]
    [InlineData(Surface.Blocking, 1)]
    [InlineData(Surface.Blocking, 2)]
    [InlineData(Surface.Blocking, 4)]
    [InlineData(Surface.Awaiting, 1)]
    [InlineData(Surface.Awaiting, 2)]
    [InlineData(Surface.Awaiting, 4)]
    public async Task RepeatedServerErrorsStopAtTheAttemptBound(Surface surface, int maxAttempts)
    {
        Api.WillAnswer(ServerError(), ServerError(), ServerError(), ServerError(), ServerError());
        using Hook0Client client = Client(Options(maxAttempts: maxAttempts));

        SendException gave = await Assert.ThrowsAsync<SendException>(
            () => Send(client, AnEvent(), surface));

        Assert.Equal(maxAttempts, Api.Received.Count);
        Assert.Equal(maxAttempts, gave.Attempts);
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task ARefusalTheApiWouldRepeatIssuesExactlyOneRequest(Surface surface)
    {
        Api.WillAnswer(Refusal(400, "EventInvalidJsonPayload"), Ingested(IngestedId));
        using Hook0Client client = Client();

        SendException gave = await Assert.ThrowsAsync<SendException>(
            () => Send(client, AnEvent(), surface));

        Assert.Single(Api.Received);
        Assert.Equal(1, gave.Attempts);
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task AnAlreadyIngestedEventIsASuccessOnARetry(Surface surface)
    {
        // The identifier is this send's own, so the API saying it is taken on a *repeated* request
        // says the earlier attempt landed.
        Api.WillAnswer(ServerError(), AlreadyIngested());
        Guid keyed = Guid.Parse(IngestedId);
        using Hook0Client client = Client();

        string answered = await Send(client, AnEvent(eventId: keyed), surface);

        Assert.Equal(IngestedId, answered);
        Assert.Equal(2, Api.Received.Count);
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task AnAlreadyIngestedEventIsAFailureOnAFirstAttempt(Surface surface)
    {
        // Nothing this send did can have taken that identifier, so it is a genuine conflict.
        Api.WillAnswer(AlreadyIngested());
        using Hook0Client client = Client();

        SendException gave = await Assert.ThrowsAsync<SendException>(
            () => Send(client, AnEvent(eventId: Guid.Parse(IngestedId)), surface));

        Assert.Single(Api.Received);
        Assert.Equal(1, gave.Attempts);
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task RetryingCanBeTurnedOff(Surface surface)
    {
        Api.WillAnswer(ServerError(), Ingested(IngestedId));
        using Hook0Client client = Client(new ClientOptions { RetryPolicy = RetryPolicy.Disabled });

        await Assert.ThrowsAsync<SendException>(() => Send(client, AnEvent(), surface));

        Assert.Single(Api.Received);
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task AnOversizedPayloadIsRefusedBeforeAnyRequest(Surface surface)
    {
        Api.WillAnswer(Ingested(IngestedId));
        using Hook0Client client = Client(Options() with { MaxPayloadBytes = 32 });

        SendException gave = await Assert.ThrowsAsync<SendException>(
            () => Send(client, AnEvent(payload: new string('x', 64)), surface));

        Assert.Empty(Api.Received);
        Assert.Equal(0, gave.Attempts);
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task AnAnswerCarryingMoreHeaderLinesThanAreReadIsRefused(Surface surface)
    {
        // Nothing in the framework counts header lines, so this bound is the client's own and has to
        // be provoked over a socket that can actually write that many of them.
        List<KeyValuePair<string, string>> many = [];
        for (int line = 0; line < 8; line++)
        {
            many.Add(new KeyValuePair<string, string>($"x-padding-{line}", "y"));
        }

        Api.WillAnswer(Ingested(IngestedId) with { Headers = many }, Ingested(IngestedId));
        using Hook0Client client = Client(Options() with { MaxResponseHeaders = 4 });

        SendException gave = await Assert.ThrowsAsync<SendException>(
            () => Send(client, AnEvent(), surface));

        // Refused rather than repeated: the same request draws the same oversized head.
        Assert.Single(Api.Received);
        Assert.Equal(1, gave.Attempts);
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task AnAnswerWhoseHeadIsWellAboveTheCeilingIsRefused(Surface surface)
    {
        // Well above rather than just above: whether a head near the ceiling is read is settled by
        // the runtime before this client is reached, and a case built there reports the runtime.
        List<KeyValuePair<string, string>> heavy = [];
        for (int line = 0; line < 16; line++)
        {
            heavy.Add(new KeyValuePair<string, string>($"x-padding-{line}", new string('y', 4096)));
        }

        Api.WillAnswer(Ingested(IngestedId) with { Headers = heavy }, Ingested(IngestedId));
        using Hook0Client client = Client(Options() with { MaxResponseHeaders = 64 });

        SendException gave = await Assert.ThrowsAsync<SendException>(
            () => Send(client, AnEvent(), surface));

        Assert.Single(Api.Received);
        Assert.Equal(1, gave.Attempts);
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task AnEventTypeTheApplicationAlreadyDeclaresIsNotDeclaredAgain(Surface surface)
    {
        Api.WillAnswer(new ScriptedResponse(
            200,
            new JsonArray(new JsonObject { ["event_type_name"] = "auth.user.create" })));
        using Hook0Client client = Client();

        IReadOnlyList<string> created = await Upserted(
            client,
            ["auth.user.create"],
            surface);

        Assert.Empty(created);
        Assert.Single(Api.Received);
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task AnEventTypeTheApplicationDoesNotDeclareIsCreated(Surface surface)
    {
        Api.WillAnswer(
            new ScriptedResponse(200, new JsonArray()),
            new ScriptedResponse(201, new JsonObject { ["event_type_name"] = "auth.user.create" }));
        using Hook0Client client = Client();

        IReadOnlyList<string> created = await Upserted(client, ["auth.user.create"], surface);

        Assert.Equal(["auth.user.create"], created);
        Assert.Equal(2, Api.Received.Count);
        Assert.Equal("POST", Api.Received[1].Verb);
    }

    [Fact]
    public async Task APolicyHoldingNumbersItsHeaderCannotStateIsStillStatedOnTheWire()
    {
        // Three numbers a caller should not write and can: more attempts than the policy will ever
        // make, a delay as long as a duration goes, and a negative one. What reaches the socket is
        // the schedule the policy would actually apply, rather than a header rounded through a
        // double or an exception raised while it was being composed.
        Api.WillAnswer(Ingested(IngestedId));
        using Hook0Client client = Client(new ClientOptions
        {
            RetryPolicy = new RetryPolicy
            {
                MaxAttempts = 1000,
                InitialBackoff = TimeSpan.MaxValue,
                MaxBackoff = TimeSpan.FromMilliseconds(-5),
                MaxTotalDelay = TimeSpan.MaxValue,
            },
            RequestTimeout = TimeSpan.FromSeconds(5),
        });

        await client.SendEventAsync(AnEvent());

        long longest = TimeSpan.MaxValue.Ticks / TimeSpan.TicksPerMillisecond;
        KeyValuePair<string, string> stated = Assert.Single(
            Api.Received[0].Headers,
            header => string.Equals(
                header.Key,
                "Hook0-Client-Options",
                StringComparison.OrdinalIgnoreCase));

        Assert.Equal(
            FormattableString.Invariant(
                $"attempts={RetryPolicy.MaxAttemptsCap},backoff={longest},ceiling=0,budget={longest}"),
            stated.Value);

        // The schedule reads those same numbers, so it has to survive them too: a header stating a
        // budget the scheduler raises on rather than waits describes a send that dies. What it
        // holds to is what the header states — a ceiling of nothing, before each of its attempts.
        IReadOnlyList<TimeSpan> waited = client.Options.RetryPolicy.Delays([1.0, 1.0, 1.0]);
        Assert.Equal(RetryPolicy.MaxAttemptsCap - 1, waited.Count);
        Assert.All(waited, one => Assert.Equal(TimeSpan.Zero, one));
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task ABudgetTooLargeToCountStillCutsDownTheDelayTheApiNames(Surface surface)
    {
        // The retry loop reads the budget a third time, to cut an API-named delay down to what is
        // left of it. That read is the one furthest from the schedule and the header, and it read
        // the field raw — where a TimeSpan counts ticks it survives, and where a sibling SDK counts
        // milliseconds the same read raised and killed the send between two attempts.
        Api.WillAnswer(
            Refusal(503, "ServiceUnavailable", [new KeyValuePair<string, string>("Retry-After", "0")]),
            Ingested(IngestedId));

        using Hook0Client client = Client(new ClientOptions
        {
            RetryPolicy = new RetryPolicy
            {
                MaxAttempts = 2,
                InitialBackoff = PromptBackoff,
                MaxBackoff = PromptBackoff,
                MaxTotalDelay = TimeSpan.MaxValue,
            },
            RequestTimeout = TimeSpan.FromSeconds(5),
        });

        Assert.Equal(IngestedId, await Send(client, AnEvent(), surface));
        Assert.Equal(2, Api.Received.Count);
    }

    [Fact]
    public void ADurationThisClientHoldsCannotBeNonFinite()
    {
        // The shared rule reads a non-finite delay as the default of the field it was written in.
        // There is nothing to read here: a TimeSpan counts ticks, and the conversions that could
        // carry an infinity or a NaN into one refuse before a policy can hold it. The rule is
        // satisfied by the type rather than by a branch, and this is what says so — a branch for it
        // would be one no input can reach and no test can cover.
        Assert.Throws<OverflowException>(() => TimeSpan.FromMilliseconds(double.PositiveInfinity));
        Assert.Throws<OverflowException>(() => TimeSpan.FromMilliseconds(double.NegativeInfinity));
        Assert.Throws<ArgumentException>(() => TimeSpan.FromMilliseconds(double.NaN));
    }

    [Fact]
    public async Task WhatTheHeaderStatesIsWhatTheScheduleWaits()
    {
        // The header is worth nothing if it describes a schedule the client does not keep. Every
        // delay is drawn against the whole of its ceiling, which is the drawn schedule at its
        // longest, and the numbers the wire carries are held against it rather than against the
        // policy they were both read from.
        RetryPolicy policy = new()
        {
            MaxAttempts = 3,
            InitialBackoff = TimeSpan.FromMilliseconds(200),
            MaxBackoff = TimeSpan.FromMilliseconds(400),
            MaxTotalDelay = TimeSpan.FromSeconds(5),
        };

        Api.WillAnswer(Ingested(IngestedId));
        using Hook0Client client = Client(new ClientOptions
        {
            RetryPolicy = policy,
            RequestTimeout = TimeSpan.FromSeconds(5),
        });

        await client.SendEventAsync(AnEvent());

        Dictionary<string, long> stated = Assert
            .Single(
                Api.Received[0].Headers,
                header => string.Equals(
                    header.Key,
                    "Hook0-Client-Options",
                    StringComparison.OrdinalIgnoreCase))
            .Value
            .Split(',')
            .ToDictionary(
                part => part[..part.IndexOf('=', StringComparison.Ordinal)],
                part => long.Parse(
                    part[(part.IndexOf('=', StringComparison.Ordinal) + 1)..],
                    CultureInfo.InvariantCulture),
                StringComparer.Ordinal);

        IReadOnlyList<TimeSpan> longest = policy.Delays([1.0, 1.0]);

        Assert.Equal(
            stated["backoff"],
            (long)policy.BackoffCeiling(1).TotalMilliseconds);
        Assert.All(
            longest,
            one => Assert.True(
                (long)one.TotalMilliseconds <= stated["ceiling"],
                $"the schedule waits {one.TotalMilliseconds}ms where the header states a ceiling " +
                $"of {stated["ceiling"]}"));

        long spent = longest.Sum(one => (long)one.TotalMilliseconds);
        Assert.True(
            spent <= stated["budget"],
            $"the schedule spends {spent}ms where the header states a budget of {stated["budget"]}");
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task AnEventSaysWhatElseItCarriesAndWhenItHappened(Surface surface)
    {
        Api.WillAnswer(Ingested(IngestedId));
        using Hook0Client client = Client();

        Event carrying = AnEvent() with
        {
            Metadata = new Dictionary<string, string>(StringComparer.Ordinal) { ["tenant"] = "acme" },
            OccurredAt = DateTimeOffset.Parse("2026-01-02T03:04:05Z", CultureInfo.InvariantCulture),
        };
        await Send(client, carrying, surface);

        JsonNode sent = Api.Received[0].Json();

        Assert.Equal("acme", sent["metadata"]!["tenant"]!.GetValue<string>());

        // The moment the caller named travels, rather than the one the send was made at. It is read
        // back rather than matched as text: this client writes a zero offset as `+00:00` where the
        // JVM clients write `Z`, and both are the same moment to anything reading RFC 3339.
        Assert.Equal(
            DateTimeOffset.Parse("2026-01-02T03:04:05Z", CultureInfo.InvariantCulture),
            DateTimeOffset.Parse(Text(sent, "occurred_at"), CultureInfo.InvariantCulture));
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task ASendThatGaveUpSaysWhichEventItWasAndHowLongItWaited(Surface surface)
    {
        // What a caller catches has to say enough to act on: which event never landed, how many
        // requests were spent on it, and how much of the wait was this client's own doing.
        Api.WillAnswer(ServerError(), ServerError(), ServerError());
        using Hook0Client client = Client(Options(maxAttempts: 3));

        SendException gave = await Assert.ThrowsAsync<SendException>(
            () => Send(client, AnEvent(new Guid(IngestedId)), surface));

        Assert.Equal(IngestedId, gave.EventId);
        Assert.Equal(3, gave.Attempts);
        Assert.True(gave.Waited > TimeSpan.Zero, "a send that retried twice says it waited for nothing");
        Assert.True(gave.Waited <= TimeSpan.FromSeconds(1), $"a send waited {gave.Waited} of a one-second budget");
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task AListOfEventTypesTheApiRefusesToAnswerIsReported(Surface surface)
    {
        Api.WillAnswer(ServerError());
        using Hook0Client client = Client();

        EventTypeException reported = await Assert.ThrowsAsync<EventTypeException>(
            () => Upserted(client, ["auth.user.create"], surface));

        Assert.StartsWith("Getting available event types failed", reported.Message, StringComparison.Ordinal);
        // Nothing was created, since what is already declared was never learnt.
        Assert.Single(Api.Received);
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task AnAnswerThatIsNotAListOfEventTypesIsReported(Surface surface)
    {
        Api.WillAnswer(new ScriptedResponse(200, new JsonObject { ["event_type_name"] = "auth.user.create" }));
        using Hook0Client client = Client();

        EventTypeException reported = await Assert.ThrowsAsync<EventTypeException>(
            () => Upserted(client, ["auth.user.create"], surface));

        Assert.Contains("did not answer a list of event types", reported.Message, StringComparison.Ordinal);
        Assert.Single(Api.Received);
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task AnAnswerToAListOfEventTypesThatIsNotJsonAtAllIsReported(Surface surface)
    {
        Api.WillAnswer(new ScriptedResponse(200, null, Written: "<html>a proxy answered this</html>"));
        using Hook0Client client = Client();

        EventTypeException reported = await Assert.ThrowsAsync<EventTypeException>(
            () => Upserted(client, ["auth.user.create"], surface));

        Assert.StartsWith("Getting available event types failed", reported.Message, StringComparison.Ordinal);
        Assert.Single(Api.Received);
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task AListOfEventTypesTheApiNeverAnswersIsReported(Surface surface)
    {
        // The request got no answer at all rather than one that could not be read, and a caller has
        // to hear about it as the same failure whichever way the call was made.
        Api.WillAnswer(new ScriptedResponse(200, new JsonArray(), TimeSpan.FromSeconds(1)));
        using Hook0Client client = Client(Options(requestTimeout: 0.2));

        EventTypeException reported = await Assert.ThrowsAsync<EventTypeException>(
            () => Upserted(client, ["auth.user.create"], surface));

        Assert.StartsWith("Getting available event types failed", reported.Message, StringComparison.Ordinal);
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task AskingForNoEventTypesDeclaresNoneAndIssuesNoRequest(Surface surface)
    {
        using Hook0Client client = Client();

        Assert.Empty(await Upserted(client, [], surface));
        Assert.Empty(Api.Received);
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task MoreEventTypesThanOneCallDeclaresIsRefusedBeforeAnyRequest(Surface surface)
    {
        string[] many = [.. Enumerable.Range(0, 257).Select(at => $"auth.user.create{at}")];
        using Hook0Client client = Client();

        EventTypeException refused = await Assert.ThrowsAsync<EventTypeException>(
            () => Upserted(client, many, surface));

        Assert.Contains("above the", refused.Message, StringComparison.Ordinal);
        Assert.Empty(Api.Received);
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task AnEventTypeTheApiRefusesToCreateIsReported(Surface surface)
    {
        Api.WillAnswer(new ScriptedResponse(200, new JsonArray()), Refusal(409, "EventTypeAlreadyExist"));
        using Hook0Client client = Client();

        EventTypeException reported = await Assert.ThrowsAsync<EventTypeException>(
            () => Upserted(client, ["auth.user.create"], surface));

        Assert.StartsWith(
            "Creating event type 'auth.user.create' failed",
            reported.Message,
            StringComparison.Ordinal);
        Assert.Equal(2, Api.Received.Count);
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task AnAnswerTakingTheEventWithoutSayingWhichIsNotReadAsASuccess(Surface surface)
    {
        // Two ways an answer can take the event and not say which: a document carrying no
        // `event_id`, and a body that is not a document at all.
        foreach (ScriptedResponse taken in new[]
        {
            new ScriptedResponse(201, new JsonObject { ["application_id"] = "app-123" }),
            new ScriptedResponse(201, null, Written: "<html>a proxy answered this</html>"),
        })
        {
            Restarted();
            Api.WillAnswer(taken, Ingested(IngestedId));
            using Hook0Client client = Client();

            SendException reported = await Assert.ThrowsAsync<SendException>(
                () => Send(client, AnEvent(), surface));

            Assert.Contains("without an event id", reported.Message, StringComparison.Ordinal);
            // Repeating the request would meet the same answer, so the send stops rather than
            // spending every attempt it had left on it.
            Assert.Single(Api.Received);
        }
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task AnAnswerCarryingNoDocumentAtAllIsStillReported(Surface surface)
    {
        Api.WillAnswer(new ScriptedResponse(500, null, Written: "<html>a proxy answered this</html>"));
        using Hook0Client client = Client(Options(maxAttempts: 1));

        SendException reported = await Assert.ThrowsAsync<SendException>(
            () => Send(client, AnEvent(), surface));

        Assert.Contains("a proxy answered this", reported.Message, StringComparison.Ordinal);
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task ADelayTheApiNamesInAnythingButSecondsLeavesTheScheduleInPlace(Surface surface)
    {
        // A `Retry-After` carrying a date is a clock this client would be comparing against its own;
        // one carrying nothing, or more than a delay is ever written in, is a header nobody meant.
        // None of them stops the send: it comes back on its own schedule rather than on one it read
        // out of a spelling it does not understand.
        foreach (string written in new[]
        {
            string.Empty,
            "Wed, 21 Oct 2026 07:28:00 GMT",
            new string('0', 64),
            "9999999999",
        })
        {
            Restarted();
            Api.WillAnswer(
                Refusal(503, "ServiceUnavailable", [new KeyValuePair<string, string>("Retry-After", written)]),
                Ingested(IngestedId));
            using Hook0Client client = Client(Options(maxAttempts: 2));

            Assert.Equal(IngestedId, await Send(client, AnEvent(), surface));
            Assert.Equal(2, Api.Received.Count);
        }
    }

    [Fact]
    public void AFailureThisClientMetSaysWhatWasReportedUnderneathIt()
    {
        InvalidOperationException underneath = new("what the socket said");
        Hook0Exception reported = new("what this client says", underneath);

        Assert.Equal("what this client says", reported.Message);
        Assert.Same(underneath, reported.InnerException);
    }

    [Fact]
    public void AnEventTypeThatDoesNotNameAllThreeOfItsPartsIsRefused()
    {
        Assert.Throws<EventTypeException>(() => EventType.Parse("auth.user"));
        Assert.Throws<EventTypeException>(() => EventType.Parse("auth..create"));
        Assert.Throws<EventTypeException>(() => EventType.Parse("auth.user.create.extra"));
        Assert.Equal("auth.user.create", EventType.Parse("auth.user.create").Written());
    }

    [Fact]
    public void AnEventTypeNobodyCouldHaveMeantIsRefusedRatherThanSpelledOut()
    {
        // Nothing, nothing at all, more than the API stores, and a segment carrying a character an
        // event type is not written with.
        Assert.Throws<EventTypeException>(() => EventType.Parse(null!));
        Assert.Throws<EventTypeException>(() => EventType.Parse(string.Empty));
        Assert.Throws<EventTypeException>(() => EventType.Parse(new string('a', EventType.MaxLength + 1)));
        Assert.Throws<EventTypeException>(() => EventType.Parse("auth.user!.create"));
    }

    [Fact]
    public void MintedIdentifiersCarryTheMomentTheyWereMintedIn()
    {
        // The tail is random, so two minted inside one millisecond carry no order: what is ordered is
        // the moment, and that is what is asserted rather than the whole identifier.
        Guid first = Hook0Client.NewEventId();
        Assert.Equal('7', first.ToString("D")[14]);
        Assert.True("89ab".Contains(first.ToString("D")[19], StringComparison.Ordinal));

        long before = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds();
        Guid minted = Hook0Client.NewEventId();
        long after = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds();

        long carried = Moment(minted);
        Assert.True(
            carried >= before && carried <= after,
            $"an identifier minted between {before} and {after} carries {carried}");
    }

    [Fact]
    public async Task IdentifiersMintedAMomentApartAreOrdered()
    {
        Guid earlier = Hook0Client.NewEventId();
        await Task.Delay(TimeSpan.FromMilliseconds(5));
        Guid later = Hook0Client.NewEventId();

        Assert.True(
            Moment(earlier) < Moment(later),
            $"{earlier} was minted before {later} and does not sort before it");
    }

    /// <summary>The moment a UUIDv7 carries, which is its leading 48 bits.</summary>
    private static long Moment(Guid minted)
    {
        byte[] written = minted.ToByteArray(bigEndian: true);
        long moment = 0;
        for (int index = 0; index < 6; index++)
        {
            moment = (moment << 8) | written[index];
        }

        return moment;
    }

    /// <summary>What request number <paramref name="index"/> carried under that member.</summary>
    private string Sent(int index, string name) =>
        Api.Received[index].Json()[name]?.GetValue<string>() ?? string.Empty;

    private static Task<string> Send(Hook0Client client, Event ingested, Surface surface) =>
        surface == Surface.Blocking
            ? Task.Run(() => client.SendEvent(ingested))
            : client.SendEventAsync(ingested);

    private static Task<IReadOnlyList<string>> Upserted(
        Hook0Client client,
        IReadOnlyList<string> eventTypes,
        Surface surface) =>
        surface == Surface.Blocking
            ? Task.Run(() => client.UpsertEventTypes(eventTypes))
            : client.UpsertEventTypesAsync(eventTypes);
}
