// What reaching the API costs a caller, on the transport's own surface rather than through a send.
//
// `ClientTests` drives this class through `Hook0Client` and `GeneratedTests` through the generated
// methods, and between them they cover the exchange that works. What is here is what neither of
// them can reach from where it stands: the base URLs and paths no request is ever issued against,
// the failures that are the caller's own decision rather than the network's, and the ceilings the
// head of an answer is read under. Each case names the verdict the shared conformance corpus gives
// the failure, because that name is what a caller branches on.

using System;
using System.Collections.Generic;
using System.Text.Json.Nodes;
using System.Threading;
using System.Threading.Tasks;
using Xunit;

namespace Hook0.Tests;

/// <summary>What the transport does with a request it cannot issue, and an answer it cannot read.</summary>
public sealed class TransportTests : ApiCase
{
    /// <summary>The verdicts the corpus gives the three ways reaching the API fails.</summary>
    private const string UnusableApiUrl = "unusable_api_url";
    private const string NoAnswer = "no_answer";
    private const string AnswerAboveABound = "answer_above_a_bound";

    /// <summary>Long enough for a loopback exchange, short enough that a stall is not a wait.</summary>
    private static readonly TimeSpan Patience = TimeSpan.FromSeconds(5);

    [Theory]
    [InlineData("not a url at all")]
    [InlineData("ftp://example.test/api/v1")]
    [InlineData("http:///api/v1")]
    [InlineData("")]
    public void AnApiUrlThatIsNotSomewhereARequestCanGoIsRefusedRatherThanReachedFor(string apiUrl)
    {
        // Each way a base URL fails to be one a request can be sent to: not a URL, a scheme nothing
        // is sent over, and no host to send it to. All of them are refused when the request is built
        // rather than when a connection is attempted, and refused as a misconfiguration — repeating
        // it builds the same unusable request, so a caller told to retry would only wait to fail.
        using HttpTransport transport = new(apiUrl, "token-xyz");

        TransportException refused = Assert.Throws<TransportException>(
            () => transport.Deliver("GET", "somewhere", [], null));

        Assert.Equal(UnusableApiUrl, refused.CauseName);
        Assert.False(refused.Retryable);
        Assert.Empty(Api.Received);
    }

    [Theory]
    [InlineData("///")]
    [InlineData("http://")]
    [InlineData("http://[not-an-authority")]
    [InlineData(null)]
    public void APathThatIsNotSomewhereARequestCanGoIsRefusedRatherThanSent(string? path)
    {
        // The base URL is reachable and the path is not, which the caller hears as the same kind of
        // failure and about the path rather than about the network. A generated method cannot land
        // here — its paths come out of the API description — but the transport is published, and a
        // caller assembling a path of its own can.
        using HttpTransport transport = Reaching();

        TransportException refused = Assert.Throws<TransportException>(
            () => transport.Deliver("GET", path!, [], null));

        Assert.Equal(UnusableApiUrl, refused.CauseName);
        Assert.False(refused.Retryable);
        Assert.Empty(Api.Received);
    }

    [Theory]
    [InlineData("/api/v1")]
    [InlineData("/api/v1/")]
    public void ARequestLandsUnderTheBaseUrlWhetherOrNotItWasWrittenWithATrailingSlash(string under)
    {
        // Whether the API URL a caller configured ends in a slash is not something they should have
        // to think about, and it is the difference between landing under the base and landing at the
        // root of the host — which is a different API.
        Api.WillAnswer(Answering());
        using HttpTransport transport = Reaching(Api.BaseUrl + under);

        transport.Deliver("GET", "somewhere", [], null);

        Assert.Equal("/api/v1/somewhere", Assert.Single(Api.Received).Target);
    }

    [Fact]
    public void AQueryIsAddedToWhateverThePathAlreadyCarried()
    {
        // A path that already names something is extended rather than given a second `?`, so the
        // API reads one query string rather than a request line it has to guess at.
        Api.WillAnswer(Answering(), Answering());
        using HttpTransport transport = Reaching();

        transport.Deliver("GET", "somewhere", [new KeyValuePair<string, string>("first", "one")], null);
        transport.Deliver(
            "GET",
            "somewhere?tenant=acme",
            [new KeyValuePair<string, string>("first", "one")],
            null);

        Assert.Equal("/api/v1/somewhere?first=one", Api.Received[0].Target);
        Assert.Equal("/api/v1/somewhere?tenant=acme&first=one", Api.Received[1].Target);
    }

