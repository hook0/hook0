using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Net.Http;
using System.Net.Http.Headers;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
using System.Threading.Tasks;

namespace Hook0;

/// <summary>A request the API never answered, and what caused that.</summary>
/// <remarks>
/// The three causes are told apart because only one of them could end differently. A request that
/// got no answer — a connection refused or reset, an attempt out of time, a body that stopped
/// mid-way — says nothing about whether the API acted on it, which is exactly why a send carries an
/// identifier the client chose itself, and why repeating it is safe and worth doing. An answer that
/// crossed a ceiling this client set for itself draws the same answer the second time, and reading
/// it again four times over costs the caller four times as much for the same failure. A URL nothing
/// can be sent to was never sent at all, and a repetition builds the same unusable request, turning
/// a misconfiguration into a message that accuses the network.
/// <para>
/// The names are the ones the shared conformance corpus gives them, so the verdict a client applies
/// and the verdict that corpus writes down are the same words.
/// </para>
/// </remarks>
public sealed class TransportException : Exception
{
    private TransportException(string detail, string causeName, bool retryable, Exception? cause)
        : base(detail, cause)
    {
        CauseName = causeName;
        Retryable = retryable;
    }

    /// <summary>Which of the corpus's causes this is.</summary>
    public string CauseName { get; }

    /// <summary>Whether repeating the request that met this could end differently.</summary>
    public bool Retryable { get; }

    /// <summary>The API was reached for and answered nothing this client could read to its end.</summary>
    /// <param name="detail">What went wrong, in the words a caller is given.</param>
    /// <param name="cause">What the framework reported, when it reported anything.</param>
    /// <returns>The failure, classified as one a repetition could clear.</returns>
    public static TransportException NoAnswer(string detail, Exception? cause = null) =>
        new(detail, "no_answer", true, cause);

    /// <summary>The API answered, and what it answered crossed a ceiling this client set.</summary>
    /// <param name="detail">What went wrong, in the words a caller is given.</param>
    /// <returns>The failure, classified as one a repetition would meet again.</returns>
    public static TransportException AnswerAboveABound(string detail) =>
        new(detail, "answer_above_a_bound", false, null);

    /// <summary>There is nowhere to send the request, so nothing was sent.</summary>
    /// <param name="detail">What went wrong, in the words a caller is given.</param>
    /// <returns>The failure, classified as one no repetition can clear.</returns>
    public static TransportException UnusableApiUrl(string detail) =>
        new(detail, "unusable_api_url", false, null);
}

/// <summary>How a request reaches the API, and what a server on the other end is not allowed to cost.</summary>
/// <remarks>
/// The transport answers the status and the bytes and knows nothing of what the API declares:
/// reading those bytes is the generated half's job, and deciding whether to send them again is the
/// client's. That is what lets one HTTP implementation serve both the hand-written event path and
/// every generated method, in both idioms: it satisfies <see cref="ITransport"/> and
/// <see cref="IAsyncTransport"/> alike, and the two share every line that is not the call itself.
/// <para>
/// Nothing here reaches for a third-party HTTP library. Everything a server controls is bounded:
/// how long one exchange may take, how long the head of an answer may be, how many header lines it
/// may carry, and how many bytes of body are read off the socket.
/// </para>
/// </remarks>
public sealed class HttpTransport : ITransport, IAsyncTransport, IDisposable
{
    /// <summary>
    /// Longest one attempt at reaching the API is given before it is abandoned.
    /// </summary>
    /// <remarks>
    /// Ten seconds is far above what ingesting an event takes when the API is healthy, and short
    /// enough that a stuck connection does not hold a caller for a noticeable time.
    /// </remarks>
    public static readonly TimeSpan DefaultRequestTimeout = TimeSpan.FromSeconds(10);

    /// <summary>Largest response body read off a socket, in bytes.</summary>
    public const int DefaultMaxResponseBytes = 8 * 1024 * 1024;

    /// <summary>How many header lines an answer may carry before it is refused.</summary>
    /// <remarks>
    /// <see cref="SocketsHttpHandler.MaxResponseHeadersLength"/> bounds how many bytes the head of
    /// an answer may take, and nothing in the framework bounds how many lines it is made of: sixty
    /// thousand empty headers cross no ceiling it applies. So the count is this client's own.
    /// </remarks>
    public const int DefaultMaxResponseHeaders = 64;

