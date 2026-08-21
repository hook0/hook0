// The rest of the file, for every C# example of the SDK reference.
//
// A snippet on a page is written for a reader: it leaves out a `using` a neighbouring snippet
// already showed, it assumes a client or an event is already built, and it names a token or an
// application ID without saying where it came from. Each region below is the file that snippet
// would live in, with a hole where it goes. The page points at one by name on the fence, so what a
// snippet is standing on is one word away from the snippet itself.
//
// Only one file of a C# project may hold top-level statements, so none of these regions is written
// as one: every example lands inside a method of a class of its own namespace, which is also why
// every region opens with its own `namespace {{Name}};`.

// HARNESS send
using Hook0;

namespace {{Name}};

internal static class Program
{
    internal static void Run()
    {
        EXAMPLE
    }
}

// END HARNESS

// HARNESS send_async
using Hook0;

namespace {{Name}};

internal static class Program
{
    // What this send was handed before it started.
    internal static async Task RunAsync(Hook0Client client, Event anEvent, CancellationToken cancellationToken)
    {
        EXAMPLE
    }
}

// END HARNESS

// HARNESS event
using Hook0;

namespace {{Name}};

internal static class Program
{
    // The value the page shows, held so that every field of it is checked against the client.
    internal static readonly Event Configured =
        EXAMPLE
        ;
}

// END HARNESS

// HARNESS configure
using Hook0;

namespace {{Name}};

internal static class Program
{
    // What this client is handed before it starts.
    internal static void Configure(string applicationId, string token)
    {
        EXAMPLE
    }
}

// END HARNESS

// HARNESS verify
using Hook0;

namespace {{Name}};

// What the reader's own HTTP framework would have handed this handler; not part of the client.
// `Dictionary<string, string>` already reads as both an indexer and an
// `IEnumerable<KeyValuePair<string, string>>`, which is everything the page asks of `request.Headers`.
internal sealed class AssumedRequest
{
    internal Dictionary<string, string> Headers { get; } = new();
}

internal static class Program
{
    internal static void Handle(AssumedRequest request, byte[] rawRequestBody, string subscriptionSecret)
    {
        EXAMPLE
    }
}

// END HARNESS

// HARNESS aspnet
using Hook0;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Http;

namespace {{Name}};

internal static class Program
{
    internal static void Configure(WebApplication app, string subscriptionSecret)
    {
        EXAMPLE
    }

    // Where the handler above hands a verified delivery off to; not part of the client.
    private static Task HandleDeliveryAsync(byte[] payload, CancellationToken cancellationToken) =>
        Task.CompletedTask;
}

// END HARNESS

// HARNESS upsert
using Hook0;

namespace {{Name}};

internal static class Program
{
    internal static void Declare(Hook0Client client)
    {
        EXAMPLE
    }
}

// END HARNESS

// HARNESS generated
using Hook0;
using Hook0.Generated;

namespace {{Name}};

internal static class Program
{
    // The client this page assumes is already built and reachable.
    internal static async Task ListAsync(Hook0Client client, CancellationToken cancellationToken)
    {
        EXAMPLE
    }
}

// END HARNESS

// HARNESS errors
using Hook0;
using Microsoft.Extensions.Logging;

namespace {{Name}};

internal static class Program
{
    internal static void Send(Hook0Client client, Event anEvent, ILogger logger)
    {
        EXAMPLE
    }
}

// END HARNESS
