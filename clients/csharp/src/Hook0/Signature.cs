using System;
using System.Collections.Generic;
using System.Globalization;
using System.Security.Cryptography;
using System.Text;

namespace Hook0;

/// <summary>Why a delivery was refused.</summary>
/// <remarks>
/// The four that the shared conformance corpus names are here under the names it gives them, so a
/// client that refuses the right delivery for the wrong reason — computing a code over a header that
/// never arrived, and reporting a mismatch — is caught rather than credited.
/// </remarks>
public enum SignatureRefusal
{
    /// <summary>The header is not a signature at all, in a way the corpus does not name.</summary>
    Malformed,

    /// <summary>A code the signature carries is not whole hexadecimal.</summary>
    CodeNotHexadecimal,

    /// <summary>A header the signature covers was not part of the request.</summary>
    HeaderNotDelivered,

    /// <summary>The code is not the one the subscription secret produces.</summary>
    CodeMismatch,

    /// <summary>The moment the signature names sits outside the window it is accepted within.</summary>
    OutsideTolerance,
}

/// <summary>A delivery this client refuses, and why.</summary>
public sealed class SignatureException : Hook0Exception
{
    /// <summary>Reports a delivery that was refused.</summary>
    /// <param name="refusal">Which refusal this is.</param>
    /// <param name="detail">What to say about it.</param>
    public SignatureException(SignatureRefusal refusal, string detail)
        : base(detail) => Refusal = refusal;

    /// <summary>Which refusal this is.</summary>
    public SignatureRefusal Refusal { get; }
}

/// <summary>A signature header, read into the pieces a verification needs.</summary>
/// <remarks>
/// A signature names the moment it was signed and one or two message authentication codes over the
/// body. The <c>v1</c> scheme also covers a list of request headers, so a receiver can tell apart
/// two deliveries that carry the same body but not the same context; <c>v0</c> covers the body alone
/// and is what an older sender still produces. When both are offered, <c>v1</c> is the one verified:
/// accepting the weaker of two schemes on the strength of the sender offering it is how a downgrade
/// works.
/// <para>
/// Two things are refused before any code is computed. A header the signature says it covers but the
/// request did not carry is refused outright, because signing over an absent value would let a
/// sender drop a header and keep the signature valid. And a signature whose codes are not whole
/// hexadecimal is refused rather than decoded as far as it goes: a decoder that stops at the first
/// bad character compares a prefix, and a prefix of the right code is not the right code.
/// </para>
/// </remarks>
public sealed class Signature
{
    /// <summary>
    /// Longest signature header read. The header is written by whoever reached the endpoint, so its
    /// size is bounded before any of it is split, decoded or compared.
    /// </summary>
    public const int MaxSignatureBytes = 8 * 1024;

    /// <summary>Most <c>key=value</c> parts one signature header is split into.</summary>
    public const int MaxSignatureParts = 32;

    /// <summary>Most header names one signature covers.</summary>
    public const int MaxCoveredHeaders = 64;

    /// <summary>
    /// Furthest from the epoch, in either direction, a signature's moment may sit. A header carrying
    /// hundreds of digits would otherwise reach the arithmetic that holds it against the current
    /// time and cost more than reading it did.
    /// </summary>
    public const long MaxTimestamp = 1_000_000_000_000L;

    /// <summary>What separates one part of the signature header from the next.</summary>
    private const char PartSeparator = ',';

    /// <summary>
    /// What separates the name of a part from its value. Only the first one counts: a value may hold
    /// further ones, and splitting on all of them would silently drop everything past the second.
    /// </summary>
    private const char PartAssignator = '=';

    /// <summary>What separates two header names inside the <c>h</c> part, and what they are joined back with.</summary>
    private const string HeaderNameSeparator = " ";

    /// <summary>What separates the pieces of the message a code is computed over.</summary>
    private const string MessageSeparator = ".";

    /// <summary>Part naming the moment the delivery was signed, in whole seconds since the epoch.</summary>
    private const string TimestampPart = "t";

    /// <summary>Part carrying the code covering the body alone.</summary>
    private const string BodySchemePart = "v0";

    /// <summary>Part carrying the code covering the covered headers and the body.</summary>
    private const string HeadersSchemePart = "v1";

    /// <summary>Part listing the headers the <c>v1</c> code covers, in the order it covers them.</summary>
    private const string CoveredHeadersPart = "h";