    /// <summary>Longest the whole head of an answer may be, in bytes.</summary>
    /// <remarks>
    /// This is the one head bound the framework already applies, so it is set rather than added:
    /// <see cref="SocketsHttpHandler.MaxResponseHeadersLength"/> counts the status line and every
    /// header line together, in kibibytes, and refuses the answer past that. It is the aggregate,
    /// and it is what actually bounds the memory a head can cost — the count and the size per line
    /// below multiply, and sixty-four lines of sixty-four kibibytes is four mebibytes of head that
    /// both of them admit.
    /// <para>
    /// The framework's own default is four times this. It is lowered rather than inherited, so that
    /// two clients differing only in which runtime they happen to run on cannot return opposite
    /// verdicts on the same head.
    /// </para>
    /// </remarks>
    public const int DefaultMaxHeadBytes = 16 * 1024;

    /// <summary>Longest one header line may be, name and value together, in bytes.</summary>
    /// <remarks>
    /// The aggregate above is the ceiling; this and the count earn their place by refusing sooner,
    /// on the first line that is too long rather than once the whole head has been read.
    /// </remarks>
    public const int DefaultMaxHeaderBytes = 64 * 1024;

    /// <summary>What a request body says it carries, and what an answer is asked for in.</summary>
    public const string JsonMediaType = "application/json";

    /// <summary>How many bytes are read off the socket at a time.</summary>
    private const int ChunkBytes = 64 * 1024;

    /// <summary>How many kibibytes one kibibyte is written as, which is the unit the handler takes.</summary>
    private const int BytesPerKibibyte = 1024;

    /// <summary>Longest each part the <c>User-Agent</c> is composed out of may be, in characters.</summary>
    /// <remarks>
    /// The runtime and the operating system are described by the platform rather than by this
    /// assembly, so their length is not this assembly's to guarantee: they are cut here so that the
    /// header cannot grow with whatever the platform feels like saying. Every part is also stripped
    /// of anything the grammar of the header uses as punctuation, so a platform cannot forge a shape
    /// it does not have.
    /// </remarks>
    private const int MaxUserAgentPartChars = 64;

    /// <summary>Which SDK, at which version, on which runtime and operating system, is talking.</summary>
    /// <remarks>
    /// Composed once, since nothing it is built out of changes over the life of a process.
    /// </remarks>
    private static readonly string UserAgent = Composed();

    private readonly Uri _baseUrl;
    private readonly string _token;
    private readonly TimeSpan _timeout;
    private readonly int _maxResponseBytes;
    private readonly int _maxResponseHeaders;
    private readonly int _maxHeaderBytes;
    private readonly HttpClient _http;
    private readonly bool _unusableBaseUrl;
    private readonly string _clientOptions;

    /// <summary>Reaches an API over HTTP, under the bounds the caller sets.</summary>
    /// <param name="baseUrl">Where the API lives, such as <c>https://app.hook0.com/api/v1</c>.</param>
    /// <param name="token">An authentication token valid for that API.</param>
    /// <param name="timeout">How long one attempt is given.</param>
    /// <param name="maxResponseBytes">The largest answer read off a socket.</param>
    /// <param name="maxResponseHeaders">How many header lines an answer may carry.</param>
    /// <param name="maxHeadBytes">How long the whole head of an answer may be.</param>
    /// <param name="maxHeaderBytes">How long one header line of it may be.</param>
    /// <remarks>
    /// A transport built this way reports the default retry schedule, since it is handed no other.
    /// The overload taking a <see cref="RetryPolicy"/> is what a client that retries calls.
    /// </remarks>
    public HttpTransport(
        string baseUrl,
        string token,
        TimeSpan? timeout = null,
        int maxResponseBytes = DefaultMaxResponseBytes,
        int maxResponseHeaders = DefaultMaxResponseHeaders,
        int maxHeadBytes = DefaultMaxHeadBytes,
        int maxHeaderBytes = DefaultMaxHeaderBytes)
        : this(
            baseUrl,
            token,
            timeout,
            maxResponseBytes,
            maxResponseHeaders,
            maxHeadBytes,
            maxHeaderBytes,
            new RetryPolicy())
    {
    }

