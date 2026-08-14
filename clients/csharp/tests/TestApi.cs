// A Hook0 API on a loopback port, and what every case is written against.
//
// Every case below goes over a real socket: the request the client builds, the headers it sets, the
// way it reads an answer and the way it gives up on one are all the real ones. Nothing here stands
// in for a part of the client, so a case that passes says the client works rather than that it was
// called. In particular no `HttpMessageHandler` is replaced: the transport talks to this server the
// way it talks to Hook0.
//
// The server is a plain `TcpListener` speaking as much HTTP/1.1 as one exchange needs, which is what
// lets a case hold an answer past a timeout, answer more header lines than a client reads, or answer
// a body above what it agreed to hold — none of which a higher-level listener would allow.

using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Net;
using System.Net.Sockets;
using System.Text;
using System.Text.Json;
using System.Text.Json.Nodes;
using System.Threading;
using System.Threading.Tasks;

namespace Hook0.Tests;

/// <summary>What the API answers to one request, in the order the case scripted it.</summary>
public sealed record ScriptedResponse(
    int Status,
    JsonNode? Body,
    TimeSpan HeldFor = default,
    IReadOnlyList<KeyValuePair<string, string>>? Headers = null);

/// <summary>A request the API received, in the order it received it.</summary>
public sealed record ReceivedRequest(
    string Verb,
    string Target,
    IReadOnlyList<KeyValuePair<string, string>> Headers,
    string Body)
{
    /// <summary>What the request carried, read as the JSON document it is.</summary>
    public JsonNode Json() => JsonNode.Parse(Body) ?? throw new InvalidOperationException("no body");
}

/// <summary>A Hook0 API listening on a loopback port for the lifetime of one case.</summary>
public sealed class FakeApi : IDisposable
{
    /// <summary>Most connections one case opens, which bounds what the server holds at once.</summary>
    public const int MaxConnections = 64;

    /// <summary>No request a case makes is anywhere near this large.</summary>
    private const int MaxRequestBodyBytes = 64 * 1024;

    /// <summary>Longest request line or header line the server reads.</summary>
    private const int MaxLineBytes = 8 * 1024;

    /// <summary>Most header lines the server reads out of one request.</summary>
    private const int MaxHeaders = 64;

    /// <summary>No case talks to anything but a loopback socket, so none takes this long.</summary>
    private static readonly TimeSpan Patience = TimeSpan.FromSeconds(20);

    private readonly TcpListener _listener;
    private readonly List<ReceivedRequest> _received = [];
    private readonly List<ScriptedResponse> _scripted = [];
    private readonly object _held = new();
    private readonly CancellationTokenSource _closing = new();
    private readonly Task _serving;
    private int _answered;

    /// <summary>Starts an API on a loopback port of the operating system's choosing.</summary>
    public FakeApi()
    {
        _listener = new TcpListener(IPAddress.Loopback, 0);
        _listener.Start();
        BaseUrl = string.Create(
            CultureInfo.InvariantCulture,
            $"http://127.0.0.1:{((IPEndPoint)_listener.LocalEndpoint).Port}");
        _serving = Task.Run(ServeAsync);
    }

    /// <summary>Where the client reaches this API.</summary>
    public string BaseUrl { get; }

    /// <summary>The requests this API received, in the order it received them.</summary>
    public IReadOnlyList<ReceivedRequest> Received
    {
        get
        {
            lock (_held)
            {
                return [.. _received];
            }
        }
    }

    /// <summary>Queues the answers the case expects the client to draw, in order.</summary>
    /// <param name="responses">What to answer, one per request.</param>
    public void WillAnswer(params ScriptedResponse[] responses)
    {
        lock (_held)
        {
            _scripted.AddRange(responses);
        }
    }

    /// <inheritdoc />
    public void Dispose()
    {
        _closing.Cancel();
        _listener.Stop();
        _serving.Wait(Patience);
        _closing.Dispose();
    }

    private async Task ServeAsync()
    {
        List<Task> connections = new(MaxConnections);

        for (int opened = 0; opened < MaxConnections; opened++)
        {
            TcpClient socket;
            try
            {
                socket = await _listener.AcceptTcpClientAsync(_closing.Token).ConfigureAwait(false);
            }
            catch (Exception closed) when (closed is OperationCanceledException
                or SocketException
                or ObjectDisposedException
                or InvalidOperationException)
            {
                break;
            }

            connections.Add(Task.Run(() => AnswerAsync(socket)));
        }

        await Task.WhenAll(connections).ConfigureAwait(false);
    }

