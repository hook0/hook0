// Package hook0 is the Go SDK for Hook0, an open source Webhooks-as-a-Service platform.
//
// Two halves live here. This one is hand-written: sending an event, upserting the event types an
// application uses, and verifying that a webhook came from Hook0 unchanged. The other is generated
// from the OpenAPI snapshot the API commits — one type per schema it declares, one error value per
// problem it can report, one method per operation — and is reached through the generated package
// beside this one, over the transport this one exports.
//
// # Sending an event is idempotent, and retried
//
// SendEvent sends every event under an identifier this client knows: the one set on the Event, or a
// UUIDv7 it generates when the event carries none. Passing none does not mean the identifier comes
// from Hook0 — the value comes from here, travels with the request, and is what SendEvent answers.
//
// That is what makes retrying safe. Hook0 keys events on that identifier, so a request repeated
// after a network failure or a server error ingests the event once rather than twice; without a
// client-chosen identifier, a repeated request would create a second event and deliver it to every
// subscriber. It also gives the answer to a retry its meaning: EventAlreadyIngested in reply to a
// repeated request says an earlier attempt of that same send reached the API, so the send succeeded.
// The same answer to a first attempt is a genuine conflict and is reported as one.
//
// Only what could end differently is retried: a request that got no answer, a server error, and an
// instance saying it is being reached faster than it accepts. What the API refuses outright — a
// quota that is spent, a payload it will not read — is answered as is, since repeating it would only
// spend the same round trip again. The verdict for every problem the API can report is written down
// in the conformance corpus committed beside this module, which the suite here reads.
//
// A send is bounded on five axes, each of them the caller's to set: the size of the payload, which
// is refused before a socket is opened; how long one attempt is given; how many attempts are made;
// how long a single wait between them may be; and how long every wait of one send may add up to.
package hook0

import (
	"context"
	"crypto/rand"
	"encoding/json"
	"errors"
	"fmt"
	"math"
	// Named apart from crypto/rand, which is what an identifier is drawn from: jitter only has to
	// spread emitters out, and saying which of the two a line reaches for is the point of the name.
	weakrand "math/rand/v2"
	"net/http"
	"net/url"
	"regexp"
	"strconv"
	"strings"
	"time"
)

const (
	// DefaultMaxPayloadBytes is the largest event payload the client agrees to send.
	//
	// Hook0's API refuses request bodies above 2 MiB, so a payload above 1 MiB is already at risk of
	// being refused once the JSON envelope around it — metadata, labels, identifiers — is counted.
	// The client rules such an event out rather than spending a round trip, and every retry after
	// it, on a request that cannot be accepted.
	DefaultMaxPayloadBytes = 1024 * 1024

	// MaxAttemptsCap is the most attempts a retry policy can ever make, whatever MaxAttempts says.
	//
	// A policy is configuration, and configuration can be wrong; this cap keeps a mistyped
	// MaxAttempts from turning one send into an unbounded series of requests.
	MaxAttemptsCap = 16

	// AlreadyIngested is the identifier Hook0 gives the problem it answers when an event identifier
	// is already taken.
	AlreadyIngested = "EventAlreadyIngested"

	// RateLimited is the identifier Hook0 gives the problem it answers when requests are reaching
	// the instance faster than it accepts them.
	//
	// It shares its status with the quota problems, and is the only one of them worth repeating: a
	// quota clears when a plan changes or a day turns, neither of which happens inside the seconds a
	// send is given, while pacing clears on its own and the answer says when.
	RateLimited = "RateLimited"
)

const (
	// maxBackoffDoublings is where any backoff has long since reached its ceiling.
	maxBackoffDoublings = 30

	// conflictStatus is what Hook0 answers when the event identifier a request carries is taken.
	conflictStatus = 409

	// pacedStatus is what Hook0 answers both when a quota is spent and when requests are coming in
	// faster than the instance accepts them. Which of the two it is only the problem the body names
	// can say, which is why this status alone decides nothing.
	pacedStatus = 429

	// lowestServerError is the first status saying the failure is on Hook0's side, and so could
	// clear on its own.
	lowestServerError = 500

	// delayHeader is what the API names the delay before the request becomes servable in, in whole
	// seconds.
	delayHeader = "Retry-After"

	// eventPath is where an event is ingested, under the API URL.
	eventPath = "event"

	// eventTypesPath is where event types are read and created, under the API URL.
	eventTypesPath = "event_types"
)

