using System;
using System.Buffers.Binary;
using System.Collections.Generic;
using System.Globalization;
using System.Security.Cryptography;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;

namespace Hook0;

/// <summary>How a client spaces out the attempts of a single send.</summary>
/// <remarks>
/// The delay before a retry doubles from <see cref="InitialBackoff"/> and is capped by
/// <see cref="MaxBackoff"/>; the delay actually waited is then drawn anywhere between zero and that
/// ceiling, so that emitters which failed at the same moment do not come back at the same moment.
/// Retrying stops as soon as the delays of the send would add up to more than
/// <see cref="MaxTotalDelay"/>.
/// <para>
/// The defaults are four attempts spread over at most five seconds: three retries absorb the blips a
/// webhook emitter meets in production — a connection reset, a rolling deployment answering 503 —
/// without holding the caller for long, and the five-second budget bounds what the worst send costs
/// whatever the individual delays turn out to be.
/// </para>
/// </remarks>
public sealed record RetryPolicy
{
    /// <summary>Most attempts a policy can ever make, whatever <see cref="MaxAttempts"/> says.</summary>
    /// <remarks>
    /// A policy is configuration, and configuration can be wrong; this cap keeps a mistyped
    /// <see cref="MaxAttempts"/> from turning one send into an unbounded series of requests.
    /// </remarks>
    public const int MaxAttemptsCap = 16;

    /// <summary>Beyond this many doublings any backoff has long since reached its ceiling.</summary>
    private const int MaxBackoffDoublings = 30;

    /// <summary>Attempts a single send makes at most, the first one included. <c>1</c> disables retrying.</summary>
    public int MaxAttempts { get; init; } = 4;

    /// <summary>Ceiling of the delay before the first retry.</summary>
    public TimeSpan InitialBackoff { get; init; } = TimeSpan.FromMilliseconds(100);

    /// <summary>Ceiling no single delay ever exceeds.</summary>
    public TimeSpan MaxBackoff { get; init; } = TimeSpan.FromSeconds(2);

    /// <summary>Budget all the delays of one send share.</summary>
    public TimeSpan MaxTotalDelay { get; init; } = TimeSpan.FromSeconds(5);

    /// <summary>A policy that never retries: one attempt, and the caller hears what it answered.</summary>
    public static RetryPolicy Disabled => new()
    {
        MaxAttempts = 1,
        InitialBackoff = TimeSpan.Zero,
        MaxBackoff = TimeSpan.Zero,
        MaxTotalDelay = TimeSpan.Zero,
    };

    /// <summary>Attempts this policy actually makes: <see cref="MaxAttempts"/>, brought inside its cap.</summary>
    public int Attempts => Math.Clamp(MaxAttempts, 1, MaxAttemptsCap);

    /// <summary>Ceiling of the delay before retry number <paramref name="retryNumber"/>.</summary>
    /// <remarks>
    /// It doubles from <see cref="InitialBackoff"/> and never exceeds <see cref="MaxBackoff"/>, so
    /// the ceilings of successive retries never decrease.
    /// </remarks>
    /// <param name="retryNumber">Which retry this is, where <c>1</c> is the first.</param>
    /// <returns>The longest that retry may be delayed by.</returns>
    public TimeSpan BackoffCeiling(int retryNumber)
    {
        int doublings = Math.Clamp(retryNumber - 1, 0, MaxBackoffDoublings);
        double ceiling = Math.Max(MaxBackoff.TotalSeconds, 0);
        double doubled = Math.Max(InitialBackoff.TotalSeconds, 0) * Math.Pow(2, doublings);
        return TimeSpan.FromSeconds(Math.Clamp(doubled, 0, ceiling));
    }