    /// <summary>What a header name is written with, as RFC 9110 spells a token.</summary>
    private const string HeaderNameCharacters =
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!#$%&'*+-.^_`|~";

    private Signature(long timestamp, IReadOnlyList<string> coveredHeaders, byte[]? bodyCode, byte[]? headersCode)
    {
        Timestamp = timestamp;
        CoveredHeaders = coveredHeaders;
        BodyCode = bodyCode;
        HeadersCode = headersCode;
    }

    /// <summary>The moment the delivery was signed, in whole seconds since the epoch.</summary>
    public long Timestamp { get; }

    /// <summary>The headers the stronger scheme covers, lowercased and in order.</summary>
    public IReadOnlyList<string> CoveredHeaders { get; }

    /// <summary>The <c>v0</c> code, decoded, when the signature carries one.</summary>
    public byte[]? BodyCode { get; }

    /// <summary>The <c>v1</c> code, decoded, when the signature carries one.</summary>
    public byte[]? HeadersCode { get; }

    /// <summary>Reads a signature header, refusing anything it cannot read whole.</summary>
    /// <param name="signature">The value of the <c>X-Hook0-Signature</c> header.</param>
    /// <returns>The pieces a verification needs.</returns>
    /// <exception cref="SignatureException">For every way a header can fail to be one.</exception>
    public static Signature Parse(string signature)
    {
        if (signature is null)
        {
            throw Refused(SignatureRefusal.Malformed, "the signature is nothing, not a header value");
        }

        if (signature.Length > MaxSignatureBytes)
        {
            throw Refused(
                SignatureRefusal.Malformed,
                $"the signature is {signature.Length} characters long, above the {MaxSignatureBytes} accepted");
        }

        Dictionary<string, string> read = PartsOf(signature);
        if (read.Count < 2)
        {
            throw Refused(SignatureRefusal.Malformed, "the signature carries neither a timestamp nor a code");
        }

        byte[]? bodyCode = CodeOf(read, BodySchemePart);
        byte[]? headersCode = CodeOf(read, HeadersSchemePart);
        if (bodyCode is null && headersCode is null)
        {
            throw Refused(
                SignatureRefusal.Malformed,
                $"the signature carries neither a `{BodySchemePart}` nor a `{HeadersSchemePart}` code");
        }

        return new Signature(TimestampOf(read), CoveredHeadersOf(read), bodyCode, headersCode);
    }

    /// <summary>Whether the code this signature carries is the one the secret produces.</summary>
    /// <remarks>
    /// The stronger scheme wins when both are offered, and the comparison is made in constant time:
    /// one that gave up at the first differing byte would say, by how long it took, how much of a
    /// guess was right.
    /// </remarks>
    /// <param name="payload">The raw body of the webhook request.</param>
    /// <param name="coveredValues">The values of the covered headers, in order.</param>
    /// <param name="subscriptionSecret">The signing secret of the subscription.</param>
    /// <returns>Whether the delivery is the one that was signed.</returns>
    public bool Matches(byte[] payload, IReadOnlyList<string> coveredValues, string subscriptionSecret)
    {
        ArgumentNullException.ThrowIfNull(payload);
        ArgumentNullException.ThrowIfNull(coveredValues);

        using IncrementalHash code = IncrementalHash.CreateHMAC(
            HashAlgorithmName.SHA256,
            Encoding.UTF8.GetBytes(subscriptionSecret ?? string.Empty));

        Fed(code, Timestamp.ToString(CultureInfo.InvariantCulture));
        Fed(code, MessageSeparator);

        if (HeadersCode is not null)
        {
            Fed(code, string.Join(HeaderNameSeparator, CoveredHeaders));
            Fed(code, MessageSeparator);
            Fed(code, string.Join(MessageSeparator, coveredValues));
            Fed(code, MessageSeparator);
            code.AppendData(payload);
            return CryptographicOperations.FixedTimeEquals(code.GetHashAndReset(), HeadersCode);
        }

        // A signature carrying neither code is refused while it is being read, so what is left here
        // is the body-only scheme.
        code.AppendData(payload);
        return BodyCode is not null
            && CryptographicOperations.FixedTimeEquals(code.GetHashAndReset(), BodyCode);
    }

    private static void Fed(IncrementalHash code, string text) =>
        code.AppendData(Encoding.UTF8.GetBytes(text));

