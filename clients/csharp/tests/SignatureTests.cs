// What signature verification does beyond what the shared corpus states.
//
// Every vector the corpus carries is run by `ConformanceTests`, against codes computed outside this
// implementation. What is here is what a corpus of deliveries cannot state: the ceilings a header
// written by whoever reached the endpoint is read under, the refusals that happen while a header is
// being read rather than while a code is being compared, and the fact that the clock window looks
// both ways. The vectors those cases start from are read out of the corpus rather than copied, so a
// secret or a code edited there travels here too.

using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Text.Json.Nodes;
using Xunit;

namespace Hook0.Tests;

/// <summary>What a signature header is read under, and what is refused while reading it.</summary>
public sealed class SignatureTests
{
    private static readonly JsonNode Signatures = Corpus.Contract("signature.json");

    [Fact]
    public void AHeaderAboveWhatIsReadIsRefusedBeforeItIsSplit()
    {
        string oversized = new('t', Signature.MaxSignatureBytes + 1);

        SignatureException refused = Assert.Throws<SignatureException>(() => Signature.Parse(oversized));
        Assert.Equal(SignatureRefusal.Malformed, refused.Refusal);
    }

    [Fact]
    public void AHeaderOfMorePartsThanAreReadIsRefused()
    {
        string many = string.Join(",", Enumerable.Range(0, Signature.MaxSignatureParts + 1).Select(part => $"p{part}=1"));

        SignatureException refused = Assert.Throws<SignatureException>(() => Signature.Parse(many));
        Assert.Equal(SignatureRefusal.Malformed, refused.Refusal);
    }

    [Fact]
    public void ASignatureCoveringMoreHeadersThanAreReadIsRefused()
    {
        string covered = string.Join(
            " ",
            Enumerable.Range(0, Signature.MaxCoveredHeaders + 1).Select(header => $"x-{header}"));

        SignatureException refused = Assert.Throws<SignatureException>(
            () => Signature.Parse($"t=1800000000,h={covered},v1=00"));
        Assert.Equal(SignatureRefusal.Malformed, refused.Refusal);
    }

    [Theory]
    [InlineData("v0=00")]
    [InlineData("t=1800000000")]
    [InlineData("t=1800000000,x=1")]
    [InlineData("t=not-a-moment,v0=00")]
    [InlineData("t=,v0=00")]
    [InlineData("t=1800000000000000,v0=00")]
    [InlineData("nothing at all")]
    [InlineData("v0=00,h=x-event-id")]
    public void AHeaderThatIsNotASignatureIsRefusedWhileItIsRead(string written)
    {
        // The last of these carries a code and covers a header and still says nothing about when it
        // was signed, which is what bounds how long a captured delivery can be replayed. A signature
        // without it is not a weaker signature, it is not one.
        SignatureException refused = Assert.Throws<SignatureException>(() => Signature.Parse(written));
        Assert.Equal(SignatureRefusal.Malformed, refused.Refusal);
    }

    [Fact]
    public void ACodeIsReadWholeOrNotAtAll()
    {
        // The value of a part is what follows its *first* `=`, so a code carrying another one is a
        // code that is not hexadecimal rather than a prefix of one that is.
        Assert.Equal(
            SignatureRefusal.CodeNotHexadecimal,
            Assert.Throws<SignatureException>(() => Signature.Parse("t=1800000000,v0=00=ff")).Refusal);
        Assert.Equal(
            SignatureRefusal.CodeNotHexadecimal,
            Assert.Throws<SignatureException>(() => Signature.Parse("t=1800000000,v0=0")).Refusal);
        Assert.Equal(
            SignatureRefusal.CodeNotHexadecimal,
            Assert.Throws<SignatureException>(() => Signature.Parse("t=1800000000,v0=")).Refusal);
    }