    [Fact]
    public void AnAnswerCarryingAHeaderLineAboveWhatIsReadIsRefusedAndSaysWhichOne()
    {
        // The head is read under a ceiling per line as well as one for the whole of it, and the per
        // line one earns its place by refusing on the line that is too long rather than once every
        // line has been read. What it says names the header, so the operator of a proxy that added
        // it can find it.
        Api.WillAnswer(Answering() with
        {
            Headers = [new KeyValuePair<string, string>("X-Padding", new string('y', 100))],
        });
        using HttpTransport transport = Reaching(maxHeaderBytes: 64);

        TransportException refused = Assert.Throws<TransportException>(
            () => transport.Deliver("GET", "somewhere", [], null));

        Assert.Equal(AnswerAboveABound, refused.CauseName);
        Assert.False(refused.Retryable);
        Assert.Contains("x-padding", refused.Message, StringComparison.Ordinal);
    }

    [Fact]
    public void AnApiNothingIsListeningOnIsAFailureARepetitionCouldClear()
    {
        // A refused connection says nothing about whether the API acted on the request, which is
        // exactly the case a repetition is for — and the identifier a send chooses itself is what
        // makes repeating it safe. It is also the one transport failure that is not a clock running
        // out, so it is the one that proves the classification is not reading every failure as one.
        string closed = Closed();
        using HttpTransport transport = Reaching(closed);

        TransportException refused = Assert.Throws<TransportException>(
            () => transport.Deliver("GET", "somewhere", [], null));

        Assert.Equal(NoAnswer, refused.CauseName);
        Assert.True(refused.Retryable);
        Assert.NotNull(refused.InnerException);
    }

    [Fact]
    public async Task ACallerThatAbandonsItsOwnRequestIsNotToldTheNetworkFailed()
    {
        // A cancellation the caller asked for travels back as the cancellation it is. Reporting it
        // as a transport failure would have the client retry a request its caller just gave up on,
        // and would put the caller's own decision in a message that accuses the API.
        using HttpTransport transport = Reaching();
        using CancellationTokenSource abandoned = new();
        await abandoned.CancelAsync();

        await Assert.ThrowsAnyAsync<OperationCanceledException>(
            () => transport.DeliverAsync("GET", "somewhere", [], null, abandoned.Token));

        Assert.Empty(Api.Received);
    }