    /// <summary>Reaches an API over HTTP, stating the schedule its caller retries on.</summary>
    /// <param name="baseUrl">Where the API lives, such as <c>https://app.hook0.com/api/v1</c>.</param>
    /// <param name="token">An authentication token valid for that API.</param>
    /// <param name="timeout">How long one attempt is given.</param>
    /// <param name="maxResponseBytes">The largest answer read off a socket.</param>
    /// <param name="maxResponseHeaders">How many header lines an answer may carry.</param>
    /// <param name="maxHeadBytes">How long the whole head of an answer may be.</param>
    /// <param name="maxHeaderBytes">How long one header line of it may be.</param>
    /// <param name="retryPolicy">The schedule the requests of one send are spaced out by.</param>
    /// <remarks>
    /// This is an overload rather than one more optional argument on the constructor above, and
    /// that is not a matter of taste. C# resolves optional arguments at the call site, so a caller
    /// compiled against the published package carries a call to the arity it saw. Growing that
    /// constructor by one parameter deletes the signature they call and hands them a
    /// <c>MissingMethodException</c> at run time, in code they never touched and a build that
    /// never went red.
    /// <para>
    /// Every parameter here is required, and the schedule is last, so that the two constructors can
    /// never both apply to one call. A schedule sitting earlier, among the optional arguments, would
    /// leave <c>new HttpTransport(url, token, null)</c> ambiguous between a timeout and a policy —
    /// which is a break too, a smaller one that surfaces at a caller's next build rather than in
    /// production, and one there is no reason to leave lying around.
    /// </para>
    /// </remarks>
    public HttpTransport(
        string baseUrl,
        string token,
        TimeSpan? timeout,
        int maxResponseBytes,
        int maxResponseHeaders,
        int maxHeadBytes,
        int maxHeaderBytes,
        RetryPolicy retryPolicy)
    {
        _token = token;
        _timeout = timeout ?? DefaultRequestTimeout;
        _maxResponseBytes = maxResponseBytes;
        _maxResponseHeaders = maxResponseHeaders;
        _maxHeaderBytes = maxHeaderBytes;
        _clientOptions = Stated(retryPolicy);

        // A base URL nothing can be sent to is remembered rather than thrown from a constructor: it
        // is a request that was never issued, and the corpus classifies it as one.
        _unusableBaseUrl = !Reachable(baseUrl, out Uri? reached);
        _baseUrl = reached ?? new Uri("http://unusable.invalid/", UriKind.Absolute);

        _http = new HttpClient(
            new SocketsHttpHandler
            {
                MaxResponseHeadersLength = Math.Max(1, maxHeadBytes / BytesPerKibibyte),
                AllowAutoRedirect = false,
            },
            disposeHandler: true)
        {
            // The whole exchange is bounded by a token this class owns rather than by the client's
            // own timeout: the framework's stops at the head of the answer when the body is read as
            // a stream, and a server that stalls mid-body would otherwise never be given up on.
            Timeout = Timeout.InfiniteTimeSpan,
        };
    }

    /// <inheritdoc />
    public TransportAnswer Request(
        string method,
        string path,
        IReadOnlyList<KeyValuePair<string, string>> query,
        object? body)
    {
        TransportDelivery delivered = Deliver(method, path, query, body);
        return new TransportAnswer(delivered.Status, delivered.Payload);
    }

    /// <inheritdoc />
    public async Task<TransportAnswer> RequestAsync(
        string method,
        string path,
        IReadOnlyList<KeyValuePair<string, string>> query,
        object? body,
        CancellationToken cancellationToken)
    {
        TransportDelivery delivered = await DeliverAsync(method, path, query, body, cancellationToken)
            .ConfigureAwait(false);
        return new TransportAnswer(delivered.Status, delivered.Payload);
    }

    /// <summary>What the API answered, headers included, whether or not it answered a success.</summary>
    /// <param name="method">The HTTP method the operation is issued under.</param>
    /// <param name="path">Where the request lands, absolute or under the base URL.</param>
    /// <param name="query">The name and value pairs of the query string.</param>
    /// <param name="body">What to send as a JSON document, or nothing at all.</param>
    /// <returns>The status, the headers and the body.</returns>
    public TransportDelivery Deliver(
        string method,
        string path,
        IReadOnlyList<KeyValuePair<string, string>> query,
        object? body)
    {
        using HttpRequestMessage request = Built(method, path, query, body);
        using CancellationTokenSource abandoned = new(_timeout);

