// A Hook0 API on a loopback port, and the events the cases beside this one send to it.
//
// Every case goes over a real socket: the request the client builds, the headers it sets, the way it
// reads an answer and the way it gives up on one are all the real ones. Nothing here stands in for a
// part of the client, so a case that passes says the client works rather than that it was called.

package hook0_test

import (
	"context"
	"encoding/json"
	"net"
	"net/http"
	"net/http/httptest"
	"sync"
	"testing"
	"time"

	hook0 "github.com/hook0/hook0-go/v2"
)

const (
	// maxRequestBodyBytes bounds what one connection buffers. No case makes a request anywhere near
	// this large.
	maxRequestBodyBytes = 64 * 1024

	// caseTimeout is what every case is given. They all talk to a loopback socket, so one taking
	// this long is one that hung rather than one that was slow.
	caseTimeout = 20 * time.Second

	applicationId  = "3f2504e0-4f89-41d3-9a0c-0305e82c3301"
	organizationId = "3f2504e0-4f89-41d3-9a0c-0305e82c3302"
	token          = "token-xyz"
)

// scriptedResponse is what the API answers to one request, in the order the case scripted it.
type scriptedResponse struct {
	// status is what the answer carries, when there is one.
	status int
	// body is the document the answer carries.
	body any
	// headers are what the answer carries beside its body, such as the delay a paced instance names.
	headers map[string]string
	// heldFor is how long the API sits on the answer before writing anything.
	heldFor time.Duration
	// hangsUp says the API closes the connection without answering at all, which is the transport
	// failure a send is supposed to ride out.
	hangsUp bool
	// stopsMidBody says the API announces a body and closes before it has written the whole of it,
	// which is the answer a client gets nothing worth reading out of.
	stopsMidBody bool
}

// receivedRequest is a request the API received, in the order it received it.
type receivedRequest struct {
	method  string
	target  string
	headers http.Header
	body    string
}

// json reads the document a request carried.
func (r receivedRequest) json(t *testing.T) map[string]any {
	t.Helper()

	var document map[string]any
	if err := json.Unmarshal([]byte(r.body), &document); err != nil {
		t.Fatalf("request body %q is not a JSON object: %v", r.body, err)
	}
	return document
}

// fakeAPI is a Hook0 API listening on a loopback port for the lifetime of one case.
type fakeAPI struct {
	server   *httptest.Server
	mutex    sync.Mutex
	scripted []scriptedResponse
	answered int
	received []receivedRequest
}

// listen starts an API and stops it when the case is over.
func listen(t *testing.T) *fakeAPI {
	t.Helper()

	api := &fakeAPI{}
	api.server = httptest.NewServer(http.HandlerFunc(api.serve))
	t.Cleanup(api.server.Close)
	return api
}

// willAnswer queues the answers the case expects the client to draw, in order.
func (a *fakeAPI) willAnswer(responses ...scriptedResponse) {
	a.mutex.Lock()
	defer a.mutex.Unlock()
	a.scripted = append(a.scripted, responses...)
}

// baseURL is where the client reaches this API.
func (a *fakeAPI) baseURL() string {
	return a.server.URL
}

// requests is what the API received, in the order it received it.
func (a *fakeAPI) requests() []receivedRequest {
	a.mutex.Lock()
	defer a.mutex.Unlock()
	return append([]receivedRequest(nil), a.received...)
}

// requestCount is how many requests reached the API.
func (a *fakeAPI) requestCount() int {
	a.mutex.Lock()
	defer a.mutex.Unlock()
	return len(a.received)
}

// eventIdOf is the event identifier request number index carried, as the API read it.
func (a *fakeAPI) eventIdOf(t *testing.T, index int) string {
	t.Helper()

	requests := a.requests()
	if index >= len(requests) {
		t.Fatalf("expected at least %d requests, got %d", index+1, len(requests))
	}

	carried, ok := requests[index].json(t)["event_id"].(string)
	if !ok {
		t.Fatalf("request %d carried no event id", index)
	}
	return carried
}

func (a *fakeAPI) next() scriptedResponse {
	a.mutex.Lock()
	defer a.mutex.Unlock()

	if a.answered >= len(a.scripted) {
		return scriptedResponse{
			status: http.StatusInternalServerError,
			body:   map[string]any{"error": "the case scripted no answer for this request"},
		}
	}
	scripted := a.scripted[a.answered]
	a.answered++
	return scripted
}