    private async Task AnswerAsync(TcpClient socket)
    {
        using (socket)
        {
            try
            {
                await ExchangeAsync(socket).ConfigureAwait(false);
            }
            catch (Exception gone) when (gone is IOException
                or SocketException
                or ObjectDisposedException
                or OperationCanceledException)
            {
                // The client gave up waiting and closed the connection, which is the very thing a
                // held answer is scripted to make it do.
            }
        }
    }

    private async Task ExchangeAsync(TcpClient socket)
    {
        NetworkStream talking = socket.GetStream();
        ReceivedRequest request = await ReadAsync(talking).ConfigureAwait(false);

        ScriptedResponse scripted;
        lock (_held)
        {
            _received.Add(request);
            scripted = _answered < _scripted.Count
                ? _scripted[_answered]
                : new ScriptedResponse(500, Problem("TheCaseScriptedNoAnswerForThisRequest", 500));
            _answered++;
        }

        if (scripted.HeldFor > TimeSpan.Zero)
        {
            await Task.Delay(scripted.HeldFor, _closing.Token).ConfigureAwait(false);
        }

        byte[] answer = Encoding.UTF8.GetBytes(
            scripted.Body is null ? string.Empty : scripted.Body.ToJsonString());
        StringBuilder head = new();
        head.Append(CultureInfo.InvariantCulture, $"HTTP/1.1 {scripted.Status} Answer\r\n");
        head.Append("Content-Type: application/json\r\n");
        head.Append(CultureInfo.InvariantCulture, $"Content-Length: {answer.Length}\r\n");
        foreach (KeyValuePair<string, string> header in scripted.Headers ?? [])
        {
            head.Append(CultureInfo.InvariantCulture, $"{header.Key}: {header.Value}\r\n");
        }

        head.Append("Connection: close\r\n\r\n");

        await talking.WriteAsync(Encoding.UTF8.GetBytes(head.ToString())).ConfigureAwait(false);
        await talking.WriteAsync(answer).ConfigureAwait(false);
        await talking.FlushAsync().ConfigureAwait(false);
        socket.Client.Shutdown(SocketShutdown.Send);
    }

    private static async Task<ReceivedRequest> ReadAsync(NetworkStream talking)
    {
        string[] opening = (await LineAsync(talking).ConfigureAwait(false)).Split(' ');
        List<KeyValuePair<string, string>> headers = new(MaxHeaders);
        int length = 0;

        for (int read = 0; read < MaxHeaders; read++)
        {
            string line = (await LineAsync(talking).ConfigureAwait(false)).Trim();
            if (line.Length == 0)
            {
                break;
            }

            int assigned = line.IndexOf(':', StringComparison.Ordinal);
            if (assigned < 0)
            {
                continue;
            }

            string name = line[..assigned].Trim();
            string value = line[(assigned + 1)..].Trim();
            headers.Add(new KeyValuePair<string, string>(name, value));
            if (string.Equals(name, "content-length", StringComparison.OrdinalIgnoreCase))
            {
                length = int.Parse(value, CultureInfo.InvariantCulture);
            }
        }

        if (length > MaxRequestBodyBytes)
        {
            throw new IOException($"a case sent more than {MaxRequestBodyBytes} bytes");
        }

        byte[] body = new byte[length];
        int taken = 0;
        while (taken < length)
        {
            int held = await talking.ReadAsync(body.AsMemory(taken)).ConfigureAwait(false);
            if (held == 0)
            {
                throw new IOException("the connection closed mid-request");
            }

            taken += held;
        }

        return new ReceivedRequest(
            opening.Length > 0 ? opening[0] : string.Empty,
            opening.Length > 1 ? opening[1] : string.Empty,
            headers,
            Encoding.UTF8.GetString(body));
    }

    private static async Task<string> LineAsync(NetworkStream talking)
    {
        byte[] one = new byte[1];
        StringBuilder line = new();

        for (int read = 0; read < MaxLineBytes; read++)
        {
            int held = await talking.ReadAsync(one.AsMemory(0, 1)).ConfigureAwait(false);
            if (held == 0)
            {
                throw new IOException("the connection closed mid-request");
            }

            if (one[0] == (byte)'\n')
            {
                return line.ToString();
            }

            if (one[0] != (byte)'\r')
            {
                line.Append((char)one[0]);
            }
        }

        throw new IOException($"a case sent a line above the {MaxLineBytes} bytes read");
    }

