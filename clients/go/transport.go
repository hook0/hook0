// How a request reaches the API, and what a server on the other end is not allowed to cost.
//
// The transport answers the status and the bytes and knows nothing of what the API declares:
// reading those bytes is the generated half's job, and deciding whether to send them again is the
// client's. That is what lets one HTTP implementation serve both the hand-written event path and
// every generated method — the generated package declares the shape it needs, and this type answers
// to it without either half importing the other.
//
// Nothing here reaches for a third-party HTTP library. Everything a server controls is bounded: how
// long one exchange may take, and how many bytes of body are read off the socket.

package hook0

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strings"
	"time"
)

const (
	// DefaultRequestTimeout is the longest one attempt at reaching the API is given before it is
	// abandoned.
	//
	// Ten seconds is far above what ingesting an event takes when the API is healthy, and short
	// enough that a stuck connection does not hold a caller for a noticeable time.
	DefaultRequestTimeout = 10 * time.Second

	// DefaultMaxResponseBytes is the largest response body read off a socket.
	DefaultMaxResponseBytes int64 = 8 * 1024 * 1024

	// MaxResponseHeaders is the most headers read out of one answer, and MaxHeaderBytes the longest
	// one of them may be.
	//
	// The head of an answer is written by the other end, so it is bounded like the body: a server
	// that is broken or hostile can otherwise spend a caller's memory on headers alone. Both are
	// the numbers the conformance corpus names, so that no two SDKs bound different things.
	MaxResponseHeaders = 64
	MaxHeaderBytes     = 64 * 1024
)

// jsonMediaType is what a request body says it carries, and what an answer is asked for in.
const jsonMediaType = "application/json"

// TransportError is a request that produced no answer to read.
//
// Several natures of failure land here — a connection that was refused or reset, an answer above a
// ceiling this client set for itself, a URL nothing can be sent to — and only the first of them
// could end differently. What a send retries is therefore decided by errors.Is against
// ErrUnreachable, ErrAnswerAboveABound and ErrUnusableAPIURL, never by this type: a client deciding
// by the type spends four attempts on a mistyped API URL and then hands its caller a message that
// accuses the network.
type TransportError struct {
	// Detail says what went wrong, in the words a caller is given.
	Detail string
	// Err is the nature of the failure, and under it whatever the standard library reported.
	Err error
}

// Error says why the API was not reached.
func (e *TransportError) Error() string {
	return e.Detail
}

// Unwrap answers what the standard library reported.
func (e *TransportError) Unwrap() error {
	return e.Err
}

// Transport issues one request and reads the answer.
//
// It answers the shape the generated package declares, so a generated operation group is built on
// one of these directly.
type Transport struct {
	baseURL          string
	token            string
	client           *http.Client
	maxResponseBytes int64
}

// NewTransport builds a transport reaching an instance of the API with a token valid for it.
//
// A timeout or a ceiling that names nothing is the default rather than no bound at all: a transport
// with no timeout is one a single hung connection holds forever.
func NewTransport(baseURL string, token string, timeout time.Duration, maxResponseBytes int64) *Transport {
	if timeout <= 0 {
		timeout = DefaultRequestTimeout
	}
	if maxResponseBytes <= 0 {
		maxResponseBytes = DefaultMaxResponseBytes
	}

	// The head is bounded where it is read off the socket rather than only after it arrived: a
	// server writing headers forever would otherwise be buffered whole before anything counted them.
	carrier := &http.Transport{}
	if standard, is := http.DefaultTransport.(*http.Transport); is {
		carrier = standard.Clone()
	}
	carrier.MaxResponseHeaderBytes = int64(MaxResponseHeaders) * int64(MaxHeaderBytes)

	return &Transport{
		baseURL:          baseURL,
		token:            token,
		client:           &http.Client{Timeout: timeout, Transport: carrier},
		maxResponseBytes: maxResponseBytes,
	}
}

// Request issues one request and answers the status, the body, and why it got neither.
//
// A refusal is an answer: the status and the body are what say whether repeating the request could
// end differently, so they are answered rather than raised over. Only a request that got no answer
// at all is an error here.
//
// This is the shape the generated package declares, which reads what the API sent and nothing about
// how it was sent. A caller that also needs the headers — the delay a paced instance names beside a
// refusal is one — asks Deliver for them.
func (t *Transport) Request(
	ctx context.Context,
	method string,
	path string,
	query url.Values,
	body any,
) (int, []byte, error) {
	status, _, payload, err := t.Deliver(ctx, method, path, query, body)
	return status, payload, err
}

