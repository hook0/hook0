// The Go client against a Hook0 that is really running.
//
// Two things happen here, and the second is the reason the first is worth having.
//
// The control: whether an application secret the API minted is accepted, whether a second send
// under an identifier already ingested is reported as the conflict it is, and whether a signature
// the output worker computed verifies. Those are the three questions no loopback suite can ask
// itself, because a suite that signs and verifies with the same module only proves the module
// agrees with itself.
//
// The surface: every operation the API document declares, driven through the generated layer
// against the same instance, and every model type it decodes out of a real answer.
// `clients/go/generated_test.go` already drives all of them — against an API the suite itself
// writes, out of the same document the client was generated from. That proves the client matches
// the document. It cannot prove the document matches Hook0, and a field the API really answers
// under another name passes there and fails on a consumer's first call.
package main

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"net/url"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"time"

	hook0 "github.com/hook0/hook0-go"
	"github.com/hook0/hook0-go/generated"
)

// How long the two sends together are given.
const sendingWithin = 2 * time.Minute

// How long the whole surface is given, compiling excluded.
const surfaceWithin = 10 * time.Minute

// What this smoke labels everything it creates with, so that the subscription it makes and the
// event it sends find each other.
const language = "go"

// Where the subscription this smoke creates points. Nothing listens there, deliberately: what a
// delivery proves is proved once, by the webhook the harness catches and every language verifies.
const nowhere = "http://127.0.0.1:1/"

// What a paced instance answers, the most times one request is sent again after it, and the
// shortest and longest this waits in between.
const (
	tooManyRequests = 429
	pacedAgain      = 8
	shortestPause   = 200 * time.Millisecond
	longestPause    = 10 * time.Second
)

