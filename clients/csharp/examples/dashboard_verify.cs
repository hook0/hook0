// What the dashboard shows under "Verify a webhook", for C#.
//
// Sending is only half of what a reader has come to do, and it is the easier half. This is the one
// the SDK reference calls the half most often got wrong by hand, so the dashboard shows it beside
// the send rather than leaving it to be found later.
//
// The secret is read from the environment on purpose. The dashboard cannot know which subscription
// a reader means — outside the onboarding it loads none, and an application may have several — so
// it points at the subscription instead of guessing one, and no second secret is put on screen.
//
// Read the markers as in `dashboard_send.cs`: `hook0:snippet` is what is displayed, everything
// outside it is what makes the file compile.

// hook0:snippet:begin
using System;
using System.Collections.Generic;
using Hook0;

public static class WebhookEndpoint
{
    // Verify against the *raw* bytes: a body that has been parsed and serialised again no longer
    // hashes to what was signed, which is how a valid delivery comes to be refused. The tolerance
    // is bilateral, so a delivery dated too far ahead is refused exactly like one dated too far
    // behind.
    public static bool Accept(
        string signature,
        byte[] payload,
        IEnumerable<KeyValuePair<string, string>> headers)
    {
        // The secret of the subscription being verified, which the dashboard links to rather than
        // prints: it cannot know which subscription a reader means, and an application may have
        // several. Read before the `try` and allowed to throw. A variable nobody set and one set
        // empty are the same defect: verification hashes the delivery against whatever key it is
        // handed, so either one refuses every genuine delivery as forged while saying nothing.
        string? subscriptionSecret =
            Environment.GetEnvironmentVariable("HOOK0_SUBSCRIPTION_SECRET");
        if (string.IsNullOrEmpty(subscriptionSecret))
        {
            throw new InvalidOperationException("HOOK0_SUBSCRIPTION_SECRET is not set");
        }

        try
        {
            Webhooks.VerifyWebhookSignature(
                signature,
                payload,
                headers,
                subscriptionSecret,
                TimeSpan.FromMinutes(5));
            return true;
        }
        catch (SignatureException)
        {
            // `SignatureException.Refusal` says which of the five ways it was refused: `Malformed`,
            // `CodeNotHexadecimal`, `HeaderNotDelivered`, `CodeMismatch` or `OutsideTolerance`.
            return false;
        }
    }
}
// hook0:snippet:end

public static class DashboardVerify
{
    public static bool Run()
    {
        // Nothing here is ever run: this file exists to be compiled against the real client.
        return WebhookEndpoint.Accept(
            string.Empty,
            Array.Empty<byte>(),
            new Dictionary<string, string> { ["x-hook0-signature"] = string.Empty });
    }
}
