using System;
using System.Globalization;

namespace Hook0;

/// <summary>The failures this client reports about itself.</summary>
/// <remarks>
/// Sending an event, upserting an event type and verifying a webhook all raise a kind of this, so a
/// caller has one thing to catch whatever it asked the client to do. The failures the <em>API</em>
/// reports are a different matter: those are the problems it names in its own error contract, and
/// the generated half of this package raises one exception per problem.
/// </remarks>
public class Hook0Exception : Exception
{
    /// <summary>Reports a failure this client met.</summary>
    /// <param name="detail">What went wrong, in the words a caller is given.</param>
    public Hook0Exception(string detail)
        : base(detail)
    {
    }

    /// <summary>Reports a failure this client met.</summary>
    /// <param name="detail">What went wrong, in the words a caller is given.</param>
    /// <param name="cause">What was reported underneath it.</param>
    public Hook0Exception(string detail, Exception cause)
        : base(detail, cause)
    {
    }
}

/// <summary>A send that did not end with the event ingested.</summary>
/// <remarks>
/// It says how many requests the send issued, which is what tells a transient outage from a request
/// that will never be accepted — and what a caller reads rather than counting connections on the
/// other end. A send refused before anything was sent issued none, and says so.
/// </remarks>
public sealed class SendException : Hook0Exception
{
    private SendException(string eventId, int attempts, TimeSpan waited, string detail)
        : base(detail)
    {
        EventId = eventId;
        Attempts = attempts;
        Waited = waited;
    }

    /// <summary>The identifier the event was to be sent under.</summary>
    public string EventId { get; }

    /// <summary>How many requests this send issued.</summary>
    public int Attempts { get; }

    /// <summary>How long the send spent waiting between its attempts.</summary>
    public TimeSpan Waited { get; }

    /// <summary>A send the API refused for a reason repeating it would not change.</summary>
    /// <param name="eventId">The identifier the event was sent under.</param>
    /// <param name="detail">What the API answered.</param>
    /// <returns>The failure a caller is handed.</returns>
    public static SendException Refused(string eventId, string detail) =>
        new(eventId, 1, TimeSpan.Zero, $"Sending event {eventId} failed: {detail}");

    /// <summary>A send that ran out of attempts, or out of the delay budget its attempts share.</summary>
    /// <param name="eventId">The identifier the event was sent under.</param>
    /// <param name="attempts">How many requests the send issued.</param>
    /// <param name="waited">How long it spent waiting between them.</param>
    /// <param name="detail">What the last attempt answered.</param>
    /// <returns>The failure a caller is handed.</returns>
    public static SendException GaveUp(string eventId, int attempts, TimeSpan waited, string detail) =>
        new(
            eventId,
            attempts,
            waited,
            string.Create(
                CultureInfo.InvariantCulture,
                $"Sending event {eventId} failed: gave up after {attempts} attempts spread over " +
                $"{waited.TotalSeconds:F3}s of retry delay; last failure: {detail}"));

    /// <summary>A payload above what the client agrees to send, refused before a socket is opened.</summary>
    /// <param name="eventId">The identifier the event would have been sent under.</param>
    /// <param name="size">How large the payload is.</param>
    /// <param name="maximum">How large one may be.</param>
    /// <returns>The failure a caller is handed.</returns>
    public static SendException PayloadTooLarge(string eventId, int size, int maximum) =>
        new(
            eventId,
            0,
            TimeSpan.Zero,
            string.Create(
                CultureInfo.InvariantCulture,
                $"Sending event {eventId} failed: event payload is {size} bytes, which is more than " +
                $"the {maximum} bytes this client sends at most; nothing was sent"));
}

/// <summary>An event type that could not be read, listed or declared.</summary>
public sealed class EventTypeException : Hook0Exception
{
    private EventTypeException(string detail)
        : base(detail)
    {
    }

    /// <summary>An event type that does not read as <c>service.resource_type.verb</c>.</summary>
    /// <param name="eventType">What the caller passed.</param>
    /// <returns>The failure a caller is handed.</returns>
    public static EventTypeException Invalid(string eventType) =>
        new($"Provided event type '{eventType}' does not have a valid syntax (service.resource_type.verb)");

    /// <summary>The list of event types the application already declares could not be read.</summary>
    /// <param name="detail">What went wrong.</param>
    /// <returns>The failure a caller is handed.</returns>
    public static EventTypeException Unavailable(string detail) =>
        new($"Getting available event types failed: {detail}");

    /// <summary>An event type that could not be created.</summary>
    /// <param name="eventType">The event type being declared.</param>
    /// <param name="detail">What went wrong.</param>
    /// <returns>The failure a caller is handed.</returns>
    public static EventTypeException NotCreated(string eventType, string detail) =>
        new($"Creating event type '{eventType}' failed: {detail}");
}
