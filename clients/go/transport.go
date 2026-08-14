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
)

// jsonMediaType is what a request body says it carries, and what an answer is asked for in.
const jsonMediaType = "application/json"

// TransportError is a request that got no answer: a connection refused or reset, an attempt out of
// time, a body that stopped mid-way.
//
// None of these says whether the API acted on the request, which is exactly why a send carries an
// identifier the client chose itself. It is also the one failure a send retries: anything else is
// either what the API answered or a request that could never be built.
type TransportError struct {
	// Detail says what went wrong, in the words a caller is given.
	Detail string
	// Err is what the standard library reported, nil when there is nothing to name.
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

	return &Transport{
		baseURL:          baseURL,
		token:            token,
		client:           &http.Client{Timeout: timeout},
		maxResponseBytes: maxResponseBytes,
	}
}

// Request issues one request and answers the status, the body, and why it got neither.
//
// A refusal is an answer: the status and the body are what say whether repeating the request could
// end differently, so they are answered rather than raised over. Only a request that got no answer
// at all is an error here.
func (t *Transport) Request(
	ctx context.Context,
	method string,
	path string,
	query url.Values,
	body any,
) (int, []byte, error) {
	target, err := t.resolve(path, query)
	if err != nil {
		return 0, nil, err
	}

	var encoded io.Reader
	if body != nil {
		written, err := json.Marshal(body)
		if err != nil {
			// A body that cannot be written is not a request that could be repeated.
			return 0, nil, fmt.Errorf("the request body cannot be written as JSON: %w", err)
		}
		encoded = bytes.NewReader(written)
	}

	request, err := http.NewRequestWithContext(ctx, method, target, encoded)
	if err != nil {
		return 0, nil, fmt.Errorf("the request cannot be built: %w", err)
	}
	request.Header.Set("Authorization", "Bearer "+t.token)
	request.Header.Set("Accept", jsonMediaType)
	if body != nil {
		request.Header.Set("Content-Type", jsonMediaType)
	}

	answer, err := t.client.Do(request)
	if err != nil {
		return 0, nil, &TransportError{Detail: err.Error(), Err: err}
	}
	defer answer.Body.Close()

	payload, err := io.ReadAll(io.LimitReader(answer.Body, t.maxResponseBytes+1))
	if err != nil {
		return 0, nil, &TransportError{Detail: err.Error(), Err: err}
	}
	if int64(len(payload)) > t.maxResponseBytes {
		return 0, nil, &TransportError{
			Detail: fmt.Sprintf("the API answered more than the %d bytes read at most", t.maxResponseBytes),
		}
	}

	return answer.StatusCode, payload, nil
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
