// The cases the shared conformance corpus dictates, run against this client.
//
// The corpus sits at clients/conformance, is hand-authored, and is read by the suite of every SDK.
// Nothing below writes down a verdict, a bound or a signature of its own: they are read out of the
// committed documents and this client is driven against them over a real socket. A case added to
// the corpus is therefore exercised here without this file being touched, and a verdict changed
// there fails here until this client agrees with it again.

package hook0_test

import (
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	hook0 "github.com/hook0/hook0/clients/go"
)

const (
	// corpusDirectory is where the shared contract sits, from the directory this package tests in.
	corpusDirectory = "../conformance"

	// maxCorpusBytes is the largest document of the corpus read back. The corpus is committed, so
	// one above this is one that grew out of shape rather than one somebody meant.
	maxCorpusBytes = 512 * 1024

	// promptBackoff is the schedule a case that is not about waiting spends between attempts.
	promptBackoff = 5 * time.Millisecond

	// delayBudget is the budget the delay cases share. A delay the API names above it is expected to
	// be cut down to it, so it also bounds what those cases cost.
	delayBudget = 1100 * time.Millisecond

	// delaySlack is what a wait may overshoot by before it is read as more than what was asked for:
	// a loopback round trip, a timer and a scheduler all sit inside it.
	delaySlack = 400 * time.Millisecond
)

// retryContract is what the corpus says a client does with each way one attempt can end.
type retryContract struct {
	Transport struct {
		Causes []struct {
			Cause     string `json:"cause"`
			Retryable bool   `json:"retryable"`
			Reason    string `json:"reason"`
		} `json:"causes"`
	} `json:"transport"`
	Statuses []struct {
		Status    int    `json:"status"`
		Retryable bool   `json:"retryable"`
		Reason    string `json:"reason"`
	} `json:"statuses"`
	Problems []struct {
		Problem   string `json:"problem"`
		Status    int    `json:"status"`
		Retryable bool   `json:"retryable"`
		Reason    string `json:"reason"`
	} `json:"problems"`
	RetryAfter struct {
		Header string `json:"header"`
		Cases  []struct {
			Name     string `json:"name"`
			Header   string `json:"header"`
			Honoured bool   `json:"honoured"`
			Seconds  int    `json:"seconds"`
		} `json:"cases"`
	} `json:"retry_after"`
}

// boundsContract is what the corpus says one send is held to, and what the other end may cost it.
type boundsContract struct {
	Bounds struct {
		MaxAttempts        int   `json:"max_attempts"`
		MaxAttemptsCap     int   `json:"max_attempts_cap"`
		InitialBackoffs    int64 `json:"initial_backoff_ms"`
		MaxBackoffMs       int64 `json:"max_backoff_ms"`
		MaxTotalDelayMs    int64 `json:"max_total_delay_ms"`
		RequestTimeout     int64 `json:"request_timeout_ms"`
		MaxPayloadBytes    int   `json:"max_payload_bytes"`
		MaxResponseBytes   int64 `json:"max_response_bytes"`
		MaxResponseHeaders int   `json:"max_response_headers"`
		MaxHeaderBytes     int   `json:"max_header_bytes"`
	} `json:"bounds"`
}

// requestContract is what the corpus says every request carries beside its body.
type requestContract struct {
	Occasions []string `json:"occasions"`
	Headers   []struct {
		Name   string `json:"name"`
		Value  string `json:"value"`
		When   string `json:"when"`
		Reason string `json:"reason"`
	} `json:"headers"`
}

// signatureContract is every delivery the corpus pins a verdict for.
type signatureContract struct {
	Refusals []string `json:"refusals"`
	Vectors  []struct {
		Name             string     `json:"name"`
		Secret           string     `json:"secret"`
		Payload          string     `json:"payload"`
		Headers          [][]string `json:"headers"`
		Signature        string     `json:"signature"`
		CurrentTime      int64      `json:"current_time"`
		ToleranceSeconds int64      `json:"tolerance_seconds"`
		Verdict          string     `json:"verdict"`
		Refusal          string     `json:"refusal"`
		Reason           string     `json:"reason"`
	} `json:"vectors"`
}