// eventTypePattern is what an event type reads as.
var eventTypePattern = regexp.MustCompile(`^([A-Za-z0-9_]+)[.]([A-Za-z0-9_]+)[.]([A-Za-z0-9_]+)$`)

// RetryPolicy says how a client spaces out the attempts of a single send.
//
// The delay before a retry doubles from InitialBackoff and is capped by MaxBackoff; the delay
// actually waited is then drawn anywhere between zero and that ceiling, so that emitters which
// failed at the same moment do not come back at the same moment. Retrying stops as soon as the
// delays of the send would add up to more than MaxTotalDelay.
type RetryPolicy struct {
	// MaxAttempts is how many attempts a single send makes at most, the first one included. One
	// disables retrying, and nothing above MaxAttemptsCap is honoured.
	MaxAttempts int
	// InitialBackoff is the ceiling of the delay before the first retry.
	InitialBackoff time.Duration
	// MaxBackoff is the ceiling no single delay ever exceeds, however many retries were made.
	MaxBackoff time.Duration
	// MaxTotalDelay is the budget all the delays of one send share.
	MaxTotalDelay time.Duration
}

// DefaultRetryPolicy is four attempts spread over at most five seconds.
//
// Three retries absorb the blips a webhook emitter meets in production — a connection reset, a
// rolling deployment answering 503 — without holding the caller for long, and the five-second budget
// bounds what the worst send costs whatever the individual delays turn out to be.
func DefaultRetryPolicy() RetryPolicy {
	return RetryPolicy{
		MaxAttempts:    4,
		InitialBackoff: 100 * time.Millisecond,
		MaxBackoff:     2 * time.Second,
		MaxTotalDelay:  5 * time.Second,
	}
}

// DisabledRetryPolicy never retries: one attempt, and the caller hears what it answered.
func DisabledRetryPolicy() RetryPolicy {
	return RetryPolicy{MaxAttempts: 1}
}

// Attempts is how many attempts this policy actually makes: MaxAttempts, brought back inside one
// and MaxAttemptsCap.
func (p RetryPolicy) Attempts() int {
	if p.MaxAttempts < 1 {
		return 1
	}
	if p.MaxAttempts > MaxAttemptsCap {
		return MaxAttemptsCap
	}
	return p.MaxAttempts
}

// BackoffCeiling is the ceiling of the delay before retry number retry, where one is the first
// retry.
//
// It doubles from InitialBackoff and never exceeds MaxBackoff, so the ceilings of successive retries
// never decrease.
func (p RetryPolicy) BackoffCeiling(retry int) time.Duration {
	initial := max(p.InitialBackoff, 0)
	ceiling := max(p.MaxBackoff, 0)

	doublings := min(max(retry-1, 0), maxBackoffDoublings)
	delay := initial
	for range doublings {
		// Doubling stops at the ceiling rather than past it, which is also what keeps a long
		// schedule of a long backoff from running out of the room a duration has.
		if delay > ceiling/2 {
			delay = ceiling
			break
		}
		delay *= 2
	}

	return min(delay, ceiling)
}