    /// <summary>The delays this policy waits between the attempts of one send, one per retry.</summary>
    /// <remarks>
    /// Each delay lands between zero and the ceiling of its retry, and the schedule is cut short as
    /// soon as the next delay would spend more than <see cref="MaxTotalDelay"/>. There are therefore
    /// at most <c>Attempts - 1</c> delays, and they add up to at most <see cref="MaxTotalDelay"/>.
    /// <para>
    /// A draw that is missing or is not a finite number is read as <c>1</c>, which asks for the whole
    /// ceiling: an unusable source of randomness makes the client wait longer, never less.
    /// </para>
    /// </remarks>
    /// <param name="draws">One draw in <c>[0, 1)</c> per retry.</param>
    /// <returns>The delays, in the order they are waited out.</returns>
    public IReadOnlyList<TimeSpan> Delays(IReadOnlyList<double> draws)
    {
        double budget = Math.Max(MaxTotalDelay.TotalSeconds, 0);
        List<TimeSpan> waits = new(Math.Max(Attempts - 1, 0));
        double spent = 0;

        for (int retryNumber = 1; retryNumber <= Attempts - 1; retryNumber++)
        {
            double delay = BackoffCeiling(retryNumber).TotalSeconds * Draw(draws, retryNumber - 1);
            if (spent + delay > budget)
            {
                break;
            }

            spent += delay;
            waits.Add(TimeSpan.FromSeconds(delay));
        }

        return waits;
    }

    /// <summary>The draw for one retry, brought back inside <c>[0, 1]</c> whatever the randomness gave.</summary>
    /// <param name="draws">What the randomness gave.</param>
    /// <param name="index">Which draw this is.</param>
    /// <returns>A number between zero and one.</returns>
    public static double Draw(IReadOnlyList<double> draws, int index)
    {
        if (draws is null || index < 0 || index >= draws.Count)
        {
            return 1;
        }

        double drawn = draws[index];
        return double.IsFinite(drawn) ? Math.Clamp(drawn, 0, 1) : 1;
    }
}

/// <summary>Every bound a client applies to one send.</summary>
public sealed record ClientOptions
{
    /// <summary>Largest event payload the client agrees to send, in bytes.</summary>
    /// <remarks>
    /// Hook0's API refuses request bodies above 2 MiB, so a payload above 1 MiB is already at risk of
    /// being refused once the JSON envelope around it — metadata, labels, identifiers — is counted.
    /// The client rules such an event out rather than spending a round trip, and every retry after
    /// it, on a request that cannot be accepted.
    /// </remarks>
    public const int DefaultMaxPayloadBytes = 1024 * 1024;

    /// <summary>How the attempts of one send are spaced out.</summary>
    public RetryPolicy RetryPolicy { get; init; } = new();

    /// <summary>How long one attempt is given.</summary>
    public TimeSpan RequestTimeout { get; init; } = HttpTransport.DefaultRequestTimeout;

    /// <summary>The largest payload sent, refused before a socket is opened.</summary>
    public int MaxPayloadBytes { get; init; } = DefaultMaxPayloadBytes;

    /// <summary>The largest answer read off a socket.</summary>
    public int MaxResponseBytes { get; init; } = HttpTransport.DefaultMaxResponseBytes;

    /// <summary>How many header lines an answer may carry.</summary>
    public int MaxResponseHeaders { get; init; } = HttpTransport.DefaultMaxResponseHeaders;

    /// <summary>How long the whole head of an answer may be.</summary>
    public int MaxHeadBytes { get; init; } = HttpTransport.DefaultMaxHeadBytes;

    /// <summary>How long one header line of it may be.</summary>
    public int MaxHeaderBytes { get; init; } = HttpTransport.DefaultMaxHeaderBytes;
}

/// <summary>An event to send to Hook0.</summary>
/// <remarks>
/// <see cref="EventId"/> is the caller's to set when it already has one to key the event on. Left
/// unset, the client generates a UUIDv7, sends it and answers it — which is what lets it repeat a
/// request without risking a second copy of the event being ingested and delivered to every
/// subscriber.
/// </remarks>
public sealed record Event
{
    /// <summary>The type of the event, as the application declares it.</summary>
    public required string EventType { get; init; }

    /// <summary>What the event carries.</summary>
    public required string Payload { get; init; }

    /// <summary>How to read the payload.</summary>
    public required string PayloadContentType { get; init; }

    /// <summary>What Hook0 routes the event by.</summary>
    public IReadOnlyDictionary<string, string> Labels { get; init; } =
        new Dictionary<string, string>(StringComparer.Ordinal);

    /// <summary>Anything else worth carrying.</summary>
    public IReadOnlyDictionary<string, string>? Metadata { get; init; }

    /// <summary>When the event happened; the current moment when unset.</summary>
    public DateTimeOffset? OccurredAt { get; init; }

    /// <summary>What to key the event on; the client chooses when unset.</summary>
    public Guid? EventId { get; init; }
}