    /// <summary>The <c>key=value</c> parts of a header, split on the first assignator of each and trimmed.</summary>
    private static Dictionary<string, string> PartsOf(string signature)
    {
        string[] parts = signature.Split(PartSeparator);
        if (parts.Length > MaxSignatureParts)
        {
            throw Refused(
                SignatureRefusal.Malformed,
                $"the signature carries more than the {MaxSignatureParts} parts accepted");
        }

        Dictionary<string, string> read = new(parts.Length, StringComparer.Ordinal);
        foreach (string part in parts)
        {
            int assigned = part.IndexOf(PartAssignator, StringComparison.Ordinal);
            if (assigned < 0)
            {
                continue;
            }

            read[part[..assigned].Trim()] = part[(assigned + 1)..].Trim();
        }

        return read;
    }

    /// <summary>The moment the signature names, which it is not a signature without.</summary>
    private static long TimestampOf(IReadOnlyDictionary<string, string> read)
    {
        if (!read.TryGetValue(TimestampPart, out string? written))
        {
            throw Refused(SignatureRefusal.Malformed, $"the signature carries no `{TimestampPart}` part");
        }

        if (!WholeSeconds(written))
        {
            throw Refused(SignatureRefusal.Malformed, $"`{written}` is not a number of seconds");
        }

        if (!long.TryParse(written, NumberStyles.AllowLeadingSign, CultureInfo.InvariantCulture, out long seconds)
            || Math.Abs(seconds) > MaxTimestamp)
        {
            throw Refused(
                SignatureRefusal.Malformed,
                $"the signature's moment is further than {MaxTimestamp} seconds from the epoch");
        }

        return seconds;
    }

    /// <summary>
    /// What a whole number of seconds reads as. Nothing else is accepted: a spelling no sender
    /// produces is not one a receiver should invent a meaning for.
    /// </summary>
    private static bool WholeSeconds(string written)
    {
        int digits = written.StartsWith('-') ? 1 : 0;
        if (written.Length == digits)
        {
            return false;
        }

        for (int index = digits; index < written.Length; index++)
        {
            if (!char.IsAsciiDigit(written[index]))
            {
                return false;
            }
        }

        return true;
    }

    /// <summary>One of the codes a signature offers, decoded whole or not at all.</summary>
    private static byte[]? CodeOf(IReadOnlyDictionary<string, string> read, string part)
    {
        if (!read.TryGetValue(part, out string? written))
        {
            return null;
        }

        if (written.Length == 0 || written.Length % 2 != 0 || !Hexadecimal(written))
        {
            throw Refused(SignatureRefusal.CodeNotHexadecimal, $"the `{part}` code is not hexadecimal");
        }

        return Convert.FromHexString(written);
    }

    private static bool Hexadecimal(string written)
    {
        foreach (char character in written)
        {
            if (!char.IsAsciiHexDigit(character))
            {
                return false;
            }
        }

        return true;
    }

    /// <summary>The headers the stronger scheme covers, in the order it covers them.</summary>
    private static IReadOnlyList<string> CoveredHeadersOf(IReadOnlyDictionary<string, string> read)
    {
        if (!read.TryGetValue(CoveredHeadersPart, out string? written) || written.Length == 0)
        {
            return Array.Empty<string>();
        }

        string[] names = written.Split(HeaderNameSeparator);
        if (names.Length > MaxCoveredHeaders)
        {
            throw Refused(
                SignatureRefusal.Malformed,
                $"the signature covers more than the {MaxCoveredHeaders} headers accepted");
        }

        List<string> covered = new(names.Length);
        foreach (string name in names)
        {
            if (name.Length == 0 || !HeaderName(name))
            {
                throw Refused(SignatureRefusal.Malformed, $"`{name}` is not a header name");
            }

            covered.Add(name.ToLowerInvariant());
        }

        return covered;
    }

    private static bool HeaderName(string name)
    {
        foreach (char character in name)
        {
            if (!HeaderNameCharacters.Contains(character, StringComparison.Ordinal))
            {
                return false;
            }
        }

        return true;
    }

    private static SignatureException Refused(SignatureRefusal refusal, string detail) =>
        new(refusal, detail);
}

/// <summary>Verifying that a webhook came from Hook0, and that nothing in it changed on the way.</summary>
public static class Webhooks
{
    /// <summary>How far a signature's moment may sit from now unless the caller says otherwise.</summary>
    /// <remarks>
    /// Five minutes is a reasonable trade-off between tolerating clock drift and bounding how long a
    /// captured delivery can be replayed.
    /// </remarks>
    public static readonly TimeSpan DefaultTolerance = TimeSpan.FromMinutes(5);