// Delays is what this policy waits between the attempts of one send, one per retry, given one draw
// in [0, 1) per retry.
//
// Each delay lands between zero and the ceiling of its retry, and the schedule is cut short as soon
// as the next delay would spend more than MaxTotalDelay. There are therefore at most Attempts() - 1
// delays, and they add up to at most MaxTotalDelay.
//
// A draw that is missing or is not a finite number is read as one, which asks for the whole ceiling:
// an unusable source of randomness makes the client wait longer, never less.
func (p RetryPolicy) Delays(draws []float64) []time.Duration {
	budget := max(p.MaxTotalDelay, 0)
	retries := p.Attempts() - 1

	delays := make([]time.Duration, 0, retries)
	var spent time.Duration

	for retry := 1; retry <= retries; retry++ {
		delay := max(time.Duration(float64(p.BackoffCeiling(retry))*draw(draws, retry-1)), 0)
		// Written as what is left of the budget rather than as a sum, so nothing is added that
		// could carry two durations past what one can hold.
		if delay > budget-spent {
			break
		}
		spent += delay
		delays = append(delays, delay)
	}

	return delays
}

// draw is the draw for one retry, brought back inside [0, 1] whatever the randomness gave.
func draw(draws []float64, index int) float64 {
	if index >= len(draws) {
		return 1
	}
	drawn := draws[index]
	if math.IsNaN(drawn) || math.IsInf(drawn, 0) {
		return 1
	}
	return min(max(drawn, 0), 1)
}

// jitterDraws is the randomness used to jitter the delays of one send.
//
// Jitter only has to keep emitters that failed together from coming back together; it does not have
// to be unpredictable, so the platform's own generator is enough.
func jitterDraws(count int) []float64 {
	draws := make([]float64, 0, max(count, 0))
	for range max(count, 0) {
		draws = append(draws, weakrand.Float64())
	}
	return draws
}

// Options is every bound a client applies to one send.
type Options struct {
	// RetryPolicy is how the attempts of one send are spaced out.
	RetryPolicy RetryPolicy
	// RequestTimeout is how long one attempt is given.
	RequestTimeout time.Duration
	// MaxPayloadBytes is the largest payload sent, refused before a socket is opened.
	MaxPayloadBytes int
	// MaxResponseBytes is the largest answer read off a socket.
	MaxResponseBytes int64
}

// DefaultOptions is the bounds a client applies when the caller names none.
func DefaultOptions() Options {
	return Options{
		RetryPolicy:      DefaultRetryPolicy(),
		RequestTimeout:   DefaultRequestTimeout,
		MaxPayloadBytes:  DefaultMaxPayloadBytes,
		MaxResponseBytes: DefaultMaxResponseBytes,
	}
}

// Event is an event to send to Hook0.
//
// EventId is the caller's to set when it already has one to key the event on. Left empty, the client
// generates a UUIDv7, sends it and answers it — which is what lets it repeat a request without
// risking a second copy of the event being ingested and delivered to every subscriber.
type Event struct {
	// EventType is the type of the event, as the application declares it.
	EventType string
	// Payload is what the event carries.
	Payload string
	// PayloadContentType says how to read the payload.
	PayloadContentType string
	// Labels are what Hook0 routes the event by.
	Labels map[string]string
	// Metadata is anything else worth carrying, nil when there is none.
	Metadata map[string]string
	// OccurredAt is when the event happened; the zero moment means now.
	OccurredAt time.Time
	// EventId is what to key the event on, empty when the client is to choose.
	EventId string
}

// EventType is an event type, read out of the `service.resource_type.verb` it is written as.
type EventType struct {
	// Service is the leading segment.
	Service string
	// ResourceType is the middle segment.
	ResourceType string
	// Verb is the trailing segment.
	Verb string
}

// ParseEventType reads an event type, refusing one that does not name all three of its parts.
func ParseEventType(written string) (EventType, error) {
	read := eventTypePattern.FindStringSubmatch(written)
	if read == nil {
		return EventType{}, &EventTypeError{
			EventType: written,
			Detail:    "it does not read as service.resource_type.verb",
			Err:       ErrInvalidEventType,
		}
	}
	return EventType{Service: read[1], ResourceType: read[2], Verb: read[3]}, nil
}

// String writes an event type the way the API reads one.
func (e EventType) String() string {
	return fmt.Sprintf("%s.%s.%s", e.Service, e.ResourceType, e.Verb)
}