/// <summary>An event type, read out of the <c>service.resource_type.verb</c> it is written as.</summary>
/// <param name="Service">The leading segment.</param>
/// <param name="ResourceType">The middle segment.</param>
/// <param name="Verb">The trailing segment.</param>
public sealed record EventType(string Service, string ResourceType, string Verb)
{
    /// <summary>Longest an event type may be written as.</summary>
    public const int MaxLength = 256;

    /// <summary>How many segments one is made of.</summary>
    private const int Segments = 3;

    /// <summary>Reads an event type, refusing one that does not name all three of its parts.</summary>
    /// <param name="written">The event type as the application writes it.</param>
    /// <returns>Its three segments.</returns>
    /// <exception cref="EventTypeException">When it does not read as three segments.</exception>
    public static EventType Parse(string written)
    {
        if (written is null || written.Length == 0 || written.Length > MaxLength)
        {
            throw EventTypeException.Invalid(written ?? string.Empty);
        }

        string[] parts = written.Split('.');
        if (parts.Length != Segments)
        {
            throw EventTypeException.Invalid(written);
        }

        foreach (string part in parts)
        {
            if (part.Length == 0 || !Segment(part))
            {
                throw EventTypeException.Invalid(written);
            }
        }

        return new EventType(parts[0], parts[1], parts[2]);
    }

    /// <summary>The event type as the API reads one.</summary>
    /// <returns>Its three segments, joined.</returns>
    public string Written() => $"{Service}.{ResourceType}.{Verb}";

    private static bool Segment(string part)
    {
        foreach (char character in part)
        {
            if (!char.IsAsciiLetterOrDigit(character) && character != '_')
            {
                return false;
            }
        }

        return true;
    }
}

/// <summary>The Hook0 client, built once and shared wherever an application sends events.</summary>
/// <remarks>
/// Every event is sent under an identifier this client knows: the one set on the <see cref="Event"/>,
/// or a UUIDv7 it generates when the event carries none. Passing none does not mean the identifier
/// comes from Hook0 — the value comes from here, travels with the request, and is what
/// <see cref="SendEvent"/> answers.
/// <para>
/// That is what makes retrying safe. Hook0 keys events on that identifier, so a request repeated
/// after a network failure or a server error ingests the event once rather than twice; without a
/// client-chosen identifier, a repeated request would create a second event and deliver it to every
/// subscriber. It also gives the answer to a retry its meaning: <c>EventAlreadyIngested</c> in reply
/// to a <em>repeated</em> request says an earlier attempt of that same send reached the API, so the
/// send succeeded. The same answer to a <em>first</em> attempt is a genuine conflict and is reported
/// as one.
/// </para>
/// <para>
/// Only what could end differently is retried: a request that got no answer, a server error, and an
/// instance saying it is being reached faster than it accepts. What the API refuses outright — a
/// quota that is spent, a payload it will not read — is reported as is. The verdict for every problem
/// the API can report is written down in the conformance corpus committed beside this package, which
/// the suite here reads.
/// </para>
/// <para>
/// Both idioms are served by one decision: <see cref="SendEvent"/> and <see cref="SendEventAsync"/>
/// differ only in how they issue an attempt and how they wait, and everything that reads an answer
/// and decides what to do about it is shared between them. A verdict fixed for one is fixed for
/// both, because there is only one of it.
/// </para>
/// </remarks>
public sealed class Hook0Client : IDisposable
{
    /// <summary>The problem the API answers when an event identifier is already taken.</summary>
    public const string AlreadyIngested = "EventAlreadyIngested";

    /// <summary>
    /// The problem the API answers when requests are reaching the instance faster than it accepts
    /// them.
    /// </summary>
    /// <remarks>
    /// It shares its status with the quota problems, and is the only one of them worth repeating: a
    /// quota clears when a plan changes or a day turns, neither of which happens inside the seconds a
    /// send is given, while pacing clears on its own and the answer says when.
    /// </remarks>
    public const string RateLimited = "RateLimited";

    /// <summary>What the API answers when the event identifier a request carries is already taken.</summary>
    private const int Conflict = 409;

    /// <summary>
    /// What the API answers both when a quota is spent and when requests are coming in faster than
    /// the instance accepts them. Which of the two it is only the problem the body names can say,
    /// which is why this status alone decides nothing.
    /// </summary>
    private const int Paced = 429;

