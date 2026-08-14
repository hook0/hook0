// What a send actually puts on the wire, and what it does with what comes back.

package hook0_test

import (
	"errors"
	"strings"
	"testing"

	hook0 "github.com/hook0/hook0/clients/go"
)

func client(api *fakeAPI, options hook0.Options) *hook0.Client {
	return hook0.NewClient(api.baseURL(), applicationId, token, options)
}

func TestASendThatWorksIssuesOneRequest(t *testing.T) {
	api := listen(t)
	api.willAnswer(ingested("a5b4dd60-6ab4-4bd6-9f0b-1a4f8a2a0001"))

	eventId, err := client(api, promptOptions(4)).SendEvent(bounded(t), anEvent())
	if err != nil {
		t.Fatalf("the send failed: %v", err)
	}

	if eventId != "a5b4dd60-6ab4-4bd6-9f0b-1a4f8a2a0001" {
		t.Errorf("the send answered %q, not the identifier the API ingested it under", eventId)
	}
	if count := api.requestCount(); count != 1 {
		t.Errorf("a send that worked issued %d requests, not one", count)
	}

	request := api.requests()[0]
	if request.method != "POST" {
		t.Errorf("the event was sent with %s, not POST", request.method)
	}
	if request.target != "/event" {
		t.Errorf("the event was sent to %q", request.target)
	}
	if carried := request.headers.Get("Authorization"); carried != "Bearer "+token {
		t.Errorf("the request carried %q as its credential", carried)
	}
}

func TestARetriedSendCarriesTheSameIdentifierEveryTime(t *testing.T) {
	api := listen(t)
	api.willAnswer(hangsUp(), ingested("a5b4dd60-6ab4-4bd6-9f0b-1a4f8a2a0002"))

	if _, err := client(api, promptOptions(4)).SendEvent(bounded(t), anEvent()); err != nil {
		t.Fatalf("a send that met one transport failure did not survive it: %v", err)
	}

	if count := api.requestCount(); count != 2 {
		t.Fatalf("a transport failure followed by a success issued %d requests, not two", count)
	}

	first := api.eventIdOf(t, 0)
	second := api.eventIdOf(t, 1)
	if first != second {
		t.Errorf(
			"the retry carried %q where the first attempt carried %q, so Hook0 would ingest the event twice",
			second, first,
		)
	}
	if first == "" {
		t.Error("the request carried no event identifier at all")
	}
}

func TestASendStopsAtTheAttemptBound(t *testing.T) {
	api := listen(t)
	api.willAnswer(serverError(), serverError(), serverError(), serverError(), serverError())

	_, err := client(api, promptOptions(3)).SendEvent(bounded(t), anEvent())
	if err == nil {
		t.Fatal("a send that met nothing but server errors reported success")
	}

	if count := api.requestCount(); count != 3 {
		t.Errorf("a policy of three attempts issued %d requests", count)
	}

	var refusal *hook0.SendError
	if !errors.As(err, &refusal) {
		t.Fatalf("the failure is %T, not the one a send reports", err)
	}
	if refusal.Attempts != 3 {
		t.Errorf("the failure says %d attempts were made", refusal.Attempts)
	}
	if !strings.Contains(refusal.Error(), "gave up after 3 attempts") {
		t.Errorf("the failure does not say it ran out of attempts: %s", refusal)
	}
}

func TestWhatTheAPIRefusesOutrightIsNotRepeated(t *testing.T) {
	api := listen(t)
	api.willAnswer(refused(), ingested("a5b4dd60-6ab4-4bd6-9f0b-1a4f8a2a0003"))

	if _, err := client(api, promptOptions(4)).SendEvent(bounded(t), anEvent()); err == nil {
		t.Fatal("a send the API refused reported success")
	}

	if count := api.requestCount(); count != 1 {
		t.Errorf("a refusal nothing could change was repeated: %d requests were issued", count)
	}
}