// refusalSentinels is how a refusal the corpus names reads in this client's own vocabulary. Every
// name the corpus declares is looked up here, so one added there stops this suite until it is
// mapped rather than passing under whatever the client happened to answer.
var refusalSentinels = map[string]error{
	"code_not_hexadecimal": hook0.ErrSignatureUnreadable,
	"header_not_delivered": hook0.ErrHeaderNotDelivered,
	"code_mismatch":        hook0.ErrSignatureMismatch,
	"outside_tolerance":    hook0.ErrSignatureOutsideTolerance,
}

// corpus reads one document of the shared contract into the shape a case reads it through.
func corpus[Contract any](t *testing.T, document string) Contract {
	t.Helper()

	path := filepath.Join(corpusDirectory, document)
	about, err := os.Stat(path)
	if err != nil {
		t.Fatalf("the shared contract at %s is unreadable: %v", path, err)
	}
	if about.Size() > maxCorpusBytes {
		t.Fatalf("%s is %d bytes long, above the %d read back", path, about.Size(), maxCorpusBytes)
	}

	written, err := os.ReadFile(path) //nolint:gosec // the path is this repository's own corpus
	if err != nil {
		t.Fatalf("the shared contract at %s is unreadable: %v", path, err)
	}

	var read Contract
	if err := json.Unmarshal(written, &read); err != nil {
		t.Fatalf("%s does not read as the contract it is: %v", path, err)
	}
	return read
}

// answered is what the API says when it refuses a request, in the shape every Hook0 failure takes.
func answered(status int, problem string) scriptedResponse {
	return scriptedResponse{
		status: status,
		body: map[string]any{
			"id":     problem,
			"status": status,
			"title":  "refused",
			"detail": "what the corpus scripted",
			"type":   "https://hook0.com/documentation/errors/" + problem,
		},
	}
}

// issuedFor is how many requests a send made when the API answered that way and then took the
// event, and whether the send ended up reporting success.
func issuedFor(t *testing.T, refusal scriptedResponse) (int, bool) {
	t.Helper()

	api := listen(t)
	api.willAnswer(refusal, ingested("a5b4dd60-6ab4-4bd6-9f0b-1a4f8a2a0100"))

	_, err := client(api, promptOptions(4)).SendEvent(bounded(t), anEvent())
	return api.requestCount(), err == nil
}

// TestTheCorpusSaysWhatEveryProblemDoesToASend drives one send per problem the API can report.
//
// The status is not what decides: the corpus carries problems answering the same status with
// opposite verdicts, and a client reading the status alone fails half of them.
func TestTheCorpusSaysWhatEveryProblemDoesToASend(t *testing.T) {
	contract := corpus[retryContract](t, "retry.json")
	if len(contract.Problems) == 0 {
		t.Fatal("the corpus classifies no problem at all")
	}

	for _, rule := range contract.Problems {
		t.Run(rule.Problem, func(t *testing.T) {
			issued, ingested := issuedFor(t, answered(rule.Status, rule.Problem))

			if rule.Retryable {
				if issued != 2 {
					t.Errorf(
						"`%s` under %d issued %d requests where the corpus says it is retryable: %s",
						rule.Problem, rule.Status, issued, rule.Reason,
					)
				}
				if !ingested {
					t.Errorf("the send did not survive a retryable `%s`", rule.Problem)
				}
				return
			}

			if issued != 1 {
				t.Errorf(
					"`%s` under %d issued %d requests where the corpus says it is not retryable: %s",
					rule.Problem, rule.Status, issued, rule.Reason,
				)
			}
			if ingested {
				t.Errorf("a send the API refused with `%s` reported success", rule.Problem)
			}
		})
	}
}

// TestTheCorpusSaysWhatEveryStatusDoesToASend drives one send per status the API answers, with a
// body naming no problem this client could read — which is also what an older client meets when the
// API names a problem it has never heard of.
func TestTheCorpusSaysWhatEveryStatusDoesToASend(t *testing.T) {
	contract := corpus[retryContract](t, "retry.json")
	if len(contract.Statuses) == 0 {
		t.Fatal("the corpus rules on no status at all")
	}

	for _, rule := range contract.Statuses {
		t.Run(fmt.Sprint(rule.Status), func(t *testing.T) {
			issued, _ := issuedFor(t, answered(rule.Status, "AProblemThisClientHasNeverHeardOf"))

			expected := 1
			if rule.Retryable {
				expected = 2
			}
			if issued != expected {
				t.Errorf(
					"a status of %d issued %d requests where the corpus expects %d: %s",
					rule.Status, issued, expected, rule.Reason,
				)
			}
		})
	}
}