    /// <summary>First status saying the failure is on the API's side, and so could clear on its own.</summary>
    private const int LowestServerError = 500;

    /// <summary>Lowest status a response is read as a success under.</summary>
    private const int LowestSuccess = 200;

    /// <summary>Lowest status that is no longer a success.</summary>
    private const int LowestRedirection = 300;

    /// <summary>What the API names the delay before the request becomes servable in, in whole seconds.</summary>
    private const string DelayHeader = "retry-after";

    /// <summary>
    /// Longest value of that header read, and the largest delay it may name. A header written by the
    /// other end is bounded before it is turned into a number, and a delay above this is one nobody
    /// meant.
    /// </summary>
    private const int MaxDelayHeaderBytes = 32;
    private const long MaxNamedDelaySeconds = int.MaxValue;

    /// <summary>Most event types one call declares, which bounds what one round of upserts costs.</summary>
    private const int MaxUpsertedEventTypes = 256;

    /// <summary>Where an event is ingested, under the API URL.</summary>
    private const string EventPath = "event";

    /// <summary>Where event types are read and created, under the API URL.</summary>
    private const string EventTypesPath = "event_types";

    private static readonly IReadOnlyList<KeyValuePair<string, string>> NoQuery =
        Array.Empty<KeyValuePair<string, string>>();

    private readonly string _applicationId;

    /// <summary>Reaches a Hook0 instance and sends events to one of its applications.</summary>
    /// <param name="apiUrl">Base API URL, such as <c>https://app.hook0.com/api/v1</c>.</param>
    /// <param name="applicationId">Identifier of the application events are sent to.</param>
    /// <param name="token">An authentication token valid for that application.</param>
    /// <param name="options">The bounds one send is held to.</param>
    public Hook0Client(string apiUrl, string applicationId, string token, ClientOptions? options = null)
    {
        _applicationId = applicationId;
        Options = options ?? new ClientOptions();
        Transport = new HttpTransport(
            apiUrl,
            token,
            Options.RequestTimeout,
            Options.MaxResponseBytes,
            Options.MaxResponseHeaders,
            Options.MaxHeadBytes,
            Options.MaxHeaderBytes);
    }

    /// <summary>The bounds one send is held to.</summary>
    public ClientOptions Options { get; }

    /// <summary>
    /// What this client issues its requests through, which is also what a generated operation group
    /// is built on — in either idiom.
    /// </summary>
    public HttpTransport Transport { get; }

    /// <summary>Mints the kind of identifier Hook0 uses when it is the one choosing.</summary>
    /// <remarks>
    /// Its leading 48 bits are the current time in milliseconds, so identifiers minted in different
    /// milliseconds are ordered, which is what keeps the index they end up in from being written all
    /// over. The rest is random, so two minted inside one millisecond carry no order between them.
    /// Written here rather than taken from the framework, which has had <c>Guid.CreateVersion7</c>
    /// only since .NET 9 and this package targets older.
    /// </remarks>
    /// <returns>A UUIDv7.</returns>
    public static Guid NewEventId()
    {
        Span<byte> drawn = stackalloc byte[16];
        RandomNumberGenerator.Fill(drawn);

        long milliseconds = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds();
        Span<byte> moment = stackalloc byte[8];
        BinaryPrimitives.WriteInt64BigEndian(moment, milliseconds);
        moment[2..].CopyTo(drawn);

        drawn[6] = (byte)((drawn[6] & 0x0F) | 0x70);
        drawn[8] = (byte)((drawn[8] & 0x3F) | 0x80);

        return new Guid(drawn, bigEndian: true);
    }

    /// <summary>Sends an event, and answers the identifier it was sent under.</summary>
    /// <param name="ingested">The event to send.</param>
    /// <returns>The identifier the event was sent under.</returns>
    /// <exception cref="SendException">When the event was not ingested.</exception>
    public string SendEvent(Event ingested)
    {
        Sending sending = Begin(ingested);

        for (int issued = 1; issued <= sending.Attempts; issued++)
        {
            SendStep step = sending.Next(Attempted(sending.Body));
            if (step.Failure is not null)
            {
                throw step.Failure;
            }

            if (step.Ingested is not null)
            {
                return step.Ingested;
            }

            Thread.Sleep(step.Wait);
        }

        throw sending.GaveUp();
    }

