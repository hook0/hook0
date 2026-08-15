// The cases the shared conformance corpus dictates, run against this client.
//
// The corpus sits at `clients/conformance`, is hand-authored, and is read by the suite of every SDK.
// Nothing below writes down a verdict, a bound, a header or a signature of its own: they are read
// out of the committed documents and this client is driven against them over a real socket. A case
// added to the corpus is therefore exercised here without this file being touched, and a verdict
// changed there fails here until this client agrees with it again.
//
// Every verdict is driven through both surfaces. A client that carries a blocking idiom and an
// awaiting one carries two chances to be wrong, and a corpus exercised through only one of them
// would say nothing about the other.

using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Text;
using System.Text.Json.Nodes;
using System.Threading.Tasks;
using Xunit;

namespace Hook0.Tests;

/// <summary>Which surface of the client a case drives.</summary>
public enum Surface
{
    /// <summary>The one that waits for its answer.</summary>
    Blocking,

    /// <summary>The one that awaits it.</summary>
    Awaiting,
}

/// <summary>What the shared contract says, held against what this client does.</summary>
public sealed class ConformanceTests : ApiCase
{
    private static readonly JsonNode Retry = Corpus.Contract("retry.json");
    private static readonly JsonNode Bounds = Corpus.Contract("bounds.json")["bounds"]!;
    private static readonly JsonNode Signatures = Corpus.Contract("signature.json");
    private static readonly JsonNode Requests = Corpus.Contract("request.json");

    private const string IngestedId = "01961234-5678-7abc-8def-0123456789ac";
    private const string Token = "token-xyz";

    /// <summary>The budget the delay cases share.</summary>
    /// <remarks>
    /// A delay the API names above it is expected to be cut down to it, so this also bounds what
    /// those cases cost.
    /// </remarks>
    private static readonly TimeSpan DelayBudget = TimeSpan.FromSeconds(1.1);

    /// <summary>
    /// What a wait may overshoot by before it is read as more than what was asked for: a loopback
    /// round trip, a timer and a scheduler all sit inside it.
    /// </summary>
    private static readonly TimeSpan DelaySlack = TimeSpan.FromSeconds(0.6);

    /// <summary>
    /// How a refusal the corpus names reads in this client's own words. Every name the corpus
    /// declares is looked up here, so one added there stops this suite until it is mapped rather
    /// than passing under whatever the client happened to say.
    /// </summary>
    private static readonly IReadOnlyDictionary<string, SignatureRefusal> Refusals =
        new Dictionary<string, SignatureRefusal>(StringComparer.Ordinal)
        {
            ["code_not_hexadecimal"] = SignatureRefusal.CodeNotHexadecimal,
            ["header_not_delivered"] = SignatureRefusal.HeaderNotDelivered,
            ["code_mismatch"] = SignatureRefusal.CodeMismatch,
            ["outside_tolerance"] = SignatureRefusal.OutsideTolerance,
        };

