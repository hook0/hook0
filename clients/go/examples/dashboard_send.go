// What the dashboard shows under "Send an event", for Go.
//
// This file exists so that the snippet is compiled against the real client. A renamed method, a
// changed signature or a dropped field turns `clients.go.check` red on the day it happens, which is
// the whole reason the snippet lives here rather than in the dashboard: one written by hand over
// there is backed by nothing and drifts in silence.
//
// Two pairs of markers say how it is read. `hook0:snippet` delimits what a reader is shown, so that
// anything this file needs only in order to compile stays out of it. `hook0:label` delimits the one
// rendering of a label, which the dashboard repeats once per label the form carries and joins with
// the separator its manifest declares — the region carries no trailing separator of its own, and
// sits inside its container, so no label at all leaves a valid empty one. Here that container is a
// map built a statement at a time rather than a literal, for the reason `dashboard.toml` gives.
//
// The `__HOOK0_*__` words are string literals, which is what lets a file full of them compile. They
// never resolve to anything: this example is built, never run.
//
// Go admits one `main` per directory, so this file and `dashboard_verify.go` are one package rather
// than two programs. The verify half is therefore a function this program never calls — compiled by
// the same `go vet ./...` and `go test ./...` all the same, which is all that is asked of it.

// hook0:snippet:begin
package main

import (
	"context"
	"log"

	hook0 "github.com/hook0/hook0-go/v2"
)

func main() {
	client := hook0.NewClient(
		"__HOOK0_API_URL__",
		"__HOOK0_APPLICATION_ID__",
		"__HOOK0_TOKEN__",
		hook0.DefaultOptions(),
	)

	labels := map[string]string{}
	// hook0:label:begin
	labels["__HOOK0_LABEL_KEY__"] = "__HOOK0_LABEL_VALUE__" // hook0:label:end

	eventId, err := client.SendEvent(context.Background(), hook0.Event{
		EventType:          "__HOOK0_EVENT_TYPE__",
		Payload:            "__HOOK0_PAYLOAD__",
		PayloadContentType: "application/json",
		Labels:             labels,
	})
	if err != nil {
		log.Fatalf("event not sent: %v", err)
	}

	log.Printf("ingested as %s", eventId)
}

// hook0:snippet:end