func TestAnAlreadyIngestedConflictIsSuccessOnlyOnARetry(t *testing.T) {
	repeated := listen(t)
	repeated.willAnswer(hangsUp(), alreadyIngested())

	eventId, err := client(repeated, promptOptions(4)).SendEvent(bounded(t), anEvent())
	if err != nil {
		t.Fatalf("a conflict answering a repeated request was not read as the success it is: %v", err)
	}
	if eventId != repeated.eventIdOf(t, 0) {
		t.Errorf("the send answered %q, not the identifier both attempts carried", eventId)
	}

	first := listen(t)
	first.willAnswer(alreadyIngested())

	if _, err := client(first, promptOptions(4)).SendEvent(bounded(t), anEvent()); err == nil {
		t.Fatal("a conflict answering a first attempt was read as a success rather than as the conflict it is")
	}
	if count := first.requestCount(); count != 1 {
		t.Errorf("a conflict on a first attempt issued %d requests", count)
	}
}

func TestRetryingIsDisabledByAPolicyOfOneAttempt(t *testing.T) {
	api := listen(t)
	api.willAnswer(serverError(), ingested("a5b4dd60-6ab4-4bd6-9f0b-1a4f8a2a0004"))

	options := hook0.DefaultOptions()
	options.RetryPolicy = hook0.DisabledRetryPolicy()

	if _, err := client(api, options).SendEvent(bounded(t), anEvent()); err == nil {
		t.Fatal("a send that never retries reported the success of an attempt it never made")
	}
	if count := api.requestCount(); count != 1 {
		t.Errorf("a disabled retry policy issued %d requests", count)
	}
}

func TestAnOversizedPayloadIsRefusedBeforeAnySocketIsOpened(t *testing.T) {
	api := listen(t)
	api.willAnswer(ingested("a5b4dd60-6ab4-4bd6-9f0b-1a4f8a2a0005"))

	options := promptOptions(4)
	options.MaxPayloadBytes = 32

	event := anEvent()
	event.Payload = strings.Repeat("x", options.MaxPayloadBytes+1)

	_, err := client(api, options).SendEvent(bounded(t), event)
	if err == nil {
		t.Fatal("a payload above what the client sends was sent anyway")
	}
	if !errors.Is(err, hook0.ErrPayloadTooLarge) {
		t.Errorf("the refusal is %v, which does not name the bound it crossed", err)
	}
	if count := api.requestCount(); count != 0 {
		t.Errorf("an oversized payload opened %d sockets", count)
	}
}

func TestUpsertingEventTypesCreatesOnlyWhatIsMissing(t *testing.T) {
	api := listen(t)
	api.willAnswer(
		scriptedResponse{
			status: 200,
			body:   []any{map[string]any{"event_type_name": "auth.user.create"}},
		},
		scriptedResponse{status: 201, body: map[string]any{}},
	)

	created, err := client(api, promptOptions(4)).
		UpsertEventTypes(bounded(t), []string{"auth.user.create", "billing.invoice.paid"})
	if err != nil {
		t.Fatalf("upserting event types failed: %v", err)
	}

	if len(created) != 1 || created[0] != "billing.invoice.paid" {
		t.Errorf("upserting answered %v, not the one event type the application did not declare", created)
	}
	if count := api.requestCount(); count != 2 {
		t.Errorf("upserting issued %d requests, not one listing and one creation", count)
	}
}

func TestAnEventTypeThatNamesNoThreeParts(t *testing.T) {
	if _, err := hook0.ParseEventType("auth.user"); !errors.Is(err, hook0.ErrInvalidEventType) {
		t.Errorf("an event type of two parts was read as %v", err)
	}
}

func TestAGeneratedIdentifierIsAUUIDv7(t *testing.T) {
	generated := hook0.GenerateEventId()

	if len(generated) != 36 {
		t.Fatalf("%q is not the canonical form of an identifier", generated)
	}
	if generated[14] != '7' {
		t.Errorf("%q does not carry the version whose leading bits are the current time", generated)
	}
	if variant := generated[19]; !strings.ContainsRune("89ab", rune(variant)) {
		t.Errorf("%q carries %q where the variant belongs", generated, variant)
	}
	if generated == hook0.GenerateEventId() {
		t.Error("two identifiers generated in a row are the same one")
	}
}