    /// <summary>Sends an event, and answers the identifier it was sent under.</summary>
    /// <param name="ingested">The event to send.</param>
    /// <param name="cancellationToken">What abandons the send before it finishes.</param>
    /// <returns>The identifier the event was sent under.</returns>
    /// <exception cref="SendException">When the event was not ingested.</exception>
    public async Task<string> SendEventAsync(Event ingested, CancellationToken cancellationToken = default)
    {
        Sending sending = Begin(ingested);

        for (int issued = 1; issued <= sending.Attempts; issued++)
        {
            SendStep step = sending.Next(
                await AttemptedAsync(sending.Body, cancellationToken).ConfigureAwait(false));
            if (step.Failure is not null)
            {
                throw step.Failure;
            }

            if (step.Ingested is not null)
            {
                return step.Ingested;
            }

            await Task.Delay(step.Wait, cancellationToken).ConfigureAwait(false);
        }

        throw sending.GaveUp();
    }

    /// <summary>Creates the event types the application does not declare yet, and answers those.</summary>
    /// <param name="eventTypes">The event types the application uses.</param>
    /// <returns>The ones this call declared.</returns>
    /// <exception cref="EventTypeException">When the list could not be read or one could not be created.</exception>
    public IReadOnlyList<string> UpsertEventTypes(IReadOnlyList<string> eventTypes)
    {
        IReadOnlyList<EventType> wanted = Wanted(eventTypes);
        if (wanted.Count == 0)
        {
            return Array.Empty<string>();
        }

        IReadOnlySet<string> declared = Declared(
            Answered(() => Transport.Deliver(
                "GET",
                EventTypesPath,
                [new KeyValuePair<string, string>("application_id", _applicationId)],
                null)));

        List<string> created = new(wanted.Count);
        foreach (EventType eventType in wanted)
        {
            if (declared.Contains(eventType.Written()))
            {
                continue;
            }

            Created(
                eventType,
                Answered(() => Transport.Deliver("POST", EventTypesPath, NoQuery, Declaring(eventType))));
            created.Add(eventType.Written());
        }

        return created;
    }

    /// <summary>Creates the event types the application does not declare yet, and answers those.</summary>
    /// <param name="eventTypes">The event types the application uses.</param>
    /// <param name="cancellationToken">What abandons the call before it finishes.</param>
    /// <returns>The ones this call declared.</returns>
    /// <exception cref="EventTypeException">When the list could not be read or one could not be created.</exception>
    public async Task<IReadOnlyList<string>> UpsertEventTypesAsync(
        IReadOnlyList<string> eventTypes,
        CancellationToken cancellationToken = default)
    {
        IReadOnlyList<EventType> wanted = Wanted(eventTypes);
        if (wanted.Count == 0)
        {
            return Array.Empty<string>();
        }

        IReadOnlySet<string> declared = Declared(
            await AnsweredAsync(
                () => Transport.DeliverAsync(
                    "GET",
                    EventTypesPath,
                    [new KeyValuePair<string, string>("application_id", _applicationId)],
                    null,
                    cancellationToken)).ConfigureAwait(false));

        List<string> created = new(wanted.Count);
        foreach (EventType eventType in wanted)
        {
            if (declared.Contains(eventType.Written()))
            {
                continue;
            }

            Created(
                eventType,
                await AnsweredAsync(
                    () => Transport.DeliverAsync(
                        "POST",
                        EventTypesPath,
                        NoQuery,
                        Declaring(eventType),
                        cancellationToken)).ConfigureAwait(false));
            created.Add(eventType.Written());
        }

        return created;
    }

    /// <inheritdoc />
    public void Dispose() => Transport.Dispose();

    /// <summary>One attempt at sending an already-bounded event.</summary>
    private Attempt Attempted(Dictionary<string, object?> body)
    {
        try
        {
            return ReadAttempt(Transport.Deliver("POST", EventPath, NoQuery, body));
        }
        catch (TransportException failure)
        {
            return new Attempt(null, false, failure.Message, failure.Retryable, null);
        }
    }

    /// <summary>One attempt at sending an already-bounded event.</summary>
    private async Task<Attempt> AttemptedAsync(
        Dictionary<string, object?> body,
        CancellationToken cancellationToken)
    {
        try
        {
            return ReadAttempt(
                await Transport.DeliverAsync("POST", EventPath, NoQuery, body, cancellationToken)
                    .ConfigureAwait(false));
        }
        catch (TransportException failure)
        {
            return new Attempt(null, false, failure.Message, failure.Retryable, null);
        }
    }