func main() {
	if err := smoke(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}

func smoke() error {
	apiURL, err := setting("HOOK0_API_URL")
	if err != nil {
		return err
	}
	applicationID, err := setting("HOOK0_APPLICATION_ID")
	if err != nil {
		return err
	}
	token, err := setting("HOOK0_TOKEN")
	if err != nil {
		return err
	}
	eventType, err := setting("HOOK0_EVENT_TYPE")
	if err != nil {
		return err
	}
	delivery, err := setting("HOOK0_DELIVERY")
	if err != nil {
		return err
	}

	client := hook0.NewClient(apiURL, applicationID, token, hook0.DefaultOptions())
	ctx, done := context.WithTimeout(context.Background(), sendingWithin)
	defer done()

	sent, err := client.SendEvent(ctx, event(eventType, ""))
	if err != nil {
		return fmt.Errorf("the instance refused the first send: %w", err)
	}
	fmt.Printf("ingested %s\n", sent)

	_, err = client.SendEvent(ctx, event(eventType, sent))
	if err == nil {
		return fmt.Errorf("sending the same event twice was accepted twice")
	}
	if !strings.Contains(err.Error(), hook0.AlreadyIngested) {
		return fmt.Errorf("the second send failed without naming %s: %w", hook0.AlreadyIngested, err)
	}
	fmt.Printf("the second send reported %s\n", hook0.AlreadyIngested)

	if err := surface(apiURL); err != nil {
		return err
	}

	// Last, and on purpose: it needs no instance at all, so it still answers after the flow above
	// has deleted the application it was run against.
	if err := verify(delivery); err != nil {
		return err
	}
	fmt.Println("the signature the instance produced verifies")
	return nil
}

// The event both sends carry, under the identifier the caller names.
func event(eventType string, eventID string) hook0.Event {
	return hook0.Event{
		EventType:          eventType,
		Payload:            `{"from":"the go smoke"}`,
		PayloadContentType: "application/json",
		Labels:             map[string]string{"language": "go"},
		EventId:            eventID,
	}
}

// Verifies what the output worker really delivered, with this client's own verification.
func verify(delivery string) error {
	read := func(part string) (string, error) {
		what, err := os.ReadFile(filepath.Join(delivery, part))
		if err != nil {
			return "", fmt.Errorf("reading the delivered %s: %w", part, err)
		}
		return string(what), nil
	}

	signature, err := read("signature")
	if err != nil {
		return err
	}
	secret, err := read("secret")
	if err != nil {
		return err
	}
	written, err := read("tolerance")
	if err != nil {
		return err
	}
	tolerance, err := strconv.Atoi(strings.TrimSpace(written))
	if err != nil {
		return fmt.Errorf("the tolerance is not a number of seconds: %w", err)
	}
	body, err := os.ReadFile(filepath.Join(delivery, "body"))
	if err != nil {
		return fmt.Errorf("reading the delivered body: %w", err)
	}
	lines, err := read("headers")
	if err != nil {
		return err
	}

	headers := http.Header{}
	for _, line := range strings.Split(lines, "\n") {
		name, value, found := strings.Cut(line, ": ")
		if found {
			headers.Add(name, value)
		}
	}

	if err := hook0.VerifyWebhookSignature(
		strings.TrimSpace(signature),
		body,
		headers,
		strings.TrimSpace(secret),
		time.Duration(tolerance)*time.Second,
	); err != nil {
		return fmt.Errorf("the signature the instance produced was refused: %w", err)
	}
	return nil
}

// A setting the harness passes, or a refusal naming it: a smoke that ran without one would report a
// failure of the client for something the harness never handed it.
func setting(name string) (string, error) {
	value := os.Getenv(name)
	if value == "" {
		return "", fmt.Errorf("%s is not set", name)
	}
	return value, nil
}

// The instance without the path the hand-written half is built with.
//
// The generated half composes paths that already carry /api/v1, since the API document's own server
// URL is the bare origin. This client's transport happens to tolerate being handed the whole of
// HOOK0_API_URL — an absolute path replaces the base's under RFC 3986 — but pointing it at the
// origin is what the contract says and what keeps an instance mounted under a sub-path working.
func originOf(apiURL string) (string, error) {
	parsed, err := url.Parse(apiURL)
	if err != nil {
		return "", fmt.Errorf("%s is not a URL: %w", apiURL, err)
	}
	return parsed.Scheme + "://" + parsed.Host, nil
}

// What every generated method is issued through, waiting out a paced instance.
//
// Hook0 paces callers per credential, and a flow driving three dozen operations one after another
// is exactly what that is for. The answer says the request was not processed and is safe to send
// again after the delay it names, so this waits and sends it again rather than handing the caller a
// problem that says nothing about the operation it was asking about.
//
// It wraps the transport the package ships rather than replacing it: Deliver is what that transport
// offers a caller who needs what the answer carried beside its body, which is precisely the delay.
type paced struct {
	inner *hook0.Transport
}

func (through paced) Request(
	ctx context.Context,
	method string,
	path string,
	query url.Values,
	body any,
) (int, []byte, error) {
	for sent := 1; ; sent++ {
		status, headers, payload, err := through.inner.Deliver(ctx, method, path, query, body)
		if err != nil || status != tooManyRequests || sent > pacedAgain {
			return status, payload, err
		}

		select {
		case <-time.After(pause(headers)):
		case <-ctx.Done():
			return status, payload, ctx.Err()
		}
	}
}

// How long the answer says to wait, held between a floor and a ceiling of this smoke's own.
//
// The floor is there because the header counts in whole seconds and the delay being waited out is a
// fraction of one, so a truthful `Retry-After: 0` would otherwise mean sending the same request
// again immediately, forever. The ceiling is there because a header is written by a server this
// smoke does not control.
func pause(headers http.Header) time.Duration {
	asked := shortestPause
	if seconds, err := strconv.Atoi(strings.TrimSpace(headers.Get("Retry-After"))); err == nil {
		asked = time.Duration(seconds) * time.Second
	}
	return min(max(asked, shortestPause), longestPause)
}

// Reports one operation the flow goes on to use the answer of, which has to be a success.
func read(operation string, failed error) error {
	if failed != nil {
		return fmt.Errorf(
			"%s: the flow needs what it answers, and it answered %w", operation, failed,
		)
	}
	fmt.Printf("exercised %s accepted\n", operation)
	return nil
}

// Reports one operation driven for its own sake, whichever way the instance answered it.
//
// A success and a problem are both complete round trips through the generated layer: the request
// was composed, the instance answered, and this client read the answer. What is neither — the API
// not reached, a body this client cannot read, a problem it does not know — stops the smoke,
// because none of those say the client and the instance agree on anything.
func exercised(operation string, failed error) error {
	if failed == nil {
		fmt.Printf("exercised %s accepted\n", operation)
		return nil
	}

	var reported *generated.ProblemError
	if !errors.As(failed, &reported) || reported.Kind == "" {
		return fmt.Errorf("%s: %w", operation, failed)
	}
	fmt.Printf("exercised %s refused:%s\n", operation, string(reported.Kind))
	return nil
}

// Reports one generated model type as decoded out of a real answer.
//
// The value is taken rather than only named, so the line cannot outlive what it is about: a field
// that stops being part of an answer stops compiling here.
func decoded[T any](model string, _ T) {
	fmt.Printf("decoded %s\n", model)
}

// Every operation the API document declares, driven against the instance in the order a consumer
// would: what it needs is created, read and listed, updated, and destroyed last.
//
// Two credentials, because the API takes two and one of them cannot do everything. An application
// secret is scoped to the application it belongs to; what belongs to the organization — listing its
// applications, everything about service tokens, its per-day counts — needs the organization-scoped
// token beside it.
func surface(apiURL string) error {
	origin, err := originOf(apiURL)
	if err != nil {
		return err
	}
	application, err := setting("HOOK0_APPLICATION_ID")
	if err != nil {
		return err
	}
	organization, err := setting("HOOK0_ORGANIZATION_ID")
	if err != nil {
		return err
	}
	seeded, err := setting("HOOK0_SEEDED_APPLICATION_ID")
	if err != nil {
		return err
	}
	attempt, err := setting("HOOK0_REQUEST_ATTEMPT_ID")
	if err != nil {
		return err
	}
	response, err := setting("HOOK0_RESPONSE_ID")
	if err != nil {
		return err
	}
	token, err := setting("HOOK0_TOKEN")
	if err != nil {
		return err
	}
	serviceToken, err := setting("HOOK0_SERVICE_TOKEN")
	if err != nil {
		return err
	}

	ctx, done := context.WithTimeout(context.Background(), surfaceWithin)
	defer done()

	held := paced{inner: hook0.NewTransport(origin, token, 0, 0)}
	organizationWide := paced{inner: hook0.NewTransport(origin, serviceToken, 0, 0)}

	applications := generated.NewApplicationsAPI(held)
	secrets := generated.NewApplicationSecretsAPI(held)
	eventTypes := generated.NewEventTypesAPI(held)
	subscriptions := generated.NewSubscriptionsAPI(held)
	events := generated.NewEventsAPI(held)
	eventsPerDay := generated.NewEventsPerDayAPI(held)
	instance := generated.NewInstanceAPI(held)
	quotas := generated.NewQuotasAPI(held)
	payloadContentTypes := generated.NewPayloadContentTypesAPI(held)
	errorCatalogue := generated.NewErrorsAPI(held)

	organizationApplications := generated.NewApplicationsAPI(organizationWide)
	organizationEventsPerDay := generated.NewEventsPerDayAPI(organizationWide)
	requestAttempts := generated.NewRequestAttemptsAPI(organizationWide)
	responses := generated.NewResponseAPI(organizationWide)
	serviceTokens := generated.NewServiceTokenAPI(organizationWide)

	// What the instance says about itself, which is what an application asks before it has anything
	// of its own: how it is configured, what it will let this account do, what a payload may be,
	// and every problem it can report.
	configured, failed := instance.Get(ctx)
	if err := read("instance.get", failed); err != nil {
		return err
	}
	decoded("InstanceConfig", configured)

	allowed, failed := quotas.Get(ctx)
	if err := read("quotas.get", failed); err != nil {
		return err
	}
	decoded("QuotasResponseLimits", allowed.Limits)
	decoded("QuotasResponse", allowed)

	_, failed = payloadContentTypes.List(ctx)
	if err := exercised("payload_content_types.list", failed); err != nil {
		return err
	}

	catalogue, failed := errorCatalogue.List(ctx)
	if err := read("errors.list", failed); err != nil {
		return err
	}
	if len(catalogue) == 0 {
		return fmt.Errorf("the instance published an empty catalogue of the problems it can report")
	}
	decoded("ProblemId", catalogue[0].Id)
	decoded("Problem", catalogue[0])

	// The application this smoke owns. One per language, so that the three deletions at the end of
	// this flow are real deletions rather than something eleven other smokes have to live with.
	info, failed := applications.Get(ctx, application)
	if err := read("applications.get", failed); err != nil {
		return err
	}
	decoded("ApplicationInfoConsumption", info.Consumption)
	decoded("ApplicationInfoQuotas", info.Quotas)
	decoded("ApplicationInfoOnboardingStepsEvent", info.OnboardingSteps.Event)
	decoded("ApplicationInfoOnboardingStepsEventType", info.OnboardingSteps.EventType)
	decoded("ApplicationInfoOnboardingStepsSubscription", info.OnboardingSteps.Subscription)
	decoded("ApplicationInfoOnboardingSteps", info.OnboardingSteps)
	decoded("ApplicationInfo", info)

	renamed, failed := applications.Update(ctx, application, generated.ApplicationPost{
		Name:           "the application the go smoke drives",
		OrganizationId: generated.UUID(organization),
	})
	if err := read("applications.update", failed); err != nil {
		return err
	}
	decoded("Application", renamed)

	// The organization's, so the organization credential. Listing what an account has is the first
	// thing a console does.
	_, failed = organizationApplications.List(ctx, organization)
	if err := exercised("applications.list", failed); err != nil {
		return err
	}

	// This one is driven with the *application* secret on purpose, and it is the flow's one
	// refusal. Creating an application is the organization's business and an application secret is
	// not the organization's, so the instance answers a problem document and this client reads it —
	// which is the half of the client that nothing else here would exercise.
	_, failed = applications.Create(ctx, generated.ApplicationPost{
		Name:           "an application the go smoke's application secret may not create",
		OrganizationId: generated.UUID(organization),
	})
	if err := exercised("applications.create", failed); err != nil {
		return err
	}

	// A second secret, so that the one this smoke is authenticating with is never the one it
	// revokes. Deleting that one succeeds and then locks the flow out of everything below.
	mintedName := "a secret the go smoke minted"
	minted, failed := secrets.Create(ctx, generated.ApplicationSecretPost{
		ApplicationId: generated.UUID(application),
		Name:          &mintedName,
	})
	if err := read("applicationSecrets.create", failed); err != nil {
		return err
	}
	decoded("ApplicationSecret", minted)
	mintedToken := string(minted.Token)

	_, failed = secrets.List(ctx, application)
	if err := exercised("applicationSecrets.list", failed); err != nil {
		return err
	}

	renamedName := "a secret the go smoke renamed"
	_, failed = secrets.Update(ctx, mintedToken, generated.ApplicationSecretPost{
		ApplicationId: generated.UUID(application),
		Name:          &renamedName,
	})
	if err := exercised("applicationSecrets.update", failed); err != nil {
		return err
	}

	if err := exercised(
		"applicationSecrets.delete", secrets.Delete(ctx, mintedToken, application),
	); err != nil {
		return err
	}

	// An event type of this smoke's own, rather than the one the harness declared: what is created
	// here is what is subscribed to, sent, replayed and deleted below.
	declared, failed := eventTypes.Create(ctx, generated.EventTypePost{
		ApplicationId: generated.UUID(application),
		ResourceType:  "smoke",
		Service:       language,
		Verb:          "ran",
	})
	if err := read("eventTypes.create", failed); err != nil {
		return err
	}
	decoded("EventType", declared)

	_, failed = eventTypes.Get(ctx, declared.EventTypeName, application)
	if err := exercised("eventTypes.get", failed); err != nil {
		return err
	}
	_, failed = eventTypes.List(ctx, application)
	if err := exercised("eventTypes.list", failed); err != nil {
		return err
	}

	labels := map[string]string{"language": language}
	target := generated.SubscriptionPostTarget{
		Headers: map[string]string{},
		Method:  "POST",
		Type:    "http",
		Url:     nowhere,
	}
	subscribing := "what the go smoke subscribes to its own events with"
	subscription, failed := subscriptions.Create(ctx, generated.SubscriptionPost{
		ApplicationId: generated.UUID(application),
		Description:   &subscribing,
		EventTypes:    []string{declared.EventTypeName},
		IsEnabled:     true,
		Labels:        labels,
		Target:        target,
	})
	if err := read("subscriptions.create", failed); err != nil {
		return err
	}
	decoded("SubscriptionTarget", subscription.Target)
	decoded("Subscription", subscription)
	subscribed := string(subscription.SubscriptionId)

	_, failed = subscriptions.Get(ctx, subscribed)
	if err := exercised("subscriptions.get", failed); err != nil {
		return err
	}
	_, failed = subscriptions.List(ctx, application)
	if err := exercised("subscriptions.list", failed); err != nil {
		return err
	}

	renamedSubscription := "what the go smoke renamed it to"
	_, failed = subscriptions.Update(ctx, subscribed, generated.SubscriptionPost{
		ApplicationId: generated.UUID(application),
		Description:   &renamedSubscription,
		EventTypes:    []string{declared.EventTypeName},
		IsEnabled:     true,
		Labels:        labels,
		Target:        target,
	})
	if err := exercised("subscriptions.update", failed); err != nil {
		return err
	}

	// The event the subscription above selects, sent through the generated layer rather than
	// through SendEvent: the hand-written half has its own three questions above, and this is the
	// operation the document declares.
	minting := generated.UUID(hook0.GenerateEventId())
	ingested, failed := events.Ingest(ctx, generated.EventPost{
		ApplicationId:      generated.UUID(application),
		EventId:            &minting,
		EventType:          declared.EventTypeName,
		Labels:             labels,
		OccurredAt:         time.Now().UTC(),
		Payload:            `{"from":"the go smoke"}`,
		PayloadContentType: "application/json",
	})
	if err := read("events.ingest", failed); err != nil {
		return err
	}
	decoded("IngestedEvent", ingested)
	sent := string(ingested.EventId)

	whole, failed := events.Get(ctx, sent, application)
	if err := read("events.get", failed); err != nil {
		return err
	}
	decoded("EventWithPayload", whole)

	listed, failed := events.List(ctx, application)
	if err := read("events.list", failed); err != nil {
		return err
	}
	if len(listed) == 0 {
		return fmt.Errorf("the instance ingested an event and then listed none")
	}
	decoded("Event", listed[0])

	if err := exercised("events.replay", events.Replay(ctx, sent, generated.ReplayEvent{
		ApplicationId: generated.UUID(application),
	})); err != nil {
		return err
	}

	// This application was created a moment ago and the counts come out of a view the instance
	// refreshes on a cycle of its own, so this answers a list with nothing in it — which is an
	// answer, and one a client has to be able to read.
	_, failed = eventsPerDay.ListForApplication(ctx, application, nil, nil)
	if err := exercised("events_per_day.list_for_application", failed); err != nil {
		return err
	}

	// The organization's counts do have something in them: the harness waited for the instance to
	// refresh them before running any of this, precisely so that the type they are answered with is
	// one a client decodes rather than one nothing ever produces.
	perDay, failed := organizationEventsPerDay.ListForOrganization(ctx, organization, nil, nil)
	if err := read("events_per_day.list_for_organization", failed); err != nil {
		return err
	}
	if len(perDay) == 0 {
		return fmt.Errorf("the organization has ingested events and its per-day counts are empty")
	}
	decoded("EventsPerDayEntry", perDay[0])

	// An attempt and a response exist only once the output worker has finished a delivery. The
	// harness waited for one, in the application it caught the shared delivery from, and handed the
	// ids on — so this reads them back with the organization credential rather than waiting again.
	_, failed = requestAttempts.List(ctx, seeded, nil, nil, nil, nil, nil, nil)
	if err := exercised("requestAttempts.list", failed); err != nil {
		return err
	}

	attempted, failed := requestAttempts.Get(ctx, attempt, seeded)
	if err := read("requestAttempts.get", failed); err != nil {
		return err
	}
	decoded("RequestAttemptEvent", attempted.Event)
	decoded("RequestAttemptSubscription", attempted.Subscription)
	decoded("RequestAttemptStatusType", attempted.Status.Type)
	decoded("RequestAttemptStatus", attempted.Status)
	decoded("RequestAttempt", attempted)

	answered, failed := responses.Get(ctx, response, seeded)
	if err := read("response.get", failed); err != nil {
		return err
	}
	decoded("Response", answered)

	// Service tokens belong to the organization, so they are minted, read and revoked with the
	// organization credential. The one revoked below is the one minted here — never the one this
	// half of the flow is authenticating with.
	issued, failed := serviceTokens.Create(ctx, generated.ServiceTokenPost{
		Name:           "a token the go smoke minted",
		OrganizationId: generated.UUID(organization),
	})
	if err := read("serviceToken.create", failed); err != nil {
		return err
	}
	decoded("ServiceToken", issued)
	issuedID := string(issued.TokenId)

	_, failed = serviceTokens.List(ctx, organization)
	if err := exercised("serviceToken.list", failed); err != nil {
		return err
	}
	_, failed = serviceTokens.Get(ctx, issuedID, organization)
	if err := exercised("serviceToken.get", failed); err != nil {
		return err
	}
	_, failed = serviceTokens.Update(ctx, issuedID, generated.ServiceTokenPost{
		Name:           "a token the go smoke renamed",
		OrganizationId: generated.UUID(organization),
	})
	if err := exercised("serviceToken.update", failed); err != nil {
		return err
	}
	if err := exercised(
		"serviceToken.delete", serviceTokens.Delete(ctx, issuedID, organization),
	); err != nil {
		return err
	}

	// Destroyed in the order the instance can accept: the subscription that references the event
	// type, then the event type, then the application — which is last because the secret this whole
	// flow authenticates with stops authenticating the moment its application is gone.
	if err := exercised(
		"subscriptions.delete", subscriptions.Delete(ctx, subscribed, application),
	); err != nil {
		return err
	}
	if err := exercised(
		"eventTypes.delete", eventTypes.Delete(ctx, declared.EventTypeName, application),
	); err != nil {
		return err
	}
	return exercised("applications.delete", applications.Delete(ctx, application))
}