// GenerateEventId answers a UUIDv7, the shape of identifier Hook0 mints when it is the one choosing.
//
// Its leading 48 bits are the current time in milliseconds, so identifiers generated in sequence are
// ordered, which is what keeps the index they end up in from being written all over.
func GenerateEventId() string {
	drawn := make([]byte, 16)
	// crypto/rand.Read never answers a short read or an error on any platform Go supports.
	rand.Read(drawn)

	milliseconds := time.Now().UnixMilli()
	for index := range 6 {
		drawn[index] = byte(milliseconds >> (8 * (5 - index)))
	}
	drawn[6] = (drawn[6] & 0x0f) | 0x70
	drawn[8] = (drawn[8] & 0x3f) | 0x80

	return fmt.Sprintf(
		"%x-%x-%x-%x-%x",
		drawn[0:4], drawn[4:6], drawn[6:8], drawn[8:10], drawn[10:16],
	)
}

// Client is the Hook0 client, built once and shared wherever an application sends events.
type Client struct {
	apiURL        string
	applicationId string
	options       Options
	transport     *Transport
}

// NewClient builds a client reaching an instance of the API.
//
//   - apiURL: base API URL of a Hook0 instance, such as https://app.hook0.com/api/v1.
//   - applicationId: identifier of the Hook0 application events are sent to.
//   - token: an authentication token valid for that application.
//   - options: the bounds one send is held to.
func NewClient(apiURL string, applicationId string, token string, options Options) *Client {
	return &Client{
		apiURL:        apiURL,
		applicationId: applicationId,
		options:       options,
		transport: NewTransport(apiURL, token, options.RequestTimeout, options.MaxResponseBytes).
			underRetryPolicy(options.RetryPolicy),
	}
}

// APIURL is the base API URL this client reaches.
func (c *Client) APIURL() string {
	return c.apiURL
}

// ApplicationId is the application this client sends events to.
func (c *Client) ApplicationId() string {
	return c.applicationId
}

// Options is the bounds one send is held to.
func (c *Client) Options() Options {
	return c.options
}

// Transport is what this client issues its requests through, which is also what a generated
// operation group is built on.
func (c *Client) Transport() *Transport {
	return c.transport
}

// attempt is what one attempt at sending an event ended with.
type attempt struct {
	// ingested is the identifier the API ingested the event under, empty when it did not.
	ingested string
	// alreadyIngested says the API refused the event because it already holds one under the same
	// identifier.
	alreadyIngested bool
	// detail is what went wrong, in the words a caller is given.
	detail string
	// retryable says whether repeating this very request could end differently.
	retryable bool
	// err is the failure underneath, nil when the API answered and the answer is the whole story.
	// It travels to the caller so that errors.Is reaches the nature of what went wrong.
	err error
	// delayNamed says the answer named how long to wait before repeating the request.
	delayNamed bool
	// delay is how long it named, meaningful only when delayNamed says so.
	delay time.Duration
}

// SendEvent sends an event, and answers the identifier it was sent under.
func (c *Client) SendEvent(ctx context.Context, event Event) (string, error) {
	eventId := event.EventId
	if eventId == "" {
		eventId = GenerateEventId()
	}

	if size := len(event.Payload); size > c.options.MaxPayloadBytes {
		return "", &SendError{
			EventId: eventId,
			Detail: fmt.Sprintf(
				"the payload is %d bytes, which is more than the %d this client sends at most; nothing was sent",
				size, c.options.MaxPayloadBytes,
			),
			Err: ErrPayloadTooLarge,
		}
	}

	body := fullEvent(event, c.applicationId, eventId)
	policy := c.options.RetryPolicy
	delays := policy.Delays(jitterDraws(policy.Attempts() - 1))

	issued := 0
	var waited time.Duration
	for {
		issued++
		outcome := c.attemptSend(ctx, body)

		if outcome.ingested != "" {
			return outcome.ingested, nil
		}
		if outcome.alreadyIngested {
			// An earlier attempt of this very send reached the API, so the event is in and carries
			// the identifier answered here. The same answer to a first attempt is a genuine
			// conflict.
			if issued > 1 {
				return eventId, nil
			}
			return "", &SendError{EventId: eventId, Attempts: issued, Detail: outcome.detail}
		}

		retry := issued - 1
		if outcome.retryable && retry < len(delays) {
			wait := waitFor(outcome, delays[retry], policy.MaxTotalDelay-waited)
			timer := time.NewTimer(wait)
			select {
			case <-ctx.Done():
				timer.Stop()
				return "", &SendError{
					EventId:  eventId,
					Attempts: issued,
					Waited:   waited,
					Detail:   outcome.detail,
					Err:      ctx.Err(),
				}
			case <-timer.C:
			}
			waited += wait
			continue
		}

		return "", &SendError{
			EventId:  eventId,
			Attempts: issued,
			Waited:   waited,
			Detail:   outcome.detail,
			Err:      outcome.err,
		}
	}
}