        try
        {
            using HttpResponseMessage answer = _http.Send(
                request,
                HttpCompletionOption.ResponseHeadersRead,
                abandoned.Token);
            using Stream reading = answer.Content.ReadAsStream(abandoned.Token);
            return new TransportDelivery(
                (int)answer.StatusCode,
                Carried(answer),
                Bounded(reading, abandoned.Token));
        }
        catch (Exception failure) when (Unreachable(failure, CancellationToken.None))
        {
            throw Classified(failure);
        }
    }

    /// <summary>What the API answered, headers included, whether or not it answered a success.</summary>
    /// <param name="method">The HTTP method the operation is issued under.</param>
    /// <param name="path">Where the request lands, absolute or under the base URL.</param>
    /// <param name="query">The name and value pairs of the query string.</param>
    /// <param name="body">What to send as a JSON document, or nothing at all.</param>
    /// <param name="cancellationToken">What abandons the request before it is answered.</param>
    /// <returns>The status, the headers and the body.</returns>
    public async Task<TransportDelivery> DeliverAsync(
        string method,
        string path,
        IReadOnlyList<KeyValuePair<string, string>> query,
        object? body,
        CancellationToken cancellationToken)
    {
        using HttpRequestMessage request = Built(method, path, query, body);
        using CancellationTokenSource abandoned =
            CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
        abandoned.CancelAfter(_timeout);

        try
        {
            using HttpResponseMessage answer = await _http
                .SendAsync(request, HttpCompletionOption.ResponseHeadersRead, abandoned.Token)
                .ConfigureAwait(false);
            using Stream reading = await answer.Content
                .ReadAsStreamAsync(abandoned.Token)
                .ConfigureAwait(false);
            return new TransportDelivery(
                (int)answer.StatusCode,
                Carried(answer),
                await BoundedAsync(reading, abandoned.Token).ConfigureAwait(false));
        }
        catch (Exception failure) when (Unreachable(failure, cancellationToken))
        {
            throw Classified(failure);
        }
    }

    /// <inheritdoc />
    public void Dispose() => _http.Dispose();

    /// <summary>Whether that URL is somewhere a request can be sent at all.</summary>
    private static bool Reachable(string baseUrl, out Uri? reached)
    {
        reached = null;
        if (!Uri.TryCreate(Ended(baseUrl), UriKind.Absolute, out Uri? built))
        {
            return false;
        }

        if (built.Scheme != Uri.UriSchemeHttp && built.Scheme != Uri.UriSchemeHttps)
        {
            return false;
        }

        if (string.IsNullOrEmpty(built.Host))
        {
            return false;
        }

        reached = built;
        return true;
    }

    /// <summary>A base URL a relative path extends rather than replaces.</summary>
    private static string Ended(string baseUrl) =>
        baseUrl.EndsWith('/') ? baseUrl : baseUrl + "/";

    /// <summary>Which failures are a request that never produced an answer to read.</summary>
    /// <remarks>
    /// A caller that abandoned its own request is not one of them: that cancellation travels back
    /// untouched, so a client never reports the caller's own decision as the network failing.
    /// </remarks>
    private static bool Unreachable(Exception failure, CancellationToken cancellationToken)
    {
        if (cancellationToken.IsCancellationRequested && failure is OperationCanceledException)
        {
            return false;
        }

        return failure is HttpRequestException or OperationCanceledException or IOException;
    }

    /// <summary>What kind of failure that is, by its nature rather than by the type carrying it.</summary>
    private static TransportException Classified(Exception failure)
    {
        // The head of an answer above what the handler holds is the one failure of this shape that
        // is not the network: the same request draws the same oversized head the second time.
        if (failure is HttpRequestException refused
            && refused.HttpRequestError == HttpRequestError.ConfigurationLimitExceeded)
        {
            return TransportException.AnswerAboveABound(refused.Message);
        }

        if (failure is OperationCanceledException)
        {
            return TransportException.NoAnswer(
                $"the API did not answer within {failure.Message}",
                failure);
        }

        return TransportException.NoAnswer(failure.Message, failure);
    }