    /// <summary>Verifies a webhook against the current moment.</summary>
    /// <param name="signature">The value of the <c>X-Hook0-Signature</c> header.</param>
    /// <param name="payload">The raw body of the webhook request.</param>
    /// <param name="headers">The headers of the webhook request.</param>
    /// <param name="subscriptionSecret">The signing secret of the subscription it was delivered for.</param>
    /// <param name="tolerance">
    /// How far, in either direction, the moment the signature names may sit from now.
    /// </param>
    /// <exception cref="SignatureException">For every reason a webhook may be refused.</exception>
    public static void VerifyWebhookSignature(
        string signature,
        byte[] payload,
        IEnumerable<KeyValuePair<string, string>> headers,
        string subscriptionSecret,
        TimeSpan? tolerance = null)
    {
        VerifyWebhookSignatureWithCurrentTime(
            signature,
            payload,
            headers,
            subscriptionSecret,
            tolerance ?? DefaultTolerance,
            DateTimeOffset.UtcNow);
    }

    /// <summary>Verifies a webhook against a moment the caller names.</summary>
    /// <remarks>
    /// The clock window is bilateral. A moment too far in the future is refused exactly like one too
    /// far in the past, so the window a given delivery is accepted in stays the width the caller
    /// asked for, whichever way a clock drifted.
    /// </remarks>
    /// <param name="signature">The value of the <c>X-Hook0-Signature</c> header.</param>
    /// <param name="payload">The raw body of the webhook request.</param>
    /// <param name="headers">The headers of the webhook request.</param>
    /// <param name="subscriptionSecret">The signing secret of the subscription it was delivered for.</param>
    /// <param name="tolerance">
    /// How far, in either direction, the moment the signature names may sit from
    /// <paramref name="currentTime"/>.
    /// </param>
    /// <param name="currentTime">What to hold the signature's moment against.</param>
    /// <exception cref="SignatureException">For every reason a webhook may be refused.</exception>
    public static void VerifyWebhookSignatureWithCurrentTime(
        string signature,
        byte[] payload,
        IEnumerable<KeyValuePair<string, string>> headers,
        string subscriptionSecret,
        TimeSpan tolerance,
        DateTimeOffset currentTime)
    {
        Signature parsed = Signature.Parse(signature);

        IReadOnlyDictionary<string, string> delivered = Delivered(headers);
        List<string> coveredValues = new(parsed.CoveredHeaders.Count);
        foreach (string name in parsed.CoveredHeaders)
        {
            if (!delivered.TryGetValue(name, out string? value))
            {
                throw new SignatureException(
                    SignatureRefusal.HeaderNotDelivered,
                    $"the `{name}` header the signature covers was not delivered");
            }

            coveredValues.Add(value);
        }

        if (!parsed.Matches(payload ?? Array.Empty<byte>(), coveredValues, subscriptionSecret))
        {
            throw new SignatureException(
                SignatureRefusal.CodeMismatch,
                "the signature does not match what the subscription secret produces");
        }

        double drift = currentTime.ToUnixTimeMilliseconds() / 1000d - parsed.Timestamp;
        if (Math.Abs(drift) > tolerance.TotalSeconds)
        {
            throw new SignatureException(
                SignatureRefusal.OutsideTolerance,
                string.Create(
                    CultureInfo.InvariantCulture,
                    $"the signature was made {drift:F0} seconds from now, outside the {tolerance.TotalSeconds} accepted"));
        }
    }

    /// <summary>The headers of the request, under the names a signature refers to them by.</summary>
    /// <remarks>A later value wins over an earlier one under the same name.</remarks>
    private static IReadOnlyDictionary<string, string> Delivered(
        IEnumerable<KeyValuePair<string, string>> headers)
    {
        Dictionary<string, string> delivered = new(StringComparer.Ordinal);
        if (headers is null)
        {
            return delivered;
        }

        foreach (KeyValuePair<string, string> header in headers)
        {
            if (header.Key is null || header.Value is null)
            {
                throw new SignatureException(
                    SignatureRefusal.Malformed,
                    "a header is nothing, not a header value");
            }

            delivered[header.Key.ToLowerInvariant()] = header.Value;
        }

        return delivered;
    }
}
