using System;
using System.Collections.Generic;
using System.Globalization;
using System.Text;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;

namespace Hook0;

/// <summary>What the API answered one request: the status, and the bytes.</summary>
/// <param name="Status">What the API answered under.</param>
/// <param name="Payload">The body it answered, bounded by what the transport agreed to read.</param>
public readonly record struct TransportAnswer(int Status, byte[] Payload);

/// <summary>What the API answered one request, headers included.</summary>
/// <param name="Status">What the API answered under.</param>
/// <param name="Headers">
/// What it answered beside its body, under lowercased names so a caller reads a header without
/// knowing which case the server wrote it in.
/// </param>
/// <param name="Payload">The body it answered.</param>
public readonly record struct TransportDelivery(
    int Status,
    IReadOnlyDictionary<string, string> Headers,
    byte[] Payload);

/// <summary>What a generated method waits on to issue one request.</summary>
public interface ITransport
{
    /// <summary>Issues one request and waits for the answer.</summary>
    /// <param name="method">The HTTP method the operation is issued under.</param>
    /// <param name="path">Where the request lands, absolute or under the base URL.</param>
    /// <param name="query">The name and value pairs of the query string.</param>
    /// <param name="body">What to send as a JSON document, or nothing at all.</param>
    /// <returns>The status and the body the API answered.</returns>
    TransportAnswer Request(
        string method,
        string path,
        IReadOnlyList<KeyValuePair<string, string>> query,
        object? body);
}

/// <summary>What a generated method awaits to issue one request.</summary>
public interface IAsyncTransport
{
    /// <summary>Issues one request and awaits the answer.</summary>
    /// <param name="method">The HTTP method the operation is issued under.</param>
    /// <param name="path">Where the request lands, absolute or under the base URL.</param>
    /// <param name="query">The name and value pairs of the query string.</param>
    /// <param name="body">What to send as a JSON document, or nothing at all.</param>
    /// <param name="cancellationToken">What abandons the request before it is answered.</param>
    /// <returns>The status and the body the API answered.</returns>
    Task<TransportAnswer> RequestAsync(
        string method,
        string path,
        IReadOnlyList<KeyValuePair<string, string>> query,
        object? body,
        CancellationToken cancellationToken);
}

/// <summary>What the API answered is not what it declares it answers.</summary>
public sealed class DecodeException : Exception
{
    /// <summary>Reports a body this client could not read as what the API declares.</summary>
    /// <param name="detail">What could not be read, and out of what.</param>
    public DecodeException(string detail)
        : base(detail)
    {
    }

    /// <summary>Reports a body this client could not read as what the API declares.</summary>
    /// <param name="detail">What could not be read, and out of what.</param>
    /// <param name="cause">What the reader reported.</param>
    public DecodeException(string detail, Exception cause)
        : base(detail, cause)
    {
    }
}

/// <summary>
/// What the generated half of this package reads and writes values through.
/// </summary>
/// <remarks>
/// Everything here is hand-written and never regenerated. It is the one seam between what the API
/// declares — the records, the problems and the methods the generator writes under
/// <c>Generated</c> — and what it does not: how a JSON document becomes a value, how a value
/// becomes a path segment or a query pair, and what happens to a document that does not say what it
/// was declared to say.
/// </remarks>
public static class Runtime
{
    /// <summary>
    /// Longest fragment of a response body a message carries. Bodies are answered by a server this
    /// package does not control, so they are cut at a fixed budget rather than echoed whole into
    /// whatever the caller logs.
    /// </summary>
    public const int MaxPreviewBytes = 256;

    /// <summary>
    /// Largest JSON document read out of a response body, in bytes. The transport caps what it
    /// reads off a socket; this caps what is handed to the reader whichever way the bytes arrived.
    /// </summary>
    public const int MaxPayloadBytes = 8 * 1024 * 1024;

    /// <summary>
    /// Deepest a JSON document may nest before the reader gives up, which is what keeps a document
    /// that is nothing but brackets from growing the stack.
    /// </summary>
    public const int MaxPayloadNesting = 64;

    /// <summary>Most placeholders one path template is filled from.</summary>
    public const int MaxPathParameters = 32;