// provoked makes this client meet one of the causes the corpus names, for real, and answers
// whether the send survived it and how many attempts it took to give up.
//
// The API takes the event on its second answer, so a cause the corpus calls retryable is one the
// send comes back from, and one it does not is a send that stops at its first attempt.
func provoked(t *testing.T, cause string) (bool, int) {
	t.Helper()

	options := promptOptions(4)
	var sender *hook0.Client

	switch cause {
	case "no_answer":
		api := listen(t)
		api.willAnswer(hangsUp(), ingested("a5b4dd60-6ab4-4bd6-9f0b-1a4f8a2a0101"))
		sender = client(api, options)
	case "answer_above_a_bound":
		api := listen(t)
		api.willAnswer(
			scriptedResponse{status: 200, body: strings.Repeat("x", 1024)},
			ingested("a5b4dd60-6ab4-4bd6-9f0b-1a4f8a2a0103"),
		)
		options.MaxResponseBytes = 64
		sender = client(api, options)
	case "unusable_api_url":
		// Nothing listens, and nothing is sent: the URL names no scheme a request could travel on.
		sender = hook0.NewClient("://nowhere", applicationId, token, options)
	default:
		t.Fatalf("the corpus names the transport cause `%s`, which this suite cannot provoke", cause)
	}

	_, err := sender.SendEvent(bounded(t), anEvent())
	if err == nil {
		return true, 0
	}

	var refusal *hook0.SendError
	if !errors.As(err, &refusal) {
		t.Fatalf("the failure of `%s` is %T, not the one a send reports", cause, err)
	}
	return false, refusal.Attempts
}

// TestTheCorpusSaysWhatEveryTransportCauseDoesToASend drives every failure that produced no answer
// to read.
//
// They arrive as one type in this client as in most runtimes, and only one of them could end
// differently: a client deciding by the type spends four attempts on a mistyped API URL and then
// hands its caller a message that accuses the network.
func TestTheCorpusSaysWhatEveryTransportCauseDoesToASend(t *testing.T) {
	contract := corpus[retryContract](t, "retry.json")
	if len(contract.Transport.Causes) == 0 {
		t.Fatal("the corpus names no cause a failure without an answer can have")
	}

	for _, rule := range contract.Transport.Causes {
		t.Run(rule.Cause, func(t *testing.T) {
			survived, attempts := provoked(t, rule.Cause)

			if rule.Retryable {
				if !survived {
					t.Errorf(
						"a send that met `%s` gave up after %d attempts where the corpus says it is retryable: %s",
						rule.Cause, attempts, rule.Reason,
					)
				}
				return
			}

			if survived {
				t.Fatalf("a send that met `%s` reported success: %s", rule.Cause, rule.Reason)
			}
			if attempts != 1 {
				t.Errorf(
					"`%s` was met %d times where the corpus says repeating it changes nothing: %s",
					rule.Cause, attempts, rule.Reason,
				)
			}
		})
	}
}

// TestAHeadAboveTheCeilingsTheCorpusNamesIsRefused holds the head of an answer to the same bounds
// as its body.
//
// The head is written by the other end, so a client that bounds the body and not the headers has
// only moved where a server spends its caller's memory.
func TestAHeadAboveTheCeilingsTheCorpusNamesIsRefused(t *testing.T) {
	contract := corpus[boundsContract](t, "bounds.json").Bounds

	tooMany := map[string]string{}
	for index := range contract.MaxResponseHeaders + 8 {
		tooMany[fmt.Sprintf("X-Filler-%d", index)] = "filler"
	}
	tooLong := map[string]string{"X-Filler": strings.Repeat("v", contract.MaxHeaderBytes+8)}

	for name, headers := range map[string]map[string]string{
		"more headers than are read":   tooMany,
		"a header longer than is read": tooLong,
	} {
		t.Run(name, func(t *testing.T) {
			api := listen(t)
			api.willAnswer(
				scriptedResponse{status: 200, body: map[string]any{}, headers: headers},
				ingested("a5b4dd60-6ab4-4bd6-9f0b-1a4f8a2a0104"),
			)

			_, err := client(api, promptOptions(4)).SendEvent(bounded(t), anEvent())
			if !errors.Is(err, hook0.ErrAnswerAboveABound) {
				t.Fatalf("a head above what this client reads was answered %v", err)
			}

			var refusal *hook0.SendError
			if errors.As(err, &refusal) && refusal.Attempts != 1 {
				t.Errorf("an answer this client will not read was drawn %d times", refusal.Attempts)
			}
		})
	}
}