    [Fact]
    public void TheHeadersASignatureCoversAreMatchedWithoutRegardToCase()
    {
        // The signature lowercases what it covers, and a request writes its headers in whichever
        // case it likes; a receiver that compared them as written would refuse a valid delivery.
        JsonNode vector = Accepted("a header-scheme signature verifies");
        List<KeyValuePair<string, string>> shouted =
        [
            .. Delivered(vector).Select(header =>
                new KeyValuePair<string, string>(header.Key.ToUpperInvariant(), header.Value)),
        ];

        Webhooks.VerifyWebhookSignatureWithCurrentTime(
            Text(vector, "signature"),
            Encoding.UTF8.GetBytes(Text(vector, "payload")),
            shouted,
            Text(vector, "secret"),
            Tolerance(vector),
            Moment(vector));
    }

    [Fact]
    public void TheClockWindowLooksBothWays()
    {
        // A delivery the corpus accepts, held against a clock that has fallen behind it by more than
        // the tolerance, is as refused as one held against a clock that has run ahead of it. A window
        // that only looked backwards would be one a sender widens by dating its own delivery ahead.
        JsonNode vector = Accepted("a body-scheme signature verifies");
        TimeSpan tolerance = Tolerance(vector);

        foreach (TimeSpan drift in new[] { tolerance + TimeSpan.FromSeconds(1), -tolerance - TimeSpan.FromSeconds(1) })
        {
            SignatureException refused = Assert.Throws<SignatureException>(
                () => Webhooks.VerifyWebhookSignatureWithCurrentTime(
                    Text(vector, "signature"),
                    Encoding.UTF8.GetBytes(Text(vector, "payload")),
                    Delivered(vector),
                    Text(vector, "secret"),
                    tolerance,
                    Moment(vector) + drift));

            Assert.Equal(SignatureRefusal.OutsideTolerance, refused.Refusal);
        }
    }

    [Fact]
    public void TheEdgeOfTheWindowIsInsideIt()
    {
        JsonNode vector = Accepted("a body-scheme signature verifies");
        TimeSpan tolerance = Tolerance(vector);

        foreach (TimeSpan drift in new[] { tolerance, -tolerance })
        {
            Webhooks.VerifyWebhookSignatureWithCurrentTime(
                Text(vector, "signature"),
                Encoding.UTF8.GetBytes(Text(vector, "payload")),
                Delivered(vector),
                Text(vector, "secret"),
                tolerance,
                Moment(vector) + drift);
        }
    }

    [Fact]
    public void ADeliveryDatedYearsAgoIsRefusedAgainstTheCurrentMoment()
    {
        // The overload that reads the clock itself applies a tolerance of its own, and every vector
        // of the corpus is dated long enough ago for that to settle it.
        JsonNode vector = Accepted("a body-scheme signature verifies");

        SignatureException refused = Assert.Throws<SignatureException>(
            () => Webhooks.VerifyWebhookSignature(
                Text(vector, "signature"),
                Encoding.UTF8.GetBytes(Text(vector, "payload")),
                Delivered(vector),
                Text(vector, "secret")));

        Assert.Equal(SignatureRefusal.OutsideTolerance, refused.Refusal);
    }

    [Fact]
    public void ABodyThatChangedOnTheWayIsRefused()
    {
        JsonNode vector = Accepted("a body-scheme signature verifies");

        SignatureException refused = Assert.Throws<SignatureException>(
            () => Webhooks.VerifyWebhookSignatureWithCurrentTime(
                Text(vector, "signature"),
                Encoding.UTF8.GetBytes(Text(vector, "payload") + " "),
                Delivered(vector),
                Text(vector, "secret"),
                Tolerance(vector),
                Moment(vector)));

        Assert.Equal(SignatureRefusal.CodeMismatch, refused.Refusal);
    }

    [Fact]
    public void AHeaderThatIsNothingAtAllIsRefusedRatherThanRead()
    {
        SignatureException refused = Assert.Throws<SignatureException>(() => Signature.Parse(null!));

        Assert.Equal(SignatureRefusal.Malformed, refused.Refusal);
    }