    /// <summary>What an answer carried beside its body, under the names a caller looks them up by.</summary>
    /// <remarks>
    /// Read before the body is, so an abusive head costs one pass over what has already arrived
    /// rather than that plus a megabyte-scale body on top.
    /// </remarks>
    private Dictionary<string, string> Carried(HttpResponseMessage answer)
    {
        Dictionary<string, string> carried = new(StringComparer.OrdinalIgnoreCase);
        int held = 0;

        foreach (KeyValuePair<string, IEnumerable<string>> header in Lines(answer))
        {
            held++;
            if (held > _maxResponseHeaders)
            {
                throw TransportException.AnswerAboveABound(
                    $"the API answered more than the {_maxResponseHeaders} header lines read at most");
            }

            string value = string.Join(", ", header.Value).Trim();
            int line = Encoding.UTF8.GetByteCount(header.Key) + Encoding.UTF8.GetByteCount(value);
            if (line > _maxHeaderBytes)
            {
                throw TransportException.AnswerAboveABound(
                    $"the API answered a `{header.Key.ToLowerInvariant()}` header above the " +
                    $"{_maxHeaderBytes} bytes read at most");
            }

            carried[header.Key] = value;
        }

        return carried;
    }

    /// <summary>Every header line of an answer, whichever half of the message it belongs to.</summary>
    private static IEnumerable<KeyValuePair<string, IEnumerable<string>>> Lines(
        HttpResponseMessage answer)
    {
        foreach (KeyValuePair<string, IEnumerable<string>> header in answer.Headers)
        {
            yield return header;
        }

        foreach (KeyValuePair<string, IEnumerable<string>> header in answer.Content.Headers)
        {
            yield return header;
        }
    }

    /// <summary>One request, as the framework carries it.</summary>
    private HttpRequestMessage Built(
        string method,
        string path,
        IReadOnlyList<KeyValuePair<string, string>> query,
        object? body)
    {
        HttpRequestMessage request = new(new HttpMethod(method), Resolved(path, query));
        request.Headers.TryAddWithoutValidation("Authorization", $"Bearer {_token}");
        request.Headers.TryAddWithoutValidation("Accept", JsonMediaType);
        request.Headers.TryAddWithoutValidation("User-Agent", UserAgent);
        request.Headers.TryAddWithoutValidation("Hook0-Client-Options", _clientOptions);

        if (body is not null)
        {
            ByteArrayContent content = new(Runtime.Write(body));
            content.Headers.ContentType = new MediaTypeHeaderValue(JsonMediaType);
            request.Content = content;
        }

        return request;
    }

    /// <summary>How this client names itself, out of the assembly and out of the platform.</summary>
    /// <remarks>
    /// The version is the one the build stamped on this assembly rather than one written down again
    /// here: one remembered in two places is one that will disagree with itself the first time it is
    /// bumped. The stamp carries the commit beside the version, which says nothing to an API reading
    /// the header and is cut off.
    /// </remarks>
    private static string Composed()
    {
        string stamped = typeof(HttpTransport).Assembly
            .GetCustomAttribute<AssemblyInformationalVersionAttribute>()
            ?.InformationalVersion ?? string.Empty;
        int built = stamped.IndexOf('+', StringComparison.Ordinal);

        return $"hook0-client-csharp/{Clipped(built < 0 ? stamped : stamped[..built])} " +
            $"({Clipped(RuntimeInformation.FrameworkDescription)}; " +
            $"{Clipped(RuntimeInformation.RuntimeIdentifier)})";
    }

    /// <summary>The schedule a transport was built with, in the shape an API reads it off the wire.</summary>
    /// <remarks>
    /// The grammar is the one <c>X-Hook0-Signature</c> already uses — parts joined by <c>,</c>, each
    /// cut at its first <c>=</c> — so this header costs no parser that is not written twice over
    /// already.
    /// <para>
    /// What is stated is the policy in force, not what a send went on to do with it: a policy
    /// allowing one attempt still states its delays, because they are what it holds, and an API
    /// reading <c>attempts=1</c> already knows none of them will be waited. In force is also after
    /// this client's own clamps — <see cref="RetryPolicy.Attempts"/> rather than
    /// <see cref="RetryPolicy.MaxAttempts"/> — since the capped number is the one the API's traffic
    /// will show, and a thousand would send a reader looking for a burst that cannot happen.
    /// </para>
    /// </remarks>
    private static string Stated(RetryPolicy policy)
    {
        long backoff = Millis(policy.InitialBackoff);
        long ceiling = Millis(policy.MaxBackoff);
        long budget = Millis(policy.MaxTotalDelay);

        return string.Create(
            CultureInfo.InvariantCulture,
            $"attempts={policy.Attempts},backoff={backoff},ceiling={ceiling},budget={budget}");
    }