// TestEveryRequestCarriesWhatTheCorpusSaysItDoes reads back what actually reached the socket.
//
// A representation a client forgets to ask for costs nothing until the API serves a second one, at
// which point it costs everything, which is exactly the kind of divergence nobody notices by hand.
func TestEveryRequestCarriesWhatTheCorpusSaysItDoes(t *testing.T) {
	contract := corpus[requestContract](t, "request.json")
	if len(contract.Headers) == 0 {
		t.Fatal("the corpus names no header a request carries")
	}

	api := listen(t)
	api.willAnswer(ingested("a5b4dd60-6ab4-4bd6-9f0b-1a4f8a2a0105"))

	if _, err := client(api, promptOptions(4)).SendEvent(bounded(t), anEvent()); err != nil {
		t.Fatalf("the send failed: %v", err)
	}

	// A send carries a body, so every occasion the corpus declares applies to this one request.
	carried := api.requests()[0].headers
	for _, header := range contract.Headers {
		expected := strings.ReplaceAll(header.Value, "${token}", token)
		if written := carried.Get(header.Name); written != expected {
			t.Errorf(
				"the request carried `%s: %s` where the shared contract says `%s`: %s",
				header.Name, written, expected, header.Reason,
			)
		}
	}
}

// TestTheDelayTheAPINamesIsHonouredAndBounded runs every value of the delay header the corpus
// carries.
//
// The header is written by the other end, so honouring it whole would hand a stranger the length of
// this client's send. What the corpus asks for is that a delay be waited out when the budget can
// afford it and cut down to what is left of the budget when it cannot.
func TestTheDelayTheAPINamesIsHonouredAndBounded(t *testing.T) {
	contract := corpus[retryContract](t, "retry.json")
	if len(contract.RetryAfter.Cases) == 0 {
		t.Fatal("the corpus carries no value of the delay header")
	}

	paced, found := retryablePacing(contract)
	if !found {
		t.Fatal("the corpus classifies no problem the API names a delay beside")
	}

	options := promptOptions(4)
	options.RetryPolicy = hook0.RetryPolicy{
		MaxAttempts:    4,
		InitialBackoff: promptBackoff,
		MaxBackoff:     promptBackoff,
		MaxTotalDelay:  delayBudget,
	}

	for _, delay := range contract.RetryAfter.Cases {
		t.Run(delay.Name, func(t *testing.T) {
			refusal := answered(paced.Status, paced.Problem)
			refusal.headers = map[string]string{contract.RetryAfter.Header: delay.Header}

			api := listen(t)
			api.willAnswer(refusal, ingested("a5b4dd60-6ab4-4bd6-9f0b-1a4f8a2a0102"))

			started := time.Now()
			if _, err := client(api, options).SendEvent(bounded(t), anEvent()); err != nil {
				t.Fatalf("the send did not survive a paced answer: %v", err)
			}
			waited := time.Since(started)

			if issued := api.requestCount(); issued != 2 {
				t.Fatalf("a paced answer issued %d requests, not one and its retry", issued)
			}

			expected := time.Duration(0)
			if delay.Honoured {
				expected = min(time.Duration(delay.Seconds)*time.Second, delayBudget)
			}
			if waited < expected {
				t.Errorf(
					"`%s: %s` was retried after %s, sooner than the %s it asked for",
					contract.RetryAfter.Header, delay.Header, waited, expected,
				)
			}
			if waited > expected+delaySlack {
				t.Errorf(
					"`%s: %s` held the send for %s, above the %s it is bounded to",
					contract.RetryAfter.Header, delay.Header, waited, expected,
				)
			}
		})
	}
}