    /// <summary>What the API says when it refuses a request, in the shape every Hook0 failure takes.</summary>
    /// <param name="problem">Which problem it names.</param>
    /// <param name="status">What it answers under.</param>
    /// <returns>The document the API answers.</returns>
    public static JsonNode Problem(string problem, int status) => new JsonObject
    {
        ["id"] = problem,
        ["status"] = status,
        ["title"] = "refused",
        ["detail"] = "what the corpus scripted",
        ["type"] = $"https://hook0.com/documentation/errors/{problem}",
    };
}

/// <summary>The shared contract every SDK is held to, and the counter-examples kept beside it.</summary>
public static class Corpus
{
    /// <summary>
    /// Largest document of the corpus read back. The corpus is committed, so one above this is one
    /// that grew out of shape rather than one somebody meant.
    /// </summary>
    public const int MaxCorpusBytes = 512 * 1024;

    /// <summary>How far up the tree the corpus is looked for before the search gives up.</summary>
    private const int MaxLevels = 10;

    /// <summary>One document of the shared contract, bounded before it is parsed.</summary>
    /// <param name="name">Its file name.</param>
    /// <returns>What it says.</returns>
    public static JsonNode Contract(string name) => Read(Path.Combine(Directory(), name));

    /// <summary>The counter-examples worth keeping, committed beside the properties they broke.</summary>
    /// <remarks>
    /// One JSON value per line, so that a header carrying a comma, a newline or nothing at all is
    /// read back exactly as it was written down.
    /// </remarks>
    /// <param name="name">Which corpus to read.</param>
    /// <returns>Every counter-example it carries.</returns>
    public static IReadOnlyList<JsonNode> Regressions(string name)
    {
        string path = Path.Combine(Beside("regressions"), $"{name}.jsonl");
        Bounded(path);

        List<JsonNode> kept = [];
        foreach (string line in File.ReadLines(path))
        {
            if (line.Trim().Length == 0)
            {
                continue;
            }

            kept.Add(JsonNode.Parse(line) ?? throw new InvalidOperationException($"`{line}` is nothing"));
        }

        return kept;
    }

    /// <summary>Where the shared contract sits, found by walking up from where the suite runs.</summary>
    private static string Directory()
    {
        DirectoryInfo? walked = new(AppContext.BaseDirectory);
        for (int level = 0; level < MaxLevels && walked is not null; level++)
        {
            string candidate = Path.Combine(walked.FullName, "clients", "conformance");
            if (File.Exists(Path.Combine(candidate, "bounds.json")))
            {
                return candidate;
            }

            walked = walked.Parent;
        }

        throw new FileNotFoundException(
            $"no `clients/conformance` sits within {MaxLevels} levels of {AppContext.BaseDirectory}");
    }

    /// <summary>Where the counter-examples of this suite sit.</summary>
    private static string Beside(string name)
    {
        DirectoryInfo? walked = new(AppContext.BaseDirectory);
        for (int level = 0; level < MaxLevels && walked is not null; level++)
        {
            string candidate = Path.Combine(walked.FullName, name);
            if (System.IO.Directory.Exists(candidate) && walked.Name == "tests")
            {
                return candidate;
            }

            walked = walked.Parent;
        }

        throw new DirectoryNotFoundException(
            $"no `{name}` sits within {MaxLevels} levels of {AppContext.BaseDirectory}");
    }

    private static JsonNode Read(string path)
    {
        Bounded(path);
        return JsonNode.Parse(File.ReadAllText(path))
            ?? throw new InvalidOperationException($"{path} carries no document");
    }

    private static void Bounded(string path)
    {
        long size = new FileInfo(path).Length;
        if (size > MaxCorpusBytes)
        {
            throw new InvalidOperationException(
                $"{path} is {size} bytes long, above the {MaxCorpusBytes} read back");
        }
    }
}

/// <summary>What every case that talks to an API is built on.</summary>
public abstract class ApiCase : IDisposable
{
    /// <summary>
    /// A schedule short enough that a case spends its time on requests rather than on waiting. Its
    /// budget sits far above what its delays add up to, so the number of attempts a case observes is
    /// the one its policy asked for rather than the one its budget allowed.
    /// </summary>
    protected static readonly TimeSpan PromptBackoff = TimeSpan.FromMilliseconds(5);

    private FakeApi _api = new();

    /// <summary>The API this case is written against.</summary>
    protected FakeApi Api => _api;