    /// <summary>What the API answered one attempt, and whether repeating it could end differently.</summary>
    private static Attempt ReadAttempt(TransportDelivery delivered)
    {
        string detail = Runtime.Preview(delivered.Payload);

        if (delivered.Status is >= LowestSuccess and < LowestRedirection)
        {
            string? ingested = Member(delivered.Payload, "event_id");

            // The API accepted the event but answered something this client cannot read; repeating
            // the request would meet the same answer.
            return ingested is null
                ? new Attempt(
                    null,
                    false,
                    string.Create(
                        CultureInfo.InvariantCulture,
                        $"Hook0 answered {delivered.Status} without an event id"),
                    false,
                    null)
                : new Attempt(ingested, false, string.Empty, false, null);
        }

        string? problem = Member(delivered.Payload, "id");
        if (delivered.Status == Conflict && problem == AlreadyIngested)
        {
            return new Attempt(null, true, detail, false, null);
        }

        return new Attempt(null, false, detail, Retryable(delivered.Status, problem), NamedDelay(delivered.Headers));
    }

    /// <summary>Whether repeating a request the API answered that way could end differently.</summary>
    /// <remarks>
    /// The status decides on its own everywhere but under the one it answers both a spent quota and a
    /// paced instance with: a quota clears when a plan changes or a day turns, and neither is
    /// something a send spending seconds can wait for. Only the problem the body names tells the two
    /// apart, and a body naming a problem this client has never heard of falls back to what the
    /// status says.
    /// </remarks>
    private static bool Retryable(int status, string? problem) =>
        status == Paced ? problem == RateLimited : status >= LowestServerError;

    /// <summary>The delay the API named before the request becomes servable.</summary>
    /// <remarks>
    /// Only a whole number of seconds is read. The header may also carry a date, which is a clock
    /// this client would be comparing against its own, and anything else is a header nobody meant:
    /// both leave the client's own schedule in place rather than being guessed at.
    /// </remarks>
    private static TimeSpan? NamedDelay(IReadOnlyDictionary<string, string> headers)
    {
        if (!headers.TryGetValue(DelayHeader, out string? carried))
        {
            return null;
        }

        string written = carried.Trim();
        if (written.Length == 0 || written.Length > MaxDelayHeaderBytes)
        {
            return null;
        }

        foreach (char character in written)
        {
            if (!char.IsAsciiDigit(character))
            {
                return null;
            }
        }

        if (!long.TryParse(written, NumberStyles.None, CultureInfo.InvariantCulture, out long seconds)
            || seconds > MaxNamedDelaySeconds)
        {
            return null;
        }

        return TimeSpan.FromSeconds(seconds);
    }

    /// <summary>One member of the JSON object a body carries, when it carries one as text.</summary>
    private static string? Member(byte[] payload, string name)
    {
        if (payload.Length > Runtime.MaxPayloadBytes)
        {
            return null;
        }

        try
        {
            using JsonDocument read = JsonDocument.Parse(
                payload,
                new JsonDocumentOptions { MaxDepth = Runtime.MaxPayloadNesting });
            if (read.RootElement.ValueKind != JsonValueKind.Object
                || !read.RootElement.TryGetProperty(name, out JsonElement member)
                || member.ValueKind != JsonValueKind.String)
            {
                return null;
            }

            return member.GetString();
        }
        catch (JsonException)
        {
            return null;
        }
    }

    /// <summary>What one send is about to do, settled before its first attempt.</summary>
    private Sending Begin(Event ingested)
    {
        ArgumentNullException.ThrowIfNull(ingested);

        string eventId = (ingested.EventId ?? NewEventId()).ToString("D", CultureInfo.InvariantCulture);

        int size = System.Text.Encoding.UTF8.GetByteCount(ingested.Payload);
        if (size > Options.MaxPayloadBytes)
        {
            throw SendException.PayloadTooLarge(eventId, size, Options.MaxPayloadBytes);
        }

        RetryPolicy policy = Options.RetryPolicy;
        return new Sending(eventId, Full(ingested, eventId), policy, policy.Delays(Jitter(policy.Attempts - 1)));
    }