// attemptSend is one attempt at sending an already-bounded event.
func (c *Client) attemptSend(ctx context.Context, body map[string]any) attempt {
	status, headers, payload, err := c.transport.Deliver(ctx, http.MethodPost, eventPath, url.Values{}, body)
	if err != nil {
		// Decided by the nature of the failure, not by the type carrying it: an answer above a
		// ceiling and a URL nothing can be sent to arrive as the same type as a reset connection,
		// and repeating either of them meets the very same thing.
		return attempt{detail: err.Error(), retryable: errors.Is(err, ErrUnreachable), err: err}
	}
	return readAttempt(status, headers, payload)
}

// readAttempt says what the API answered one attempt, and whether repeating it could end
// differently.
func readAttempt(status int, headers http.Header, payload []byte) attempt {
	body := string(payload)

	if status >= 200 && status < 300 {
		ingested := ingestedId(payload)
		if ingested == "" {
			// The API accepted the event but answered something this client cannot read; repeating
			// the request would meet the same answer.
			return attempt{detail: fmt.Sprintf("Hook0 answered %d without an event id", status)}
		}
		return attempt{ingested: ingested}
	}

	if status == conflictStatus && problemId(payload) == AlreadyIngested {
		return attempt{alreadyIngested: true, detail: body}
	}

	delay, delayNamed := namedDelay(headers)
	return attempt{
		detail:     body,
		retryable:  isRetryable(status, problemId(payload)),
		delayNamed: delayNamed,
		delay:      delay,
	}
}

// isRetryable says whether repeating a request the API answered that way could end differently.
//
// The status decides on its own everywhere but under the one it answers both a spent quota and a
// paced instance with: a quota clears when a plan changes or a day turns, and neither is something a
// send spending seconds can wait for. Only the problem the body names tells the two apart, and a
// body naming a problem this client has never heard of falls back to what the status says.
func isRetryable(status int, problem string) bool {
	if status == pacedStatus {
		return problem == RateLimited
	}
	return status >= lowestServerError
}

// waitFor is how long to wait before the next attempt: what the API asked for when it asked for
// anything, and the client's own schedule otherwise.
//
// Either way it is cut down to what is left of the budget every delay of one send shares, so a
// delay written by the other end cannot stretch a send past what the caller allowed for it.
func waitFor(outcome attempt, scheduled time.Duration, remaining time.Duration) time.Duration {
	wanted := scheduled
	if outcome.delayNamed {
		wanted = outcome.delay
	}
	return max(min(wanted, remaining), 0)
}

// namedDelay is the delay the API named before the request becomes servable, and whether it named
// one this client can read.
//
// Only a whole number of seconds is read. The header may also carry a date, which is a clock this
// client would be comparing against its own, and anything else is a header nobody meant: both leave
// the client's own schedule in place rather than being guessed at.
func namedDelay(headers http.Header) (time.Duration, bool) {
	written := strings.TrimSpace(headers.Get(delayHeader))
	if written == "" {
		return 0, false
	}

	seconds, err := strconv.ParseInt(written, 10, 32)
	if err != nil || seconds < 0 {
		return 0, false
	}
	return time.Duration(seconds) * time.Second, true
}