// retryablePacing is a problem the corpus says is worth repeating and that shares its status with
// one it says is not. That is the answer the API names a delay beside, and the one a status alone
// cannot classify.
func retryablePacing(contract retryContract) (struct {
	Problem   string `json:"problem"`
	Status    int    `json:"status"`
	Retryable bool   `json:"retryable"`
	Reason    string `json:"reason"`
}, bool) {
	for _, rule := range contract.Problems {
		if !rule.Retryable {
			continue
		}
		for _, other := range contract.Problems {
			if other.Status == rule.Status && !other.Retryable {
				return rule, true
			}
		}
	}
	return contract.Problems[0], false
}

// TestTheBoundsAreTheOnesTheCorpusNames holds this client's defaults against the one place the
// numbers are written down.
func TestTheBoundsAreTheOnesTheCorpusNames(t *testing.T) {
	contract := corpus[boundsContract](t, "bounds.json").Bounds
	options := hook0.DefaultOptions()
	policy := options.RetryPolicy

	for _, bound := range []struct {
		name    string
		carried int64
		named   int64
	}{
		{"the attempts one send makes", int64(policy.MaxAttempts), int64(contract.MaxAttempts)},
		{"the attempts nothing may cross", int64(hook0.MaxAttemptsCap), int64(contract.MaxAttemptsCap)},
		{"the first delay", policy.InitialBackoff.Milliseconds(), contract.InitialBackoffs},
		{"the ceiling of one delay", policy.MaxBackoff.Milliseconds(), contract.MaxBackoffMs},
		{"the budget every delay shares", policy.MaxTotalDelay.Milliseconds(), contract.MaxTotalDelayMs},
		{"the time one attempt is given", options.RequestTimeout.Milliseconds(), contract.RequestTimeout},
		{"the largest payload sent", int64(options.MaxPayloadBytes), int64(contract.MaxPayloadBytes)},
		{"the largest answer read", options.MaxResponseBytes, contract.MaxResponseBytes},
		{"the most headers read", int64(hook0.MaxResponseHeaders), int64(contract.MaxResponseHeaders)},
		{"the longest header read", int64(hook0.MaxHeaderBytes), int64(contract.MaxHeaderBytes)},
	} {
		if bound.carried != bound.named {
			t.Errorf("%s is %d here and %d in the shared contract", bound.name, bound.carried, bound.named)
		}
	}
}

// TestEveryDeliveryOfTheCorpusIsVerifiedAsItSays runs every signature vector the corpus carries.
//
// A refused vector has to be refused for the reason the corpus names: a client that computed a code
// over a header that never arrived and reported a mismatch would otherwise look right.
func TestEveryDeliveryOfTheCorpusIsVerifiedAsItSays(t *testing.T) {
	contract := corpus[signatureContract](t, "signature.json")
	if len(contract.Vectors) == 0 {
		t.Fatal("the corpus carries no delivery to verify")
	}

	for _, name := range contract.Refusals {
		if _, mapped := refusalSentinels[name]; !mapped {
			t.Errorf("the corpus declares the refusal `%s`, which this suite maps to nothing here", name)
		}
	}

	for _, vector := range contract.Vectors {
		t.Run(vector.Name, func(t *testing.T) {
			delivered := http.Header{}
			for _, pair := range vector.Headers {
				if len(pair) != 2 {
					t.Fatalf("a header of `%s` is not a name and a value", vector.Name)
				}
				delivered.Set(pair[0], pair[1])
			}

			err := hook0.VerifyWebhookSignatureAt(
				vector.Signature,
				[]byte(vector.Payload),
				delivered,
				vector.Secret,
				time.Duration(vector.ToleranceSeconds)*time.Second,
				time.Unix(vector.CurrentTime, 0),
			)

			if vector.Verdict == "accepted" {
				if err != nil {
					t.Errorf("a delivery the corpus accepts was refused as %v: %s", err, vector.Reason)
				}
				return
			}

			sentinel, mapped := refusalSentinels[vector.Refusal]
			if !mapped {
				t.Fatalf("`%s` is refused as `%s`, which this suite maps to nothing here", vector.Name, vector.Refusal)
			}
			if !errors.Is(err, sentinel) {
				t.Errorf(
					"a delivery the corpus refuses as `%s` was answered %v: %s",
					vector.Refusal, err, vector.Reason,
				)
			}
		})
	}
}