    /// <inheritdoc />
    public void Dispose()
    {
        _api.Dispose();
        GC.SuppressFinalize(this);
    }

    /// <summary>Starts this case over against an API of its own.</summary>
    /// <remarks>
    /// One API per case rather than one per suite: what is counted is what one send issued, and a
    /// count carried over from the case before it would say nothing.
    /// </remarks>
    protected void Restarted()
    {
        _api.Dispose();
        _api = new FakeApi();
    }

    /// <summary>The bounds a case holds a send to.</summary>
    /// <param name="maxAttempts">How many attempts one send may make.</param>
    /// <param name="requestTimeout">How long one attempt is given.</param>
    /// <returns>The options a client is built with.</returns>
    protected static ClientOptions Options(int maxAttempts = 4, double requestTimeout = 5) =>
        new()
        {
            RetryPolicy = new RetryPolicy
            {
                MaxAttempts = maxAttempts,
                InitialBackoff = PromptBackoff,
                MaxBackoff = PromptBackoff,
                MaxTotalDelay = TimeSpan.FromSeconds(1),
            },
            RequestTimeout = TimeSpan.FromSeconds(requestTimeout),
        };

    /// <summary>A client reaching the API of this case.</summary>
    /// <param name="options">The bounds one send is held to.</param>
    /// <returns>The client under test.</returns>
    protected Hook0Client Client(ClientOptions? options = null) =>
        new(Api.BaseUrl, "app-123", "token-xyz", options ?? Options());

    /// <summary>An event a case sends.</summary>
    /// <param name="eventId">What to key it on, when the case keys it itself.</param>
    /// <param name="payload">What it carries.</param>
    /// <returns>The event under test.</returns>
    protected static Event AnEvent(Guid? eventId = null, string? payload = null) => new()
    {
        EventType = "auth.user.create",
        Payload = payload ?? "{\"email\": \"test@example.com\"}",
        PayloadContentType = "application/json",
        Labels = new Dictionary<string, string>(StringComparer.Ordinal) { ["environment"] = "production" },
        EventId = eventId,
    };

    /// <summary>What the API answers when it took the event.</summary>
    /// <param name="eventId">What it says it keyed the event on.</param>
    /// <returns>The scripted answer.</returns>
    protected static ScriptedResponse Ingested(string eventId) => new(
        201,
        new JsonObject
        {
            ["application_id"] = "app-123",
            ["event_id"] = eventId,
            ["received_at"] = "2026-01-01T00:00:00Z",
        });

    /// <summary>What the API answers when the identifier a request carries is already taken.</summary>
    /// <returns>The scripted answer.</returns>
    protected static ScriptedResponse AlreadyIngested() =>
        new(409, FakeApi.Problem(Hook0Client.AlreadyIngested, 409));

    /// <summary>What the API answers when it failed on its own side.</summary>
    /// <returns>The scripted answer.</returns>
    protected static ScriptedResponse ServerError() =>
        new(500, FakeApi.Problem("InternalServerError", 500));

    /// <summary>What the API answers when it refuses a request.</summary>
    /// <param name="status">What it answers under.</param>
    /// <param name="problem">Which problem it names.</param>
    /// <param name="headers">What it carries beside its body.</param>
    /// <returns>The scripted answer.</returns>
    protected static ScriptedResponse Refusal(
        int status,
        string problem,
        IReadOnlyList<KeyValuePair<string, string>>? headers = null) =>
        new(status, FakeApi.Problem(problem, status), TimeSpan.Zero, headers);

    /// <summary>What one member of a document says, as text.</summary>
    /// <param name="document">What to read.</param>
    /// <param name="name">Which member.</param>
    /// <returns>What it carries.</returns>
    protected static string Text(JsonNode document, string name) =>
        document[name]?.GetValue<string>() ?? string.Empty;

    /// <summary>What one member of a document says, as a whole number.</summary>
    /// <param name="document">What to read.</param>
    /// <param name="name">Which member.</param>
    /// <returns>What it carries.</returns>
    protected static int Number(JsonNode document, string name) =>
        document[name]?.GetValue<int>() ?? 0;

    /// <summary>Whether one member of a document says yes.</summary>
    /// <param name="document">What to read.</param>
    /// <param name="name">Which member.</param>
    /// <returns>What it carries.</returns>
    protected static bool Flag(JsonNode document, string name) =>
        document[name]?.GetValue<bool>() ?? false;
}
