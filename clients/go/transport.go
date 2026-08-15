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
	"runtime"
	"runtime/debug"
	"strings"
	"sync"
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

	// MaxHeadBytes is the largest whole head an answer may carry, every line counted together.
	//
	// This is the one that bounds what a head costs, because it bounds the total: a line count and
	// a size per line multiply, and the two above admit sixty-four lines of sixty-four kilobytes
	// between them. They earn their place by refusing early, on the line that crosses them rather
	// than at the end of the head; this one sets the ceiling.
	//
	// Sixteen kilobytes is the ceiling of the strictest runtime any target runs on, which is what
	// makes it a number every target can apply in library code. It is applied here rather than left
	// to MaxResponseHeaderBytes below: that one is an outer wall, set far above this so that what
	// refuses an abusive head is this client's own ceiling rather than whatever the runtime of the
	// day happens to allow.
	MaxHeadBytes = 16 * 1024
)

// jsonMediaType is what a request body says it carries, and what an answer is asked for in.
const jsonMediaType = "application/json"

// maxUserAgentPartChars is the longest each part this client composes its User-Agent out of may be,
// in characters.
//
// The runtime and the operating system are described by the platform rather than by this module, so
// their length is not this module's to guarantee: they are cut here so that the header cannot grow
// with whatever the platform feels like saying. Every part is also stripped of anything the grammar
// of the header uses as punctuation, so a platform cannot forge a shape it does not have.
const maxUserAgentPartChars = 64

// modulePath is what this module is imported under, which is the name the build records it by.
const modulePath = "github.com/hook0/hook0/clients/go"

// unknownVersion is what the version reads as when the build recorded none for this module, which
// is what a binary built without module information leaves behind.
const unknownVersion = "unknown"

// userAgent says which SDK, at which version, on which runtime and operating system, is talking to
// the API.
//
// Nothing in the module declares a version: a Go module is versioned by the tag that publishes it,
// so the number is read back out of what the build recorded rather than written down here, where it
// would disagree with that tag the first time either moved. Worked out once, since nothing it is
// built out of changes while the process runs.
var userAgent = sync.OnceValue(func() string {
	return fmt.Sprintf(
		"hook0-client-go/%s (%s; %s)",
		clipped(moduleVersion()),
		clipped(runtime.Version()),
		clipped(runtime.GOOS+" "+runtime.GOARCH),
	)
})

// moduleVersion is what the build recorded this module as: the version something else required it
// at, the version it was itself built as when it is the main module, and nothing nameable when the
// build recorded neither.
func moduleVersion() string {
	info, read := debug.ReadBuildInfo()
	if !read {
		return unknownVersion
	}
	for _, dependency := range info.Deps {
		if dependency.Path == modulePath && dependency.Version != "" {
			return dependency.Version
		}
	}
	if info.Main.Path == modulePath && info.Main.Version != "" {
		return info.Main.Version
	}
	return unknownVersion
}

// clipped is one part of the User-Agent, with everything the header's own grammar uses taken out of
// it and cut to maxUserAgentPartChars.
func clipped(part string) string {
	var kept strings.Builder
	for _, character := range part {
		if character < ' ' || character > '~' || character == '(' || character == ')' || character == ';' {
			continue
		}
		if kept.Len() == maxUserAgentPartChars {
			break
		}
		kept.WriteRune(character)
	}
	return kept.String()
}

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
	// Set rather than left alone: what the standard library names itself here says which Go built
	// the caller and nothing at all about which SDK is talking.
	request.Header.Set("User-Agent", userAgent())
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

	whole := 0
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
			whole += len(name) + len(value)
		}
	}

	// The total, once every line has been counted. A head this far along is one the runtime has
	// already read whole, so counting it to its end costs nothing more than counting part of it,
	// and the refusal can say how heavy the head actually was.
	if whole > MaxHeadBytes {
		return &TransportError{
			Detail: fmt.Sprintf(
				"the API answered a head of %d bytes, above the %d read at most", whole, MaxHeadBytes,
			),
			Err: ErrAnswerAboveABound,
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