    /// <summary>An event as the API reads one.</summary>
    private Dictionary<string, object?> Full(Event ingested, string eventId)
    {
        DateTimeOffset occurredAt = ingested.OccurredAt ?? DateTimeOffset.UtcNow;
        Dictionary<string, object?> body = new(StringComparer.Ordinal)
        {
            ["event_id"] = eventId,
            ["application_id"] = _applicationId,
            ["event_type"] = ingested.EventType,
            ["payload"] = ingested.Payload,
            ["payload_content_type"] = ingested.PayloadContentType,
            ["occurred_at"] = Runtime.Written(occurredAt),
            ["labels"] = ingested.Labels,
        };

        if (ingested.Metadata is not null)
        {
            body["metadata"] = ingested.Metadata;
        }

        return body;
    }

    /// <summary>The randomness used to jitter the delays of one send.</summary>
    /// <remarks>
    /// Jitter only has to keep emitters that failed together from coming back together; it does not
    /// have to be unpredictable, so the platform's own generator is enough.
    /// </remarks>
    private static IReadOnlyList<double> Jitter(int count)
    {
        double[] draws = new double[Math.Max(count, 0)];
        for (int index = 0; index < draws.Length; index++)
        {
            draws[index] = Random.Shared.NextDouble();
        }

        return draws;
    }

    /// <summary>The event types a caller asked for, read and bounded.</summary>
    private static IReadOnlyList<EventType> Wanted(IReadOnlyList<string> eventTypes)
    {
        ArgumentNullException.ThrowIfNull(eventTypes);
        if (eventTypes.Count > MaxUpsertedEventTypes)
        {
            throw EventTypeException.Unavailable(
                string.Create(
                    CultureInfo.InvariantCulture,
                    $"{eventTypes.Count} event types were asked for, above the {MaxUpsertedEventTypes} accepted"));
        }

        List<EventType> wanted = new(eventTypes.Count);
        foreach (string written in eventTypes)
        {
            wanted.Add(EventType.Parse(written));
        }

        return wanted;
    }

    /// <summary>What one event type declaration sends.</summary>
    private Dictionary<string, object?> Declaring(EventType eventType) => new(StringComparer.Ordinal)
    {
        ["application_id"] = _applicationId,
        ["service"] = eventType.Service,
        ["resource_type"] = eventType.ResourceType,
        ["verb"] = eventType.Verb,
    };

    /// <summary>What the API answered a request about event types, or why it answered nothing.</summary>
    private static TransportDelivery Answered(Func<TransportDelivery> issue)
    {
        try
        {
            return issue();
        }
        catch (TransportException failure)
        {
            throw EventTypeException.Unavailable(failure.Message);
        }
    }

    /// <summary>What the API answered a request about event types, or why it answered nothing.</summary>
    private static async Task<TransportDelivery> AnsweredAsync(Func<Task<TransportDelivery>> issue)
    {
        try
        {
            return await issue().ConfigureAwait(false);
        }
        catch (TransportException failure)
        {
            throw EventTypeException.Unavailable(failure.Message);
        }
    }

    /// <summary>The event types an application already declares, out of what the API answered.</summary>
    private static IReadOnlySet<string> Declared(TransportDelivery delivered)
    {
        if (delivered.Status is < LowestSuccess or >= LowestRedirection)
        {
            throw EventTypeException.Unavailable(Runtime.Preview(delivered.Payload));
        }

        HashSet<string> declared = new(StringComparer.Ordinal);
        try
        {
            using JsonDocument read = JsonDocument.Parse(
                delivered.Payload,
                new JsonDocumentOptions { MaxDepth = Runtime.MaxPayloadNesting });
            if (read.RootElement.ValueKind != JsonValueKind.Array)
            {
                throw EventTypeException.Unavailable("the API did not answer a list of event types");
            }

            foreach (JsonElement entry in read.RootElement.EnumerateArray())
            {
                if (entry.ValueKind == JsonValueKind.Object
                    && entry.TryGetProperty("event_type_name", out JsonElement name)
                    && name.ValueKind == JsonValueKind.String)
                {
                    declared.Add(name.GetString() ?? string.Empty);
                }
            }
        }
        catch (JsonException unreadable)
        {
            throw EventTypeException.Unavailable(unreadable.Message);
        }

        return declared;
    }