    [Fact]
    public void ATransportBuiltWithoutAScheduleStatesTheDefaultOne()
    {
        // The constructor that takes no policy is the one a caller who is not retrying reaches for,
        // and what it states on the wire is what a caller who spelled the default out would state.
        // An API reading the header cannot tell the two apart, which is the promise.
        Api.WillAnswer(Answering(), Answering());
        using HttpTransport assumed = new(Api.BaseUrl + "/api/v1", "token-xyz");
        using HttpTransport spelled = new(
            Api.BaseUrl + "/api/v1",
            "token-xyz",
            null,
            HttpTransport.DefaultMaxResponseBytes,
            HttpTransport.DefaultMaxResponseHeaders,
            HttpTransport.DefaultMaxHeadBytes,
            HttpTransport.DefaultMaxHeaderBytes,
            new RetryPolicy());

        assumed.Deliver("GET", "somewhere", [], null);
        spelled.Deliver("GET", "somewhere", [], null);

        Assert.Equal(Stated(Api.Received[1]), Stated(Api.Received[0]));
        Assert.Contains("attempts=", Stated(Api.Received[0]), StringComparison.Ordinal);
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task AnAnswerFarBelowTheCeilingIsReadWholeHoweverManyPiecesItArrivedIn(Surface surface)
    {
        // What bounds a body is how many bytes it is, and nothing else. How a server chose to flush
        // it, and how the network broke it up on the way, is not something the caller decided and not
        // something they can be refused over: the same document arrives in one read on loopback and
        // in dozens across a link with a smaller segment size, and both are the same answer.
        //
        // The ceiling is set low here only so the case does not have to write a hundred pieces to
        // make the point. The body is a small fraction of it either way.
        string document = new('y', 800);
        Api.WillAnswer(new ScriptedResponse(200, null, Written: $"\"{document}\"", Pieces: 8));
        using HttpTransport transport = Reaching(maxResponseBytes: 64 * 1024);

        TransportDelivery delivered = await Delivered(transport, surface);

        Assert.Equal(200, delivered.Status);
        Assert.Equal(document.Length + 2, delivered.Payload.Length);
    }

    [Theory]
    [InlineData(Surface.Blocking)]
    [InlineData(Surface.Awaiting)]
    public async Task AnAnswerThatKeepsArrivingIsStoppedByTheCeilingItCrossed(Surface surface)
    {
        // The other half of that, and the reason there is a bound here at all: a server that writes
        // and writes is cut off on the piece that crosses the ceiling, however many pieces it took to
        // get there. Refused rather than repeated — the same request draws the same oversized answer.
        Api.WillAnswer(new ScriptedResponse(200, null, Written: new string('y', 2048), Pieces: 16));
        using HttpTransport transport = Reaching(maxResponseBytes: 512);

        TransportException refused = surface == Surface.Blocking
            ? Assert.Throws<TransportException>(() => transport.Deliver("GET", "somewhere", [], null))
            : await Assert.ThrowsAsync<TransportException>(
                () => transport.DeliverAsync("GET", "somewhere", [], null, CancellationToken.None));

        Assert.Equal(AnswerAboveABound, refused.CauseName);
        Assert.False(refused.Retryable);
    }

    [Fact]
    public async Task AnAnswerThatStopsMidWayAndNeverEndsIsGivenUpOnRatherThanWaitedOutForever()
    {
        // A server that states a length, writes part of it and then holds the connection open is the
        // case where nothing about the answer itself will ever end the read. What ends it is this
        // transport's own deadline, and what the caller hears is that the API did not answer — a
        // failure repeating the request could clear, rather than one about the answer's size.
        //
        // Awaited only. The blocking surface does not hold to the deadline here: `Stream.Read` on the
        // response body does not observe the token the request was given, and the deadline is only
        // looked at between reads, so a read that never returns is bounded by whatever the server
        // eventually does. Asserting that from a case would mean waiting for the server rather than
        // for the transport, which is not what this is about.
        Api.WillAnswer(new ScriptedResponse(200, null, Written: new string('y', 500), Promising: 2000));
        using HttpTransport transport = Reaching(timeout: TimeSpan.FromMilliseconds(500));

        TransportException refused = await Assert.ThrowsAsync<TransportException>(
            () => transport.DeliverAsync("GET", "somewhere", [], null, CancellationToken.None));

        Assert.Equal(NoAnswer, refused.CauseName);
        Assert.True(refused.Retryable);
    }

    /// <summary>A transport reaching the API of this case, under the bounds a case names.</summary>
    private HttpTransport Reaching(
        string? apiUrl = null,
        int maxHeaderBytes = HttpTransport.DefaultMaxHeaderBytes,
        int maxResponseBytes = HttpTransport.DefaultMaxResponseBytes,
        TimeSpan? timeout = null) =>
        new(
            apiUrl ?? Api.BaseUrl + "/api/v1",
            "token-xyz",
            timeout ?? Patience,
            maxResponseBytes,
            HttpTransport.DefaultMaxResponseHeaders,
            HttpTransport.DefaultMaxHeadBytes,
            maxHeaderBytes);

    /// <summary>One request, on whichever of the two surfaces the case is written for.</summary>
    private static async Task<TransportDelivery> Delivered(HttpTransport transport, Surface surface) =>
        surface == Surface.Blocking
            ? transport.Deliver("GET", "somewhere", [], null)
            : await transport.DeliverAsync("GET", "somewhere", [], null, CancellationToken.None);

    /// <summary>A URL that was somewhere a request could go until the API behind it stopped.</summary>
    private static string Closed()
    {
        using FakeApi stopped = new();
        return stopped.BaseUrl + "/api/v1";
    }

    /// <summary>An answer with nothing in it worth reading, for a case that is about the request.</summary>
    private static ScriptedResponse Answering() => new(200, new JsonObject());

    /// <summary>What one request said about the schedule its transport holds.</summary>
    private static string Stated(ReceivedRequest request)
    {
        foreach (KeyValuePair<string, string> header in request.Headers)
        {
            if (string.Equals(header.Key, "Hook0-Client-Options", StringComparison.OrdinalIgnoreCase))
            {
                return header.Value;
            }
        }

        return string.Empty;
    }
}