func (a *fakeAPI) serve(writer http.ResponseWriter, request *http.Request) {
	body := make([]byte, 0)
	if request.Body != nil {
		read := http.MaxBytesReader(writer, request.Body, maxRequestBodyBytes)
		buffered := make([]byte, maxRequestBodyBytes)
		total := 0
		for total < len(buffered) {
			count, err := read.Read(buffered[total:])
			total += count
			if err != nil {
				break
			}
		}
		body = buffered[:total]
	}

	a.mutex.Lock()
	a.received = append(a.received, receivedRequest{
		method:  request.Method,
		target:  request.URL.RequestURI(),
		headers: request.Header.Clone(),
		body:    string(body),
	})
	a.mutex.Unlock()

	scripted := a.next()

	if scripted.hangsUp {
		// Hanging up without a word is what a connection reset looks like to the client, which is
		// the one failure a send may ride out.
		hijacker, ok := writer.(http.Hijacker)
		if !ok {
			panic("the test server cannot hang up on a request")
		}
		connection, _, err := hijacker.Hijack()
		if err != nil {
			panic(err)
		}
		_ = connection.(net.Conn).Close()
		return
	}

	if scripted.stopsMidBody {
		hijacker, ok := writer.(http.Hijacker)
		if !ok {
			panic("the test server cannot stop mid-answer")
		}
		connection, _, err := hijacker.Hijack()
		if err != nil {
			panic(err)
		}
		// A head announcing more than what follows it, and then a close: the client is left with a
		// body that stopped where the connection did.
		_, _ = connection.Write([]byte("HTTP/1.1 200 OK\r\nContent-Length: 4096\r\n\r\n{}"))
		_ = connection.Close()
		return
	}

	if scripted.heldFor > 0 {
		time.Sleep(scripted.heldFor)
	}

	answer, err := json.Marshal(scripted.body)
	if err != nil {
		panic(err)
	}
	writer.Header().Set("Content-Type", "application/json")
	for name, value := range scripted.headers {
		writer.Header().Set(name, value)
	}
	writer.WriteHeader(scripted.status)
	_, _ = writer.Write(answer)
}

// ingested is what the API answers when it took the event.
func ingested(eventId string) scriptedResponse {
	return scriptedResponse{
		status: http.StatusCreated,
		body: map[string]any{
			"application_id": applicationId,
			"event_id":       eventId,
			"received_at":    "2026-01-01T00:00:00Z",
		},
	}
}

// alreadyIngested is what the API answers when the identifier a request carries is taken.
func alreadyIngested() scriptedResponse {
	return scriptedResponse{
		status: http.StatusConflict,
		body: map[string]any{
			"id":     hook0.AlreadyIngested,
			"title":  "Event already Ingested",
			"detail": "This event was previously ingested and recorded inside Hook0 service.",
			"status": http.StatusConflict,
			"type":   "https://documentation.hook0.com/problems",
		},
	}
}

// serverError is a failure on the API's side, which could clear on its own.
func serverError() scriptedResponse {
	return scriptedResponse{
		status: http.StatusInternalServerError,
		body:   map[string]any{"id": "InternalServerError", "status": http.StatusInternalServerError},
	}
}

// refused is a failure repeating the request would meet again.
func refused() scriptedResponse {
	return scriptedResponse{
		status: http.StatusBadRequest,
		body:   map[string]any{"id": "Validation", "status": http.StatusBadRequest},
	}
}

// hangsUp is a request that gets no answer at all.
func hangsUp() scriptedResponse {
	return scriptedResponse{hangsUp: true}
}

// anEvent is the event the send cases carry.
func anEvent() hook0.Event {
	return hook0.Event{
		EventType:          "auth.user.create",
		Payload:            `{"email": "test@example.com"}`,
		PayloadContentType: "application/json",
		Labels:             map[string]string{"environment": "production"},
	}
}

// promptRetries is a schedule short enough that a case spends its time on requests rather than on
// waiting. Its budget sits far above what its delays add up to, so the number of attempts a case
// observes is the one its policy asked for rather than the one its budget allowed.
func promptRetries(maxAttempts int) hook0.RetryPolicy {
	return hook0.RetryPolicy{
		MaxAttempts:    maxAttempts,
		InitialBackoff: 5 * time.Millisecond,
		MaxBackoff:     5 * time.Millisecond,
		MaxTotalDelay:  time.Second,
	}
}

// promptOptions is the bounds a case holds a client to.
func promptOptions(maxAttempts int) hook0.Options {
	options := hook0.DefaultOptions()
	options.RetryPolicy = promptRetries(maxAttempts)
	options.RequestTimeout = 5 * time.Second
	return options
}

// bounded is a context no case may outlive.
func bounded(t *testing.T) context.Context {
	t.Helper()

	ctx, cancel := context.WithTimeout(context.Background(), caseTimeout)
	t.Cleanup(cancel)
	return ctx
}
