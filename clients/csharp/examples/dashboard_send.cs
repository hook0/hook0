// What the dashboard shows under "Send an event", for C#.
//
// This file exists so that the snippet is compiled against the real client. A renamed method, a
// changed signature or a dropped member turns `clients.csharp.check` red on the day it happens,
// which is the whole reason the snippet lives here rather than in the dashboard: one written by
// hand over there is backed by nothing and drifts in silence.
//
// Two pairs of markers say how it is read. `hook0:snippet` delimits what a reader is shown, so that
// anything this file needs only in order to compile stays out of it. `hook0:label` delimits the one
// rendering of a label, which the dashboard repeats once per label the form carries and joins with
// the separator its manifest declares — the region carries no trailing separator of its own, and
// sits inside its container, so no label at all leaves a valid empty one.
//
// The `__HOOK0_*__` words are string literals, which is what lets a file full of them compile. They
// never resolve to anything: this example is built, never run.

// hook0:snippet:begin
using System.Collections.Generic;
using Hook0;

public static class SendAnEvent
{
    public static string Send()
    {
        // `Hook0Client` is `IDisposable`, and one of them is meant to live as long as the
        // application rather than as long as a send.
        using Hook0Client client = new(
            apiUrl: "__HOOK0_API_URL__",
            applicationId: "__HOOK0_APPLICATION_ID__",
            token: "__HOOK0_TOKEN__");

        return client.SendEvent(new Event
        {
            EventType = "__HOOK0_EVENT_TYPE__",
            Payload = "__HOOK0_PAYLOAD__",
            PayloadContentType = "application/json",
            Labels = new Dictionary<string, string>
            {
                // hook0:label:begin
                ["__HOOK0_LABEL_KEY__"] = "__HOOK0_LABEL_VALUE__" // hook0:label:end
            },
        });
    }
}
// hook0:snippet:end