    /// <summary>Which requests each occasion the corpus names covers.</summary>
    private const string EveryRequest = "every request";
    private const string RequestWithABody = "a request carrying a body";

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task TheCorpusSaysWhatEveryProblemDoesToASend(Surface surface)
    {
        // The status is not what decides: the corpus carries problems answering the same status with
        // opposite verdicts, and a client reading the status alone fails half of them.
        foreach (JsonNode? rule in Retry["problems"]!.AsArray())
        {
            string problem = Text(rule!, "problem");
            int status = Number(rule!, "status");
            bool retryable = Flag(rule!, "retryable");

            (int issued, bool ingested) = await IssuedFor(Refusal(status, problem), surface);
            int expected = retryable ? 2 : 1;

            Assert.True(
                expected == issued,
                $"`{problem}` under {status} issued {issued} requests where the corpus expects " +
                $"{expected}: {Text(rule!, "reason")}");
            Assert.Equal(retryable, ingested);
        }
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task TheCorpusSaysWhatEveryStatusDoesToASend(Surface surface)
    {
        // A body naming no problem this client could read is also what an older client meets when
        // the API names a problem it has never heard of.
        foreach (JsonNode? rule in Retry["statuses"]!.AsArray())
        {
            int status = Number(rule!, "status");
            bool retryable = Flag(rule!, "retryable");

            (int issued, _) = await IssuedFor(
                    Refusal(status, "AProblemThisClientHasNeverHeardOf"),
                    surface);
            int expected = retryable ? 2 : 1;

            Assert.True(
                expected == issued,
                $"a status of {status} issued {issued} requests where the corpus expects " +
                $"{expected}: {Text(rule!, "reason")}");
        }
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task TheCorpusSaysWhatARequestTheApiNeverAnsweredDoes(Surface surface)
    {
        // Every cause the corpus names is provoked for real rather than reported: a server that sits
        // on an answer past the timeout, an answer above a ceiling this client set for itself, and a
        // URL nothing can be sent to.
        foreach (JsonNode? rule in Retry["transport"]!["causes"]!.AsArray())
        {
            string cause = Text(rule!, "cause");
            bool retryable = Flag(rule!, "retryable");

            (int issued, bool ingested) = await Provoked(cause, surface);
            int expected = retryable ? 2 : 1;

            Assert.True(
                expected == issued,
                $"`{cause}` issued {issued} requests where the corpus expects {expected}: " +
                Text(rule!, "reason"));
            Assert.Equal(retryable, ingested);
        }
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task TheDelayTheApiNamesIsHonouredAndBounded(Surface surface)
    {
        // The header is written by the other end, so honouring it whole would hand a stranger the
        // length of this client's send. What the corpus asks for is that a delay be waited out when
        // the budget can afford it and cut down to what is left of the budget when it cannot.
        string header = Text(Retry["retry_after"]!, "header");

        foreach (JsonNode? delay in Retry["retry_after"]!["cases"]!.AsArray())
        {
            TimeSpan waited = await WaitedFor(delay!, header, surface);
            TimeSpan expected = Flag(delay!, "honoured")
                ? TimeSpan.FromSeconds(Math.Min(Number(delay!, "seconds"), DelayBudget.TotalSeconds))
                : TimeSpan.Zero;

            Assert.True(
                waited >= expected,
                $"`{header}: {Text(delay!, "header")}` was retried after {waited.TotalSeconds:F3}s, " +
                $"sooner than the {expected.TotalSeconds:F3}s it asked for");
            Assert.True(
                waited <= expected + DelaySlack,
                $"`{header}: {Text(delay!, "header")}` held the send for {waited.TotalSeconds:F3}s, " +
                $"above the {expected.TotalSeconds:F3}s it is bounded to");
        }
    }

    [Fact]
    public void TheBoundsAreTheOnesTheCorpusNames()
    {
        // This client's defaults, held against the one place the numbers are written down. What is
        // asserted is read from the corpus rather than listed here, so a bound added there and left
        // unapplied fails instead of passing unnoticed.
        using Hook0Client built = new("http://127.0.0.1:1", "app-123", Token);
        RetryPolicy policy = built.Options.RetryPolicy;
        Dictionary<string, double> applied = new(StringComparer.Ordinal)
        {
            ["max_attempts"] = policy.MaxAttempts,
            ["max_attempts_cap"] = RetryPolicy.MaxAttemptsCap,
            ["initial_backoff_ms"] = policy.InitialBackoff.TotalMilliseconds,
            ["max_backoff_ms"] = policy.MaxBackoff.TotalMilliseconds,
            ["max_total_delay_ms"] = policy.MaxTotalDelay.TotalMilliseconds,
            ["request_timeout_ms"] = built.Options.RequestTimeout.TotalMilliseconds,
            ["max_payload_bytes"] = built.Options.MaxPayloadBytes,
            ["max_response_bytes"] = built.Options.MaxResponseBytes,
            ["max_head_bytes"] = built.Options.MaxHeadBytes,
            ["max_response_headers"] = built.Options.MaxResponseHeaders,
            ["max_header_bytes"] = built.Options.MaxHeaderBytes,
        };

        List<string> named = [.. Bounds.AsObject().Select(bound => bound.Key)];
        List<string> unapplied = [.. named.Except(applied.Keys, StringComparer.Ordinal)];
        Assert.True(
            unapplied.Count == 0,
            $"the corpus names bounds this client does not apply: {string.Join(", ", unapplied)}");

        foreach (string name in named)
        {
            double wanted = Bounds[name]!.GetValue<double>();
            Assert.True(
                Math.Abs(wanted - applied[name]) < 0.001,
                $"the corpus names {name} as {wanted} and this client applies {applied[name]}");
        }
    }

    [Fact]
    public void EveryRefusalTheCorpusDeclaresReadsAsOneOfThisClients()
    {
        // A refusal named in the corpus and mapped to nothing here would pass under any wording.
        List<string> declared = [.. Signatures["refusals"]!.AsArray().Select(name => name!.GetValue<string>())];
        List<string> unmapped = [.. declared.Except(Refusals.Keys, StringComparer.Ordinal)];

        Assert.True(
            unmapped.Count == 0,
            $"the corpus declares refusals this suite maps to nothing: {string.Join(", ", unmapped)}");
    }

    [Fact]
    public void EveryDeliveryOfTheCorpusIsVerifiedAsItSays()
    {
        // A refused delivery has to be refused for the reason the corpus names: a client that
        // computed a code over a header that never arrived and reported a mismatch would otherwise
        // look right.
        foreach (JsonNode? vector in Signatures["vectors"]!.AsArray())
        {
            if (Text(vector!, "verdict") == "accepted")
            {
                Verified(vector!);
                continue;
            }

            SignatureException refused = Assert.Throws<SignatureException>(() => Verified(vector!));
            SignatureRefusal wanted = Refusals[Text(vector!, "refusal")];

            Assert.True(
                wanted == refused.Refusal,
                $"a delivery the corpus refuses as `{Text(vector!, "refusal")}` was refused as " +
                $"`{refused.Refusal}`: {Text(vector!, "reason")}");
        }
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task EveryHeaderTheCorpusPinsArrivesAtTheSocket(Surface surface)
    {
        // What the corpus pins is held against what actually reached a listening socket, not against
        // what the client believes it set. Header names are matched without regard to case, as HTTP
        // compares them.
        List<string> occasions = [.. Requests["occasions"]!.AsArray().Select(when => when!.GetValue<string>())];
        List<string> unknown =
            [.. occasions.Except([EveryRequest, RequestWithABody], StringComparer.Ordinal)];
        Assert.True(
            unknown.Count == 0,
            $"the corpus names occasions this suite cannot decide: {string.Join(", ", unknown)}");

        Api.WillAnswer(Ingested(IngestedId), Ingested(IngestedId));
        using Hook0Client client = Client();

        await Send(client, AnEvent(), surface);
        ReceivedRequest carrying = Assert.Single(Api.Received);
        Carries(carrying, body: true);

        Restarted();
        Api.WillAnswer(new ScriptedResponse(
            200,
            new JsonArray(new JsonObject { ["event_type_name"] = "auth.user.create" })));
        using Hook0Client reading = Client();
        await Upserted(reading, surface);
        Carries(Assert.Single(Api.Received), body: false);
    }

    /// <summary>Holds one request that reached the socket against every header the corpus pins.</summary>
    private static void Carries(ReceivedRequest received, bool body)
    {
        Assert.Equal(body, received.Body.Length > 0);

        int composedAtMost = Requests["max_composed_bytes"]!.GetValue<int>();
        Dictionary<string, string> bound = new(StringComparer.Ordinal)
        {
            ["token"] = Token,
            ["language"] = "csharp",
        };

        foreach (JsonNode? pinned in Requests["headers"]!.AsArray())
        {
            string name = Text(pinned!, "name");
            string when = Text(pinned!, "when");
            string template = Text(pinned!, "value");
            List<string> chunks = TemplateChunks(template, bound);

            List<string> arrived =
            [
                .. received.Headers
                    .Where(header => string.Equals(header.Key, name, StringComparison.OrdinalIgnoreCase))
                    .Select(header => header.Value),
            ];

            if (when == RequestWithABody && !body)
            {
                Assert.True(
                    arrived.Count == 0,
                    $"a request carrying no body carried `{name}`, which the corpus pins to " +
                    $"`{when}`: {Text(pinned!, "reason")}");
                continue;
            }

            Assert.True(
                arrived.Count > 0,
                $"the request carried no `{name}` header, which the corpus pins to `{when}`: " +
                Text(pinned!, "reason"));
            Assert.True(
                arrived.Exists(one => MatchesChunks(chunks, one)),
                $"the request carried `{name}: {string.Join(", ", arrived)}` where the corpus says " +
                $"`{template}`: {Text(pinned!, "reason")}");

            // A value with a hole this suite cannot fill is one the client composed out of what the
            // platform told it, and what the platform says is as long as it feels like.
            if (chunks.Count > 1)
            {
                foreach (string one in arrived)
                {
                    Assert.True(
                        Encoding.UTF8.GetByteCount(one) <= composedAtMost,
                        $"the request carried {Encoding.UTF8.GetByteCount(one)} bytes of `{name}`, " +
                        $"above the {composedAtMost} the corpus cuts a composed value to");
                }
            }
        }
    }

    /// <summary>
    /// What a value of the request document is made of, once the holes this suite can speak for are
    /// filled in.
    /// </summary>
    /// <remarks>
    /// A value is a template: <c>${name}</c> is a hole and everything around it is literal. A hole
    /// named in <paramref name="bound"/> becomes part of the literal text around it; one that is not
    /// is a hole no suite can fill without reimplementing the client it is testing, and it separates
    /// two chunks. A template whose holes are all bound is therefore one chunk, and the whole value
    /// is that chunk.
    /// </remarks>
    /// <param name="template">The value the corpus writes down.</param>
    /// <param name="bound">What each hole this suite can speak for carries.</param>
    /// <returns>The literal text of the value, one chunk per hole it leaves open.</returns>
    private static List<string> TemplateChunks(string template, IReadOnlyDictionary<string, string> bound)
    {
        List<string> chunks = [string.Empty];
        string rest = template;

        for (int opened = rest.IndexOf("${", StringComparison.Ordinal); opened >= 0;)
        {
            int closed = rest.IndexOf('}', opened);
            if (closed < 0)
            {
                break;
            }

            chunks[^1] += rest[..opened];
            if (bound.TryGetValue(rest[(opened + 2)..closed], out string? filled))
            {
                chunks[^1] += filled;
            }
            else
            {
                chunks.Add(string.Empty);
            }

            rest = rest[(closed + 1)..];
            opened = rest.IndexOf("${", StringComparison.Ordinal);
        }

        chunks[^1] += rest;
        return chunks;
    }

    /// <summary>
    /// Whether what arrived is what those chunks describe: the literal text in order, anchored at
    /// both ends, with something non-empty standing in every hole between them.
    /// </summary>
    /// <param name="chunks">The literal text the value is made of.</param>
    /// <param name="carried">What reached the socket under that header.</param>
    /// <returns>Whether the one describes the other.</returns>
    private static bool MatchesChunks(IReadOnlyList<string> chunks, string carried)
    {
        if (chunks.Count == 1)
        {
            return string.Equals(carried, chunks[0], StringComparison.Ordinal);
        }

        if (!carried.StartsWith(chunks[0], StringComparison.Ordinal))
        {
            return false;
        }

        string rest = carried[chunks[0].Length..];
        for (int chunk = 1; chunk < chunks.Count - 1; chunk++)
        {
            // A hole stands before this chunk, and nothing is not something, so the search starts
            // past whatever fills it.
            int found = rest.Length == 0
                ? -1
                : rest.IndexOf(chunks[chunk], 1, StringComparison.Ordinal);
            if (found < 0)
            {
                return false;
            }

            rest = rest[(found + chunks[chunk].Length)..];
        }

        string last = chunks[^1];
        return rest.Length > last.Length && rest.EndsWith(last, StringComparison.Ordinal);
    }

    /// <summary>One cause of a request the API never answered, provoked over a real socket.</summary>
    private async Task<(int Issued, bool Ingested)> Provoked(string cause, Surface surface)
    {
        Restarted();

        switch (cause)
        {
            case "no_answer":
                // An attempt that runs out of time before the API writes anything.
                Api.WillAnswer(
                    Ingested(IngestedId) with { HeldFor = TimeSpan.FromSeconds(1) },
                    Ingested(IngestedId));
                return await IssuedBy(() => Client(Options(maxAttempts: 4, requestTimeout: 0.2)), surface)
                    ;

            case "answer_above_a_bound":
                // An answer larger than what this client agreed to read off the socket.
                Api.WillAnswer(
                    new ScriptedResponse(
                        201,
                        new JsonObject
                        {
                            ["event_id"] = IngestedId,
                            ["padding"] = new string('x', 2048),
                        }),
                    Ingested(IngestedId));
                return await IssuedBy(
                        () => Client(Options(maxAttempts: 4) with { MaxResponseBytes = 256 }),
                        surface)
                    ;

            case "unusable_api_url":
                // A base URL nothing can be sent to, which means nothing is ever sent.
                Api.WillAnswer(Ingested(IngestedId));
                return await IssuedBy(
                        () => new Hook0Client(
                            "gopher://nowhere.invalid",
                            "app-123",
                            Token,
                            Options(maxAttempts: 4)),
                        surface)
                    ;

            default:
                throw new InvalidOperationException(
                    $"the corpus names a cause `{cause}` this suite does not know how to provoke");
        }
    }

    /// <summary>How many attempts a send made, and whether it ended up ingesting the event.</summary>
    /// <remarks>
    /// A send that reached a server is counted by what that server received. One that never reached
    /// anything — an API URL nothing can be sent to is the corpus's own example — is counted by what
    /// the client says it did, which is also what a caller is left holding: a misconfiguration
    /// retried four times reads as a network that would not answer.
    /// </remarks>
    private async Task<(int Issued, bool Ingested)> IssuedBy(Func<Hook0Client> build, Surface surface)
    {
        using Hook0Client built = build();
        try
        {
            await Send(built, AnEvent(), surface);
            return (Api.Received.Count, true);
        }
        catch (SendException gave)
        {
            return (Math.Max(Api.Received.Count, gave.Attempts), false);
        }
    }

    /// <summary>How many requests a send made when the API answered that way and then took the event.</summary>
    private async Task<(int Issued, bool Ingested)> IssuedFor(ScriptedResponse answer, Surface surface)
    {
        Restarted();
        Api.WillAnswer(answer, Ingested(IngestedId));
        return await IssuedBy(() => Client(Options(maxAttempts: 4)), surface);
    }

    /// <summary>How long a send spent waiting when the API named that delay beside a paced answer.</summary>
    private async Task<TimeSpan> WaitedFor(JsonNode delay, string header, Surface surface)
    {
        (JsonNode paced, _) = PacedPair();
        Restarted();
        Api.WillAnswer(
            Refusal(
                Number(paced, "status"),
                Text(paced, "problem"),
                [new KeyValuePair<string, string>(header, Text(delay, "header"))]),
            Ingested(IngestedId));

        using Hook0Client client = Client(new ClientOptions
        {
            RetryPolicy = new RetryPolicy
            {
                MaxAttempts = 4,
                InitialBackoff = PromptBackoff,
                MaxBackoff = PromptBackoff,
                MaxTotalDelay = DelayBudget,
            },
            RequestTimeout = TimeSpan.FromSeconds(5),
        });

        Stopwatch started = Stopwatch.StartNew();
        await Send(client, AnEvent(), surface);
        started.Stop();

        Assert.True(Api.Received.Count == 2, "a paced answer was not retried");
        return started.Elapsed;
    }

    /// <summary>Two problems answering the same status, one worth repeating and one not.</summary>
    /// <remarks>
    /// That pair is the whole reason the corpus classifies problems rather than statuses, and the
    /// retryable one is the answer the API names a delay beside.
    /// </remarks>
    private static (JsonNode Retryable, JsonNode Refused) PacedPair()
    {
        JsonArray problems = Retry["problems"]!.AsArray();
        foreach (JsonNode? rule in problems)
        {
            if (!Flag(rule!, "retryable"))
            {
                continue;
            }

            foreach (JsonNode? other in problems)
            {
                if (Number(other!, "status") == Number(rule!, "status") && !Flag(other!, "retryable"))
                {
                    return (rule!, other!);
                }
            }
        }

        throw new InvalidOperationException("no status of the corpus carries opposite verdicts");
    }

    /// <summary>One delivery of the corpus, verified against the moment the corpus names.</summary>
    private static void Verified(JsonNode vector)
    {
        List<KeyValuePair<string, string>> headers = [];
        foreach (JsonNode? header in vector["headers"]!.AsArray())
        {
            JsonArray pair = header!.AsArray();
            headers.Add(new KeyValuePair<string, string>(
                pair[0]!.GetValue<string>(),
                pair[1]!.GetValue<string>()));
        }

        Webhooks.VerifyWebhookSignatureWithCurrentTime(
            Text(vector, "signature"),
            Encoding.UTF8.GetBytes(Text(vector, "payload")),
            headers,
            Text(vector, "secret"),
            TimeSpan.FromSeconds(vector["tolerance_seconds"]!.GetValue<double>()),
            DateTimeOffset.FromUnixTimeSeconds(vector["current_time"]!.GetValue<long>()));
    }

    /// <summary>Sends one event through whichever surface the case is driving.</summary>
    private static Task<string> Send(Hook0Client client, Event ingested, Surface surface) =>
        surface == Surface.Blocking
            ? Task.Run(() => client.SendEvent(ingested))
            : client.SendEventAsync(ingested);

    /// <summary>Reads the event types an application declares, through whichever surface.</summary>
    private static Task<IReadOnlyList<string>> Upserted(Hook0Client client, Surface surface) =>
        surface == Surface.Blocking
            ? Task.Run(() => client.UpsertEventTypes(["auth.user.create"]))
            : client.UpsertEventTypesAsync(["auth.user.create"]);

}