// Deliver issues one request and answers the status, the headers, and the body.
//
// It is Request with what the answer carried beside its body, which is what a client reads when the
// API names how long to wait before the request becomes servable again.
func (t *Transport) Deliver(
	ctx context.Context,
	method string,
	path string,
	query url.Values,
	body any,
) (int, http.Header, []byte, error) {
	target, err := t.resolve(path, query)
	if err != nil {
		return 0, nil, nil, &TransportError{
			Detail: err.Error(),
			Err:    fmt.Errorf("%w: %w", ErrUnusableAPIURL, err),
		}
	}

	var encoded io.Reader
	if body != nil {
		written, err := json.Marshal(body)
		if err != nil {
			// A body that cannot be written is not a request that could be repeated.
			return 0, nil, nil, fmt.Errorf("the request body cannot be written as JSON: %w", err)
		}
		encoded = bytes.NewReader(written)
	}

	request, err := http.NewRequestWithContext(ctx, method, target, encoded)
	if err != nil {
		return 0, nil, nil, fmt.Errorf("the request cannot be built: %w", err)
	}
	request.Header.Set("Authorization", "Bearer "+t.token)
	request.Header.Set("Accept", jsonMediaType)
	if body != nil {
		request.Header.Set("Content-Type", jsonMediaType)
	}

	answer, err := t.client.Do(request)
	if err != nil {
		return 0, nil, nil, &TransportError{
			Detail: err.Error(),
			Err:    fmt.Errorf("%w: %w", ErrUnreachable, err),
		}
	}
	defer answer.Body.Close()

	if err := bounded(answer.Header); err != nil {
		return 0, nil, nil, err
	}

	payload, err := io.ReadAll(io.LimitReader(answer.Body, t.maxResponseBytes+1))
	if err != nil {
		// A body that stopped mid-way is a request that got no answer worth reading, and the next
		// one could carry the whole of it.
		return 0, nil, nil, &TransportError{
			Detail: err.Error(),
			Err:    fmt.Errorf("%w: %w", ErrUnreachable, err),
		}
	}
	if int64(len(payload)) > t.maxResponseBytes {
		return 0, nil, nil, &TransportError{
			Detail: fmt.Sprintf("the API answered more than the %d bytes read at most", t.maxResponseBytes),
			Err:    ErrAnswerAboveABound,
		}
	}

	return answer.StatusCode, answer.Header, payload, nil
}

// bounded refuses a head above what this client agrees to hold, naming the ceiling it crossed.
func bounded(headers http.Header) error {
	if len(headers) > MaxResponseHeaders {
		return &TransportError{
			Detail: fmt.Sprintf(
				"the API answered %d headers, above the %d read at most", len(headers), MaxResponseHeaders,
			),
			Err: ErrAnswerAboveABound,
		}
	}

	for name, values := range headers {
		for _, value := range values {
			if len(name)+len(value) > MaxHeaderBytes {
				return &TransportError{
					Detail: fmt.Sprintf(
						"the API answered a `%s` header above the %d bytes read at most", name, MaxHeaderBytes,
					),
					Err: ErrAnswerAboveABound,
				}
			}
		}
	}

	return nil
}

// resolve says where a request lands: a path of its own replaces the base's, a relative one extends
// it, and the query the caller assembled is added to whatever the base already carried.
func (t *Transport) resolve(path string, query url.Values) (string, error) {
	base, err := url.Parse(t.baseURL)
	if err != nil {
		return "", fmt.Errorf("`%s` is not an API URL: %w", t.baseURL, err)
	}
	if !strings.HasSuffix(base.Path, "/") {
		base.Path += "/"
	}

	reference, err := url.Parse(path)
	if err != nil {
		return "", fmt.Errorf("`%s` is not a path: %w", path, err)
	}

	resolved := base.ResolveReference(reference)
	if len(query) > 0 {
		carried := resolved.Query()
		for name, values := range query {
			for _, value := range values {
				carried.Add(name, value)
			}
		}
		resolved.RawQuery = carried.Encode()
	}

	return resolved.String(), nil
}
