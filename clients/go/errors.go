package hook0

import (
	"errors"
	"fmt"
	"time"
)

// The reasons this client refuses to do what it was asked, as values errors.Is compares against.
//
// A caller that only wants to know whether to try again reads the sentinel; a caller that wants the
// numbers reads the error the sentinel is wrapped in.
var (
	// ErrPayloadTooLarge is an event whose payload is larger than the client agrees to send. It is
	// answered before a socket is opened, so nothing was sent when a caller sees it.
	ErrPayloadTooLarge = errors.New("the event payload is larger than this client sends")

	// ErrInvalidEventType is an event type that does not read as `service.resource_type.verb`.
	ErrInvalidEventType = errors.New("the event type does not have a valid syntax")

	// ErrUnreachable is a request that got no answer: a connection refused or reset, an attempt out
	// of time, a body that stopped mid-way.
	//
	// It is the one failure of a send that could end differently, which is why it is told apart
	// from the others rather than grouped with them under the type that carries them all. None of
	// these says whether the API acted on the request, which is exactly why a send carries an
	// identifier the client chose itself.
	ErrUnreachable = errors.New("the API could not be reached")

	// ErrAnswerAboveABound is an answer that crossed a ceiling this client set for itself: a body,
	// a header, or a number of headers above what it agrees to read.
	//
	// Repeating the request draws the same answer, so it is reported rather than retried: a client
	// that retries it reads the oversized answer four times and then blames the network.
	ErrAnswerAboveABound = errors.New("the API answered more than this client reads")

	// ErrUnusableAPIURL is an API URL no request can be sent to. Nothing was sent when a caller
	// sees it, and building the same request again would fail the same way.
	ErrUnusableAPIURL = errors.New("the API URL is not one a request can be sent to")

	// ErrSignatureUnreadable is a signature header this client cannot read whole: a part it needs
	// that is missing, a moment that is not a number of seconds, a code that is not hexadecimal.
	ErrSignatureUnreadable = errors.New("the signature header cannot be read")

	// ErrHeaderNotDelivered is a header the signature says it covers that the request did not
	// carry. Signing over an absent value would let a sender drop a header and keep the signature
	// valid, so this is refused before any code is computed.
	ErrHeaderNotDelivered = errors.New("a header the signature covers was not delivered")

	// ErrSignatureMismatch is a code that is not the one the subscription secret produces.
	ErrSignatureMismatch = errors.New("the signature does not match what the subscription secret produces")

	// ErrSignatureOutsideTolerance is a moment sitting further from now than the caller accepts, in
	// either direction: a delivery captured and replayed later, and one dated in the future by a
	// clock that is ahead or by a sender widening its own acceptance window, are refused alike.
	ErrSignatureOutsideTolerance = errors.New("the signature's moment sits outside the tolerance accepted")
)

// SendError is what a send that did not ingest an event answers with.
//
// It says how many attempts were made and how long they spent waiting, which is the difference
// between a transient outage this client rode out and a request the API will never accept. What
// went wrong underneath is under Unwrap, so errors.Is finds it.
type SendError struct {
	// EventId is the identifier the request carried, whether the caller chose it or this client
	// generated it.
	EventId string
	// Attempts is how many requests were issued, the first one included. Zero when the send was
	// refused before any socket was opened.
	Attempts int
	// Waited is how much of the delay budget the retries spent.
	Waited time.Duration
	// Detail is what the last attempt ran into, in the words a caller is given.
	Detail string
	// Err is the reason underneath, nil when the failure is only what the API answered.
	Err error
}

// Error says what went wrong, and what it cost.
func (e *SendError) Error() string {
	if e.Attempts <= 1 {
		return fmt.Sprintf("sending event %s failed: %s", e.EventId, e.Detail)
	}
	return fmt.Sprintf(
		"sending event %s failed: gave up after %d attempts spread over %s of retry delay; last failure: %s",
		e.EventId, e.Attempts, e.Waited, e.Detail,
	)
}

// Unwrap answers the reason underneath, which is what lets errors.Is name it.
func (e *SendError) Unwrap() error {
	return e.Err
}

// EventTypeError is an event type this client would not use or could not create.
type EventTypeError struct {
	// EventType is the one that was asked for.
	EventType string
	// Detail is what went wrong, in the words a caller is given.
	Detail string
	// Err is the reason underneath, nil when there is none to name.
	Err error
}

// Error says which event type failed, and why.
func (e *EventTypeError) Error() string {
	return fmt.Sprintf("event type %q failed: %s", e.EventType, e.Detail)
}

// Unwrap answers the reason underneath, which is what lets errors.Is name it.
func (e *EventTypeError) Unwrap() error {
	return e.Err
}