    /// <summary>One delay of a policy, in the whole milliseconds the header states it in.</summary>
    /// <remarks>
    /// Counted off the ticks rather than read from <see cref="TimeSpan.TotalMilliseconds"/>, which
    /// is a <c>double</c>: a whole number the header states has no business being rounded on its way
    /// there, and a count of ticks divided down can hold no value a <c>long</c> cannot. A negative
    /// delay is stated as zero, which is what the policy itself would wait for one.
    /// </remarks>
    private static long Millis(TimeSpan held) =>
        held.Ticks > 0 ? held.Ticks / TimeSpan.TicksPerMillisecond : 0;

    /// <summary>
    /// One part of the <c>User-Agent</c>, with everything the header's own grammar uses taken out of
    /// it and cut to <see cref="MaxUserAgentPartChars"/>.
    /// </summary>
    private static string Clipped(string part)
    {
        StringBuilder kept = new(MaxUserAgentPartChars);

        foreach (char one in part)
        {
            if (one is < ' ' or > '~' or '(' or ')' or ';')
            {
                continue;
            }

            kept.Append(one);
            if (kept.Length == MaxUserAgentPartChars)
            {
                break;
            }
        }

        return kept.ToString();
    }

    /// <summary>Where a request lands: a path of its own replaces the base's, a relative one extends it.</summary>
    private Uri Resolved(string path, IReadOnlyList<KeyValuePair<string, string>> query)
    {
        if (_unusableBaseUrl)
        {
            throw TransportException.UnusableApiUrl(
                "the API URL is not somewhere this transport can send a request");
        }

        if (!Uri.TryCreate(_baseUrl, path, out Uri? target))
        {
            throw TransportException.UnusableApiUrl(
                $"`{path}` is not somewhere this transport can send a request");
        }

        if (query.Count == 0)
        {
            return target;
        }

        UriBuilder written = new(target);
        List<string> pairs = new(query.Count);
        foreach (KeyValuePair<string, string> pair in query)
        {
            pairs.Add(
                $"{Uri.EscapeDataString(pair.Key)}={Uri.EscapeDataString(pair.Value)}");
        }

        string joined = string.Join("&", pairs);
        written.Query = string.IsNullOrEmpty(written.Query)
            ? joined
            : $"{written.Query.TrimStart('?')}&{joined}";
        return written.Uri;
    }

    /// <summary>The body of an answer, up to what this transport agrees to hold.</summary>
    private byte[] Bounded(Stream reading, CancellationToken cancellationToken)
    {
        using MemoryStream held = new();
        byte[] chunk = new byte[ChunkBytes];

        for (int read = 0; read <= Chunks(); read++)
        {
            cancellationToken.ThrowIfCancellationRequested();
            int taken = reading.Read(chunk, 0, chunk.Length);
            if (taken == 0)
            {
                return held.ToArray();
            }

            Fits(held.Length + taken);
            held.Write(chunk, 0, taken);
        }

        throw TransportException.AnswerAboveABound(
            $"the API answered more than the {_maxResponseBytes} bytes read at most");
    }

    /// <summary>The body of an answer, up to what this transport agrees to hold.</summary>
    private async Task<byte[]> BoundedAsync(Stream reading, CancellationToken cancellationToken)
    {
        using MemoryStream held = new();
        byte[] chunk = new byte[ChunkBytes];

        for (int read = 0; read <= Chunks(); read++)
        {
            int taken = await reading.ReadAsync(chunk, cancellationToken).ConfigureAwait(false);
            if (taken == 0)
            {
                return held.ToArray();
            }

            Fits(held.Length + taken);
            await held.WriteAsync(chunk.AsMemory(0, taken), cancellationToken).ConfigureAwait(false);
        }

        throw TransportException.AnswerAboveABound(
            $"the API answered more than the {_maxResponseBytes} bytes read at most");
    }

    /// <summary>How many reads it takes to reach the ceiling, which is what bounds the loop.</summary>
    private int Chunks() => (_maxResponseBytes / ChunkBytes) + 1;

    /// <summary>Refuses a body that has grown past what this transport agreed to hold.</summary>
    private void Fits(long held)
    {
        if (held > _maxResponseBytes)
        {
            throw TransportException.AnswerAboveABound(
                string.Create(
                    CultureInfo.InvariantCulture,
                    $"the API answered more than the {_maxResponseBytes} bytes read at most"));
        }
    }
}
