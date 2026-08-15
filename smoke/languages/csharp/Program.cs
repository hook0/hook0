// The C# client against a Hook0 that is really running.
//
// Three things the loopback suite cannot ask: whether an application secret the API minted is
// accepted, whether a second send under an identifier already ingested is reported as the conflict
// it is, and whether a signature the output worker computed verifies. Everything else about this
// client is settled by clients/csharp/tests.

using System;
using System.Collections.Generic;
using System.IO;
using Hook0;

namespace Hook0.Smoke;

internal static class Program
{
    /// <summary>The conflict the API answers a duplicated ingestion with.</summary>
    private const string AlreadyIngested = "EventAlreadyIngested";

    private static int Main()
    {
        try
        {
            Smoke();
            return 0;
        }
        catch (Exception refused)
        {
            Console.Error.WriteLine(refused);
            return 1;
        }
    }

    private static void Smoke()
    {
        string eventType = Setting("HOOK0_EVENT_TYPE");

        using Hook0Client client = new(
            apiUrl: Setting("HOOK0_API_URL"),
            applicationId: Setting("HOOK0_APPLICATION_ID"),
            token: Setting("HOOK0_TOKEN"));

        string sent = client.SendEvent(Sending(eventType, null));
        Console.WriteLine($"ingested {sent}");

        string said;
        try
        {
            client.SendEvent(Sending(eventType, Guid.Parse(sent)));
            throw new InvalidOperationException("sending the same event twice was accepted twice");
        }
        catch (SendException refused)
        {
            said = refused.Message;
        }

        if (!said.Contains(AlreadyIngested, StringComparison.Ordinal))
        {
            throw new InvalidOperationException(
                $"the second send failed without naming {AlreadyIngested}: {said}");
        }
        Console.WriteLine($"the second send reported {AlreadyIngested}");

        Verify(Setting("HOOK0_DELIVERY"));
        Console.WriteLine("the signature the instance produced verifies");
    }

    /// <summary>The event both sends carry, under the identifier the caller names.</summary>
    private static Event Sending(string eventType, Guid? eventId) => new()
    {
        EventType = eventType,
        Payload = """{"from":"the csharp smoke"}""",
        PayloadContentType = "application/json",
        Labels = new Dictionary<string, string>(StringComparer.Ordinal) { ["language"] = "csharp" },
        EventId = eventId,
    };

    /// <summary>
    /// Verifies what the output worker really delivered, with this client's own verification.
    /// </summary>
    private static void Verify(string delivery)
    {
        string Read(string part) => File.ReadAllText(Path.Combine(delivery, part));

        List<KeyValuePair<string, string>> headers = new();
        foreach (string line in Read("headers").Split('\n'))
        {
            int at = line.IndexOf(": ", StringComparison.Ordinal);
            if (at > 0)
            {
                headers.Add(new KeyValuePair<string, string>(line[..at], line[(at + 2)..]));
            }
        }

        Webhooks.VerifyWebhookSignature(
            signature: Read("signature").Trim(),
            payload: File.ReadAllBytes(Path.Combine(delivery, "body")),
            headers: headers,
            subscriptionSecret: Read("secret").Trim(),
            tolerance: TimeSpan.FromSeconds(int.Parse(Read("tolerance").Trim(),
                System.Globalization.CultureInfo.InvariantCulture)));
    }

    /// <summary>A setting the harness passes, or a refusal naming it.</summary>
    private static string Setting(string name)
    {
        string? value = Environment.GetEnvironmentVariable(name);
        if (string.IsNullOrEmpty(value))
        {
            throw new InvalidOperationException($"{name} is not set");
        }
        return value;
    }
}
