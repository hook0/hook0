// What a send actually puts on the wire, and what it does with what comes back.

package hook0_test

import (
	"context"
	"errors"
	"math"
	"net/url"
	"strconv"
	"strings"
	"testing"
	"time"

	hook0 "github.com/hook0/hook0-go/v2"
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

func TestAPolicyAtTheEdgesOfItsTypeStillStatesFourIntegers(t *testing.T) {
	// A Duration is a signed count of nanoseconds, so the far ends of it are a delay longer than any
	// caller meant and one that runs backwards. Neither is a policy anybody configures on purpose,
	// and both are what a header composed out of arithmetic gets wrong: what has to hold is that the
	// value stays four integers a reader can cut apart, rather than a crash or a word.
	edges := hook0.DefaultOptions()
	edges.RetryPolicy = hook0.RetryPolicy{
		MaxAttempts:    math.MaxInt,
		InitialBackoff: math.MaxInt64,
		MaxBackoff:     math.MinInt64,
		MaxTotalDelay:  -time.Second,
	}

	api := listen(t)
	api.willAnswer(ingested("a5b4dd60-6ab4-4bd6-9f0b-1a4f8a2a0002"))

	if _, err := client(api, edges).SendEvent(bounded(t), anEvent()); err != nil {
		t.Fatalf("a send under a policy at the edges of its type failed: %v", err)
	}

	stated := api.requests()[0].headers.Get("Hook0-Client-Options")
	for _, part := range strings.Split(stated, ",") {
		name, written, cut := strings.Cut(part, "=")
		if !cut {
			t.Fatalf("`%s` carries a part naming nothing, in `%s`", part, stated)
		}
		if _, err := strconv.ParseUint(written, 10, 64); err != nil {
			t.Errorf("`%s` states %q, which is no whole number of its own: %v", name, written, err)
		}
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

func TestAClientAnswersTheBoundsAndTheAddressesItWasBuiltWith(t *testing.T) {
	api := listen(t)
	options := promptOptions(3)

	built := client(api, options)

	if built.APIURL() != api.baseURL() {
		t.Errorf("the client reaches %q, not the API it was built for", built.APIURL())
	}
	if built.ApplicationId() != applicationId {
		t.Errorf("the client sends to %q, not the application it was built for", built.ApplicationId())
	}
	if built.Options() != options {
		t.Errorf("the client holds %+v, not the bounds it was built with", built.Options())
	}
	if built.Transport() == nil {
		t.Error("the client issues its requests through nothing")
	}
}

func TestUpsertingNoEventTypeAtAllReachesTheAPIForNothing(t *testing.T) {
	api := listen(t)

	created, err := client(api, promptOptions(1)).UpsertEventTypes(bounded(t), nil)
	if err != nil {
		t.Fatalf("upserting nothing failed: %v", err)
	}

	if len(created) != 0 {
		t.Errorf("upserting nothing answered %v", created)
	}
	if count := api.requestCount(); count != 0 {
		t.Errorf("upserting nothing issued %d requests", count)
	}
}

func TestARefusedListingOfEventTypesIsReportedWithWhatTheAPISaid(t *testing.T) {
	api := listen(t)
	api.willAnswer(scriptedResponse{
		status: 403,
		body:   map[string]any{"id": "Forbidden", "detail": "this token may not read them"},
	})

	_, err := client(api, promptOptions(1)).UpsertEventTypes(bounded(t), []string{"auth.user.create"})
	if err == nil {
		t.Fatal("a refused listing was read as an application declaring nothing")
	}

	var reported *hook0.EventTypeError
	if !errors.As(err, &reported) {
		t.Fatalf("the failure is %T, not the one an event type fails as", err)
	}
	if !strings.Contains(reported.Detail, "this token may not read them") {
		t.Errorf("the failure says %q, without what the API answered", reported.Detail)
	}
	if !strings.Contains(err.Error(), "failed") {
		t.Errorf("the failure reads as %q", err)
	}
	// Nothing is created off a listing that never arrived.
	if count := api.requestCount(); count != 1 {
		t.Errorf("a refused listing was followed by %d requests", count-1)
	}
}

func TestAListingOfEventTypesThatIsNotAListIsReported(t *testing.T) {
	api := listen(t)
	api.willAnswer(scriptedResponse{status: 200, body: map[string]any{"event_type_name": "auth.user.create"}})

	_, err := client(api, promptOptions(1)).UpsertEventTypes(bounded(t), []string{"auth.user.create"})
	if err == nil {
		t.Fatal("a listing that is not one was read as if it were")
	}

	var reported *hook0.EventTypeError
	if !errors.As(err, &reported) {
		t.Fatalf("the failure is %T, not the one an event type fails as", err)
	}
	if !strings.Contains(reported.Detail, "did not answer a list of event types") {
		t.Errorf("the failure says %q", reported.Detail)
	}
	if reported.Unwrap() == nil {
		t.Error("the failure names no reason underneath")
	}
}

func TestEventTypesCannotBeReadFromAnAPINothingIsListeningOn(t *testing.T) {
	unreachable := hook0.NewClient("http://127.0.0.1:1", applicationId, token, promptOptions(1))

	_, err := unreachable.UpsertEventTypes(bounded(t), []string{"auth.user.create"})
	if err == nil {
		t.Fatal("an API nothing is listening on answered a listing")
	}

	var reported *hook0.EventTypeError
	if !errors.As(err, &reported) {
		t.Fatalf("the failure is %T, not the one an event type fails as", err)
	}
	if reported.Unwrap() == nil {
		t.Error("the failure names no reason underneath")
	}
}

func TestAnEventTypeTheAPIRefusesToCreateIsReportedByName(t *testing.T) {
	api := listen(t)
	api.willAnswer(
		scriptedResponse{status: 200, body: []any{}},
		scriptedResponse{status: 409, body: map[string]any{"id": "EventTypeAlreadyExist"}},
	)

	_, err := client(api, promptOptions(1)).UpsertEventTypes(bounded(t), []string{"auth.user.create"})
	if err == nil {
		t.Fatal("an event type the API refused to create was answered as created")
	}

	var reported *hook0.EventTypeError
	if !errors.As(err, &reported) {
		t.Fatalf("the failure is %T, not the one an event type fails as", err)
	}
	if reported.EventType != "auth.user.create" {
		t.Errorf("the failure names %q, not the event type that could not be created", reported.EventType)
	}
	if !strings.Contains(err.Error(), "auth.user.create") {
		t.Errorf("the failure reads as %q, without the event type it is about", err)
	}
}

func TestAnEventTypeCannotBeCreatedOnAnAPIThatStopsAnswering(t *testing.T) {
	api := listen(t)
	api.willAnswer(scriptedResponse{status: 200, body: []any{}}, hangsUp())

	_, err := client(api, promptOptions(1)).UpsertEventTypes(bounded(t), []string{"auth.user.create"})
	if err == nil {
		t.Fatal("an API that hung up answered that the event type was created")
	}

	var reported *hook0.EventTypeError
	if !errors.As(err, &reported) {
		t.Fatalf("the failure is %T, not the one an event type fails as", err)
	}
	if reported.Unwrap() == nil {
		t.Error("the failure names no reason underneath")
	}
}

func TestAnAcceptedEventTheAPINamedNoIdentifierForIsNotReportedAsSent(t *testing.T) {
	// Repeating it would meet the same answer, so it is given up on rather than retried.
	for _, answered := range []scriptedResponse{
		{status: 201, body: map[string]any{"application_id": applicationId}},
		{status: 201, body: "a gateway wrote this"},
		{status: 201, body: []any{"an array"}},
	} {
		api := listen(t)
		api.willAnswer(answered)

		if _, err := client(api, promptOptions(4)).SendEvent(bounded(t), anEvent()); err == nil {
			t.Fatalf("%v was read as an event that was ingested", answered.body)
		}
		if count := api.requestCount(); count != 1 {
			t.Errorf("an answer that repeating cannot fix was tried %d times", count)
		}
	}
}

func TestAConflictNamingNoProblemThisClientReadsIsNotTakenForAnEarlierSend(t *testing.T) {
	// A 409 counts as an event already in only when the body says which problem it was.
	for _, answered := range []scriptedResponse{
		{status: 409, body: "a gateway wrote this"},
		{status: 409, body: []any{"an array"}},
		{status: 409, body: map[string]any{}},
	} {
		api := listen(t)
		api.willAnswer(answered)

		if _, err := client(api, promptOptions(1)).SendEvent(bounded(t), anEvent()); err == nil {
			t.Fatalf("%v was read as an event an earlier attempt had ingested", answered.body)
		}
	}
}

func TestAnEventCarryingNeitherLabelsNorMetadataIsStillSent(t *testing.T) {
	api := listen(t)
	api.willAnswer(ingested("a5b4dd60-6ab4-4bd6-9f0b-1a4f8a2a0009"))
	bare := anEvent()
	bare.Labels = nil
	bare.Metadata = nil
	bare.OccurredAt = time.Time{}

	if _, err := client(api, promptOptions(1)).SendEvent(bounded(t), bare); err != nil {
		t.Fatalf("an event carrying nothing optional failed: %v", err)
	}

	sent := api.requests()[0].json(t)
	// An event that names no moment is sent as having occurred when it was sent, rather than at
	// the zero of a clock.
	if occurred, _ := sent["occurred_at"].(string); occurred == "" || strings.HasPrefix(occurred, "0001-") {
		t.Errorf("the event was sent as having occurred at %q", occurred)
	}
	if labels, ok := sent["labels"].(map[string]any); !ok || len(labels) != 0 {
		t.Errorf("the event was sent with %v where an empty set of labels belongs", sent["labels"])
	}
	if _, carried := sent["metadata"]; carried {
		t.Error("the event was sent with metadata it never carried")
	}
}

func TestAnEventCarryingMetadataSendsIt(t *testing.T) {
	api := listen(t)
	api.willAnswer(ingested("a5b4dd60-6ab4-4bd6-9f0b-1a4f8a2a0010"))
	described := anEvent()
	described.Metadata = map[string]string{"traced by": "the case"}

	if _, err := client(api, promptOptions(1)).SendEvent(bounded(t), described); err != nil {
		t.Fatalf("an event carrying metadata failed: %v", err)
	}

	carried, ok := api.requests()[0].json(t)["metadata"].(map[string]any)
	if !ok || carried["traced by"] != "the case" {
		t.Errorf("the event was sent with %v as its metadata", api.requests()[0].json(t)["metadata"])
	}
}

func TestARequestBodyThatCannotBeWrittenIsNotSent(t *testing.T) {
	api := listen(t)

	// A channel is a value no JSON writer produces anything for, and a request that cannot be
	// written is not one repeating could fix.
	_, _, err := client(api, promptOptions(1)).Transport().Request(bounded(t), "POST", "/event", url.Values{}, make(chan int))
	if err == nil {
		t.Fatal("a body that cannot be written as JSON was sent anyway")
	}

	if !strings.Contains(err.Error(), "cannot be written") {
		t.Errorf("the failure reads as %q", err)
	}
	if count := api.requestCount(); count != 0 {
		t.Errorf("%d requests were issued for a body that was never written", count)
	}
}

func TestARequestThatCannotBeBuiltIsNotSent(t *testing.T) {
	api := listen(t)

	// A method is a token, and a space is not one of the characters a token is written with.
	_, _, err := client(api, promptOptions(1)).Transport().Request(bounded(t), "NOT A METHOD", "/event", url.Values{}, nil)
	if err == nil {
		t.Fatal("a request naming no method was issued anyway")
	}

	if !strings.Contains(err.Error(), "cannot be built") {
		t.Errorf("the failure reads as %q", err)
	}
	if count := api.requestCount(); count != 0 {
		t.Errorf("%d requests were issued for a request that was never built", count)
	}
}

func TestAPathThatIsNotOneIsRefusedBeforeAnySocketIsOpened(t *testing.T) {
	api := listen(t)

	_, _, err := client(api, promptOptions(1)).Transport().Request(bounded(t), "GET", "://not-a-path", url.Values{}, nil)
	if err == nil {
		t.Fatal("a path that is not one was reached anyway")
	}

	if !errors.Is(err, hook0.ErrUnusableAPIURL) {
		t.Errorf("the failure is %v, which is not the one a URL nothing can be sent to fails as", err)
	}
	if count := api.requestCount(); count != 0 {
		t.Errorf("%d requests were issued for a path that is not one", count)
	}
}

func TestASendGivenNoMoreTimeStopsWaitingForTheNextAttempt(t *testing.T) {
	api := listen(t)
	api.willAnswer(serverError(), serverError(), serverError(), serverError())

	// Long enough that the send is between two attempts when the deadline arrives, which is the
	// wait a cancelled caller has to be let out of rather than held through.
	patient := hook0.DefaultOptions()
	patient.RetryPolicy = hook0.RetryPolicy{
		MaxAttempts:    4,
		InitialBackoff: 2 * time.Second,
		MaxBackoff:     2 * time.Second,
		MaxTotalDelay:  time.Minute,
	}
	ctx, done := context.WithTimeout(context.Background(), 150*time.Millisecond)
	defer done()

	_, err := client(api, patient).SendEvent(ctx, anEvent())
	if err == nil {
		t.Fatal("a send whose caller stopped waiting answered a success")
	}

	var reported *hook0.SendError
	if !errors.As(err, &reported) {
		t.Fatalf("the failure is %T, not the one a send fails as", err)
	}
	if !errors.Is(err, context.DeadlineExceeded) {
		t.Errorf("the failure is %v, which does not name the caller that stopped waiting", err)
	}
	if count := api.requestCount(); count >= 4 {
		t.Errorf("a send that was cut short still issued %d requests", count)
	}
}

func TestUpsertingAnEventTypeThatNamesNoThreePartsReachesTheAPIForNothing(t *testing.T) {
	api := listen(t)

	_, err := client(api, promptOptions(1)).UpsertEventTypes(bounded(t), []string{"auth.user.create", "not-an-event-type"})
	if !errors.Is(err, hook0.ErrInvalidEventType) {
		t.Fatalf("an event type of two parts was read as %v", err)
	}

	if count := api.requestCount(); count != 0 {
		t.Errorf("%d requests were issued for a list this client would not use", count)
	}
}

func TestABodyThatStoppedWhereTheConnectionDidIsMetAgain(t *testing.T) {
	api := listen(t)
	api.willAnswer(
		scriptedResponse{stopsMidBody: true},
		ingested("a5b4dd60-6ab4-4bd6-9f0b-1a4f8a2a0011"),
	)

	// The next answer could carry the whole of it, and the identifier the client chose is what
	// keeps the retry from ingesting the event twice.
	eventId, err := client(api, promptOptions(4)).SendEvent(bounded(t), anEvent())
	if err != nil {
		t.Fatalf("a body that stopped mid-answer was not met again: %v", err)
	}

	if eventId != "a5b4dd60-6ab4-4bd6-9f0b-1a4f8a2a0011" {
		t.Errorf("the send answered %q", eventId)
	}
	if count := api.requestCount(); count != 2 {
		t.Errorf("a body that stopped mid-answer was tried %d times", count)
	}
	if first, second := api.eventIdOf(t, 0), api.eventIdOf(t, 1); first != second {
		t.Errorf("the retry carried %q where the first attempt carried %q", second, first)
	}
}