    /// <summary>Refuses a declaration the API did not accept.</summary>
    private static void Created(EventType eventType, TransportDelivery delivered)
    {
        if (delivered.Status is < LowestSuccess or >= LowestRedirection)
        {
            throw EventTypeException.NotCreated(eventType.Written(), Runtime.Preview(delivered.Payload));
        }
    }

    /// <summary>What one attempt at sending an event ended with.</summary>
    /// <param name="Ingested">The identifier the API says it ingested the event under.</param>
    /// <param name="AlreadyIngested">Whether the API says that identifier is already taken.</param>
    /// <param name="Detail">What to say about the attempt.</param>
    /// <param name="Retryable">Whether repeating the request could end differently.</param>
    /// <param name="NamedDelay">How long the answer said to wait before repeating it, when it said.</param>
    private readonly record struct Attempt(
        string? Ingested,
        bool AlreadyIngested,
        string Detail,
        bool Retryable,
        TimeSpan? NamedDelay);

    /// <summary>What a send does next.</summary>
    /// <param name="Ingested">The identifier to answer, when the send is over and succeeded.</param>
    /// <param name="Wait">How long to wait before the next attempt.</param>
    /// <param name="Failure">What to raise, when the send is over and failed.</param>
    private readonly record struct SendStep(string? Ingested, TimeSpan Wait, SendException? Failure);

    /// <summary>One send in progress: what it is sending, and what it does about each answer.</summary>
    /// <remarks>
    /// Every verdict of a send lives here and nowhere else, so the blocking surface and the awaiting
    /// one cannot come to different conclusions about the same answer: they issue an attempt and
    /// wait differently, and agree about everything in between by construction.
    /// </remarks>
    private sealed class Sending(
        string eventId,
        Dictionary<string, object?> body,
        RetryPolicy policy,
        IReadOnlyList<TimeSpan> delays)
    {
        private readonly RetryPolicy _policy = policy;
        private readonly IReadOnlyList<TimeSpan> _delays = delays;
        private int _issued;
        private TimeSpan _waited = TimeSpan.Zero;
        private string _detail = string.Empty;

        /// <summary>The identifier this send carries, on every attempt it makes.</summary>
        public string EventId { get; } = eventId;

        /// <summary>What the request carries.</summary>
        public Dictionary<string, object?> Body { get; } = body;

        /// <summary>How many attempts this send may make.</summary>
        public int Attempts => _policy.Attempts;

        /// <summary>What to do about what one attempt answered.</summary>
        /// <param name="outcome">What the attempt ended with.</param>
        /// <returns>The identifier to answer, the delay to wait, or the failure to raise.</returns>
        public SendStep Next(Attempt outcome)
        {
            _issued++;
            _detail = outcome.Detail;

            if (outcome.Ingested is not null)
            {
                return new SendStep(outcome.Ingested, TimeSpan.Zero, null);
            }

            // The identifier is already taken. On a repeated request that is this send's own earlier
            // attempt having landed; on the first one it is a genuine conflict.
            if (outcome.AlreadyIngested)
            {
                return _issued > 1
                    ? new SendStep(EventId, TimeSpan.Zero, null)
                    : new SendStep(null, TimeSpan.Zero, SendException.Refused(EventId, outcome.Detail));
            }

            TimeSpan? scheduled = outcome.Retryable && _issued - 1 < _delays.Count
                ? _delays[_issued - 1]
                : null;
            if (scheduled is null)
            {
                return new SendStep(null, TimeSpan.Zero, GaveUp());
            }

            // What the API asked for when it asked for anything, and this client's own schedule
            // otherwise — either way cut down to what is left of the budget every delay of one send
            // shares, so a delay written by the other end cannot stretch a send past what the caller
            // allowed for it.
            double remaining = Math.Max((_policy.MaxTotalDelay - _waited).TotalSeconds, 0);
            double wanted = (outcome.NamedDelay ?? scheduled.Value).TotalSeconds;
            TimeSpan waiting = TimeSpan.FromSeconds(Math.Clamp(wanted, 0, remaining));
            _waited += waiting;
            return new SendStep(null, waiting, null);
        }

        /// <summary>What to raise when this send is being given up on.</summary>
        /// <returns>The failure a caller is handed.</returns>
        public SendException GaveUp() => _issued <= 1
            ? SendException.Refused(EventId, _detail)
            : SendException.GaveUp(EventId, _issued, _waited, _detail);
    }
}