    [Fact]
    public void ADeliveryCarryingNoHeadersDoesNotCoverTheOnesTheSignatureNames()
    {
        // A caller that passes nothing where the delivery's headers go is refused rather than
        // verified against an empty set: the signature names a header, and none arrived.
        JsonNode vector = Accepted("a header-scheme signature verifies");

        SignatureException refused = Assert.Throws<SignatureException>(
            () => Webhooks.VerifyWebhookSignatureWithCurrentTime(
                Text(vector, "signature"),
                Encoding.UTF8.GetBytes(Text(vector, "payload")),
                null!,
                Text(vector, "secret"),
                Tolerance(vector),
                Moment(vector)));

        Assert.Equal(SignatureRefusal.HeaderNotDelivered, refused.Refusal);
    }

    [Fact]
    public void ADeliveredHeaderThatIsNothingIsRefusedWhileTheDeliveryIsRead()
    {
        JsonNode vector = Accepted("a header-scheme signature verifies");

        SignatureException refused = Assert.Throws<SignatureException>(
            () => Webhooks.VerifyWebhookSignatureWithCurrentTime(
                Text(vector, "signature"),
                Encoding.UTF8.GetBytes(Text(vector, "payload")),
                [new KeyValuePair<string, string>("x-event-id", null!)],
                Text(vector, "secret"),
                Tolerance(vector),
                Moment(vector)));

        Assert.Equal(SignatureRefusal.Malformed, refused.Refusal);
    }

    [Fact]
    public void ASecretThatIsNothingVerifiesNothing()
    {
        // A caller that passes no secret is not one that passes the right one: the code is computed
        // over an empty key and does not match what the sender signed.
        JsonNode vector = Accepted("a body-scheme signature verifies");

        SignatureException refused = Assert.Throws<SignatureException>(
            () => Webhooks.VerifyWebhookSignatureWithCurrentTime(
                Text(vector, "signature"),
                Encoding.UTF8.GetBytes(Text(vector, "payload")),
                Delivered(vector),
                null!,
                Tolerance(vector),
                Moment(vector)));

        Assert.Equal(SignatureRefusal.CodeMismatch, refused.Refusal);
    }

    [Theory]
    [InlineData("x-one  x-two")]
    [InlineData("x-a/b")]
    [InlineData("x-a=b")]
    [InlineData("x-\u00e9vent")]
    public void ASignatureCoveringSomethingThatIsNotAHeaderNameIsRefusedWhileItIsRead(string covered)
    {
        // What a header name may be spelled with is RFC 9110's token, and nothing else. A sender
        // that names something outside it is refused while the signature is read rather than at the
        // comparison, because whatever value a name like that would look up is not a header's.
        SignatureException refused = Assert.Throws<SignatureException>(
            () => Signature.Parse($"t=1800000000,h={covered},v1=00"));

        Assert.Equal(SignatureRefusal.Malformed, refused.Refusal);
    }

    [Fact]
    public void AHeaderWrittenWithSpaceAroundItsPartsIsRead()
    {
        // Each part is trimmed on both sides of its `=`, so a sender that writes the header the way
        // a person would — a space after each comma — is read rather than refused. That is why the
        // case above needs an interior double space to produce a name that is nothing: space around
        // the list never survives to be one.
        Signature parsed = Signature.Parse("t = 1800000000 , h = x-event-id , v1 = 00");

        Assert.Equal(1800000000L, parsed.Timestamp);
        Assert.Equal(["x-event-id"], parsed.CoveredHeaders);
    }

    [Fact]
    public void APartTheReaderDoesNotKnowIsSkippedRatherThanRefusingTheWholeHeader()
    {
        // A part carrying no `=` is not a part, and the header is still read for the parts that are.
        // That is what lets Hook0 add one without every deployed verifier refusing the delivery it
        // arrives on; a reader that insisted on understanding every part would make the header
        // unable to grow.
        Signature parsed = Signature.Parse("t=1800000000,something-a-later-version-adds,v0=00");

        Assert.Equal(1800000000L, parsed.Timestamp);
        Assert.Empty(parsed.CoveredHeaders);
    }