// ingestedId is the identifier the API says it ingested the event under.
func ingestedId(payload []byte) string {
	var answered struct {
		EventId string `json:"event_id"`
	}
	if err := json.Unmarshal(payload, &answered); err != nil {
		return ""
	}
	return answered.EventId
}

// problemId is the problem a refusal names, empty when the body names none this client can read.
func problemId(payload []byte) string {
	var problem struct {
		Id string `json:"id"`
	}
	if err := json.Unmarshal(payload, &problem); err != nil {
		return ""
	}
	return problem.Id
}

// fullEvent is an event as the API reads one.
func fullEvent(event Event, applicationId string, eventId string) map[string]any {
	occurredAt := event.OccurredAt
	if occurredAt.IsZero() {
		occurredAt = time.Now().UTC()
	}

	labels := event.Labels
	if labels == nil {
		labels = map[string]string{}
	}

	body := map[string]any{
		"event_id":             eventId,
		"application_id":       applicationId,
		"event_type":           event.EventType,
		"payload":              event.Payload,
		"payload_content_type": event.PayloadContentType,
		"occurred_at":          occurredAt.Format(time.RFC3339Nano),
		"labels":               labels,
	}
	if event.Metadata != nil {
		body["metadata"] = event.Metadata
	}
	return body
}

// UpsertEventTypes creates the event types the application does not declare yet, and answers those.
func (c *Client) UpsertEventTypes(ctx context.Context, eventTypes []string) ([]string, error) {
	wanted := make([]EventType, 0, len(eventTypes))
	for _, written := range eventTypes {
		eventType, err := ParseEventType(written)
		if err != nil {
			return nil, err
		}
		wanted = append(wanted, eventType)
	}
	if len(wanted) == 0 {
		return []string{}, nil
	}

	declared, err := c.declaredEventTypes(ctx)
	if err != nil {
		return nil, err
	}

	created := make([]string, 0, len(wanted))
	for _, eventType := range wanted {
		if _, found := declared[eventType.String()]; found {
			continue
		}
		if err := c.createEventType(ctx, eventType); err != nil {
			return nil, err
		}
		created = append(created, eventType.String())
	}
	return created, nil
}

// declaredEventTypes is what the application already declares.
func (c *Client) declaredEventTypes(ctx context.Context) (map[string]struct{}, error) {
	query := url.Values{}
	query.Set("application_id", c.applicationId)

	status, payload, err := c.transport.Request(ctx, http.MethodGet, eventTypesPath, query, nil)
	if err != nil {
		return nil, &EventTypeError{Detail: err.Error(), Err: err}
	}
	if status < 200 || status >= 300 {
		return nil, &EventTypeError{Detail: fmt.Sprintf("the API answered %d: %s", status, payload)}
	}

	var answered []struct {
		EventTypeName string `json:"event_type_name"`
	}
	if err := json.Unmarshal(payload, &answered); err != nil {
		return nil, &EventTypeError{Detail: "the API did not answer a list of event types", Err: err}
	}

	declared := make(map[string]struct{}, len(answered))
	for _, entry := range answered {
		declared[entry.EventTypeName] = struct{}{}
	}
	return declared, nil
}

// createEventType declares one event type on the application.
func (c *Client) createEventType(ctx context.Context, eventType EventType) error {
	body := map[string]any{
		"application_id": c.applicationId,
		"service":        eventType.Service,
		"resource_type":  eventType.ResourceType,
		"verb":           eventType.Verb,
	}

	status, payload, err := c.transport.Request(ctx, http.MethodPost, eventTypesPath, url.Values{}, body)
	if err != nil {
		return &EventTypeError{EventType: eventType.String(), Detail: err.Error(), Err: err}
	}
	if status < 200 || status >= 300 {
		return &EventTypeError{
			EventType: eventType.String(),
			Detail:    fmt.Sprintf("the API answered %d: %s", status, payload),
		}
	}
	return nil
}