    /// <summary>Most pairs one query string carries.</summary>
    public const int MaxQueryParameters = 64;

    /// <summary>The characters a path segment carries as themselves.</summary>
    private const string Unreserved =
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";

    /// <summary>How a moment travels: RFC 3339, to the precision it carries.</summary>
    private const string MomentFormat = "yyyy-MM-dd'T'HH:mm:ss.FFFFFFFK";

    /// <summary>How a day travels.</summary>
    private const string DayFormat = "yyyy-MM-dd";

    private static readonly JsonSerializerOptions Reading = new()
    {
        MaxDepth = MaxPayloadNesting,
        PropertyNameCaseInsensitive = false,
    };

    // Nothing is dropped wholesale on the way out. Which members may be left out is what the
    // document says and what the generated declarations carry, one member at a time: a blanket
    // "omit what is null" would also omit a member the document requires that happens to carry
    // nothing, and the value would not read back as the value that was written.
    private static readonly JsonSerializerOptions Writing = new()
    {
        MaxDepth = MaxPayloadNesting,
    };

    /// <summary>How a value the API declares is read back out of what it answered.</summary>
    public static JsonSerializerOptions ReadingOptions => Reading;

    /// <summary>How a value is written back the way the API reads it.</summary>
    public static JsonSerializerOptions WritingOptions => Writing;

    /// <summary>Reads a value the API declares out of the body it answered.</summary>
    /// <typeparam name="TValue">What the operation declares it answers.</typeparam>
    /// <param name="payload">The body the transport read.</param>
    /// <returns>The value the body carried.</returns>
    /// <exception cref="DecodeException">
    /// When the body is larger than this package reads, is not JSON, or is not what the API
    /// declares it answers.
    /// </exception>
    public static TValue Read<TValue>(byte[] payload)
    {
        ArgumentNullException.ThrowIfNull(payload);
        if (payload.Length > MaxPayloadBytes)
        {
            throw new DecodeException(
                $"the response is {payload.Length} bytes, above the {MaxPayloadBytes} accepted");
        }

        TValue? read;
        try
        {
            read = JsonSerializer.Deserialize<TValue>(payload, Reading);
        }
        catch (JsonException unreadable)
        {
            throw new DecodeException(
                $"the response is not what the API declares: {Preview(payload)}",
                unreadable);
        }

        return read ?? throw new DecodeException(
            $"the response carries no {typeof(TValue).Name}: {Preview(payload)}");
    }

    /// <summary>Reads a value the API declares, or nothing at all when the body is not one.</summary>
    /// <typeparam name="TValue">What the body is being read as.</typeparam>
    /// <param name="payload">The body the transport read.</param>
    /// <returns>The value the body carried, or <c>null</c> when it carried none.</returns>
    public static TValue? ReadOrNothing<TValue>(byte[] payload)
        where TValue : class
    {
        try
        {
            return Read<TValue>(payload);
        }
        catch (DecodeException)
        {
            return null;
        }
    }

    /// <summary>Writes a value back the way the API reads it.</summary>
    /// <param name="body">What to send.</param>
    /// <returns>The bytes of the JSON document.</returns>
    public static byte[] Write(object body) => JsonSerializer.SerializeToUtf8Bytes(body, Writing);

    /// <summary>As much of a response body as a message may carry.</summary>
    /// <param name="payload">The body the transport read.</param>
    /// <returns>The opening of it, as text.</returns>
    public static string Preview(byte[] payload)
    {
        ArgumentNullException.ThrowIfNull(payload);

        int kept = Math.Min(payload.Length, MaxPreviewBytes);
        string rendered = Encoding.UTF8.GetString(payload, 0, kept);
        return payload.Length > MaxPreviewBytes ? rendered + "…" : rendered;
    }

    /// <summary>What to say about an answer the API document does not describe.</summary>
    /// <param name="status">What the API answered under.</param>
    /// <param name="payload">The body it answered.</param>
    /// <returns>What a caller is told.</returns>
    public static string Unreadable(int status, byte[] payload) =>
        $"the API answered {status.ToString(CultureInfo.InvariantCulture)} with a body this client " +
        $"cannot read: {Preview(payload)}";