    [Fact]
    public void ACallerThatNamesItsOwnWindowIsHeldToThatOneRatherThanTheDefault()
    {
        // The overload that reads the clock itself takes a tolerance, and every vector of the corpus
        // sits far enough from now that the default refuses it. A window wide enough to reach the
        // moment the signature names accepts it, which is the only way to tell the argument is
        // honoured rather than shadowed by the default.
        JsonNode vector = Accepted("a body-scheme signature verifies");
        string written = Text(vector, "signature");
        DateTimeOffset signed = DateTimeOffset.FromUnixTimeSeconds(Signature.Parse(written).Timestamp);
        TimeSpan apart = (DateTimeOffset.UtcNow - signed).Duration();

        Assert.Throws<SignatureException>(
            () => Webhooks.VerifyWebhookSignature(
                written,
                Encoding.UTF8.GetBytes(Text(vector, "payload")),
                Delivered(vector),
                Text(vector, "secret")));

        Webhooks.VerifyWebhookSignature(
            written,
            Encoding.UTF8.GetBytes(Text(vector, "payload")),
            Delivered(vector),
            Text(vector, "secret"),
            apart + TimeSpan.FromDays(1));
    }

    [Fact]
    public void ADeliveryCarryingNoBodyIsReadAsOneCarryingAnEmptyBody()
    {
        // Some frameworks hand a handler nothing rather than an empty array when a request arrived
        // without a body. What is promised is that the two are the same delivery: the verdict, and
        // the words it is refused in, are identical rather than one of them being a null reference.
        JsonNode vector = Accepted("a body-scheme signature verifies");

        SignatureException fromNothing = Assert.Throws<SignatureException>(
            () => Webhooks.VerifyWebhookSignatureWithCurrentTime(
                Text(vector, "signature"),
                null!,
                Delivered(vector),
                Text(vector, "secret"),
                Tolerance(vector),
                Moment(vector)));
        SignatureException fromEmpty = Assert.Throws<SignatureException>(
            () => Webhooks.VerifyWebhookSignatureWithCurrentTime(
                Text(vector, "signature"),
                Array.Empty<byte>(),
                Delivered(vector),
                Text(vector, "secret"),
                Tolerance(vector),
                Moment(vector)));

        Assert.Equal(SignatureRefusal.CodeMismatch, fromNothing.Refusal);
        Assert.Equal(fromEmpty.Refusal, fromNothing.Refusal);
        Assert.Equal(fromEmpty.Message, fromNothing.Message);
    }

    /// <summary>One vector the corpus accepts, by the name it gives it.</summary>
    private static JsonNode Accepted(string name)
    {
        foreach (JsonNode? vector in Signatures["vectors"]!.AsArray())
        {
            if (Text(vector!, "name") == name)
            {
                return vector!;
            }
        }

        throw new InvalidOperationException($"the corpus carries no vector called `{name}`");
    }

    private static List<KeyValuePair<string, string>> Delivered(JsonNode vector)
    {
        List<KeyValuePair<string, string>> headers = [];
        foreach (JsonNode? header in vector["headers"]!.AsArray())
        {
            JsonArray pair = header!.AsArray();
            headers.Add(new KeyValuePair<string, string>(
                pair[0]!.GetValue<string>(),
                pair[1]!.GetValue<string>()));
        }

        return headers;
    }

    private static TimeSpan Tolerance(JsonNode vector) =>
        TimeSpan.FromSeconds(vector["tolerance_seconds"]!.GetValue<double>());

    private static DateTimeOffset Moment(JsonNode vector) =>
        DateTimeOffset.FromUnixTimeSeconds(vector["current_time"]!.GetValue<long>());

    private static string Text(JsonNode document, string name) =>
        document[name]?.GetValue<string>() ?? string.Empty;
}