    /// <summary>What to say about a problem the API reported.</summary>
    /// <param name="status">What the API answered under.</param>
    /// <param name="problem">The problem document it answered.</param>
    /// <returns>What a caller is told.</returns>
    public static string Reported(int status, object problem) =>
        $"the API answered {status.ToString(CultureInfo.InvariantCulture)}: {problem}";

    /// <summary>Where a request lands, with each placeholder of the template filled in.</summary>
    /// <param name="template">The path as the document writes it, placeholders included.</param>
    /// <param name="filled">The value each placeholder carries.</param>
    /// <returns>The path the request is issued against.</returns>
    public static string Path(string template, IReadOnlyList<(string Name, object? Value)> filled)
    {
        ArgumentNullException.ThrowIfNull(template);
        ArgumentNullException.ThrowIfNull(filled);
        if (filled.Count > MaxPathParameters)
        {
            throw new ArgumentOutOfRangeException(
                nameof(filled),
                filled.Count,
                $"a path is filled from at most {MaxPathParameters} placeholders");
        }

        string written = template;
        foreach ((string name, object? value) in filled)
        {
            written = written.Replace($"{{{name}}}", PathSegment(value), StringComparison.Ordinal);
        }

        return written;
    }

    /// <summary>A value as one segment of a path, with nothing left in it that names another one.</summary>
    /// <param name="value">What the caller passed.</param>
    /// <returns>The segment, percent-encoded.</returns>
    public static string PathSegment(object? value)
    {
        byte[] bytes = Encoding.UTF8.GetBytes(Written(value));
        StringBuilder segment = new(bytes.Length);
        foreach (byte written in bytes)
        {
            char character = (char)written;
            if (Unreserved.Contains(character, StringComparison.Ordinal))
            {
                segment.Append(character);
                continue;
            }

            segment.Append(CultureInfo.InvariantCulture, $"%{written:X2}");
        }

        return segment.ToString();
    }

    /// <summary>
    /// What travels in the query string: everything the document requires, and everything it does
    /// not that the caller actually passed.
    /// </summary>
    /// <param name="required">Name and value pairs the operation always sends.</param>
    /// <param name="optional">Name and value pairs it sends only when they carry something.</param>
    /// <returns>The pairs, in the order they are written.</returns>
    public static IReadOnlyList<KeyValuePair<string, string>> Query(
        IReadOnlyList<(string Name, object? Value)> required,
        IReadOnlyList<(string Name, object? Value)> optional)
    {
        ArgumentNullException.ThrowIfNull(required);
        ArgumentNullException.ThrowIfNull(optional);
        if (required.Count + optional.Count > MaxQueryParameters)
        {
            throw new ArgumentOutOfRangeException(
                nameof(optional),
                required.Count + optional.Count,
                $"a query string carries at most {MaxQueryParameters} pairs");
        }

        List<KeyValuePair<string, string>> pairs = new(required.Count + optional.Count);
        foreach ((string name, object? value) in required)
        {
            pairs.Add(new KeyValuePair<string, string>(name, Written(value)));
        }

        foreach ((string name, object? value) in optional)
        {
            if (value is not null)
            {
                pairs.Add(new KeyValuePair<string, string>(name, Written(value)));
            }
        }

        return pairs;
    }

    /// <summary>How a value travels in a request line, which is not always how C# prints it.</summary>
    /// <param name="value">What the caller passed.</param>
    /// <returns>The text that travels.</returns>
    public static string Written(object? value) => value switch
    {
        null => string.Empty,
        string text => text,
        bool flag => flag ? "true" : "false",
        DateTimeOffset moment => moment.ToUniversalTime()
            .ToString(MomentFormat, CultureInfo.InvariantCulture),
        DateTime moment => moment.ToUniversalTime()
            .ToString(MomentFormat, CultureInfo.InvariantCulture),
        DateOnly day => day.ToString(DayFormat, CultureInfo.InvariantCulture),
        Guid identifier => identifier.ToString("D", CultureInfo.InvariantCulture),
        double number => number.ToString("R", CultureInfo.InvariantCulture),
        float number => number.ToString("R", CultureInfo.InvariantCulture),
        IFormattable formattable => formattable.ToString(null, CultureInfo.InvariantCulture),
        _ => value.ToString() ?? string.Empty,
    };
}
