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
	"strconv"
	"strings"
	"testing"
	"time"

	hook0 "github.com/hook0/hook0-go"
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
//
// The numbers are read under the names the corpus writes them under, rather than into fields
// declared here: a bound added there is then one this suite sees and holds this client to, instead
// of one that lands in no field and is dropped on the floor while every case still passes.
type boundsContract struct {
	Bounds map[string]int64 `json:"bounds"`
}

// boundOf is the number the corpus writes under one name, refusing a name it does not carry: a case
// reading a bound the corpus no longer names would otherwise hold this client to zero.
func boundOf(t *testing.T, contract boundsContract, name string) int64 {
	t.Helper()

	named, carried := contract.Bounds[name]
	if !carried {
		t.Fatalf("the corpus names no bound `%s`", name)
	}
	return named
}

// requestContract is what the corpus says every request carries beside its body.
type requestContract struct {
	Occasions        []string `json:"occasions"`
	MaxComposedBytes int      `json:"max_composed_bytes"`
	Headers          []struct {
		Name   string `json:"name"`
		Value  string `json:"value"`
		When   string `json:"when"`
		Reason string `json:"reason"`
	} `json:"headers"`
}

// templateChunks is what a value of the request document is made of, once the holes this suite can
// speak for are filled in.
//
// A value is a template: `${name}` is a hole and everything around it is literal. A hole named in
// bound becomes part of the literal text around it; one that is not is a hole no suite can fill
// without reimplementing the client it is testing, and it separates two chunks. A template whose
// holes are all bound is therefore one chunk, and the whole value is that chunk.
func templateChunks(template string, bound map[string]string) []string {
	chunks := []string{""}
	rest := template

	for {
		opened := strings.Index(rest, "${")
		if opened < 0 {
			break
		}
		closed := strings.Index(rest[opened:], "}")
		if closed < 0 {
			break
		}
		closed += opened

		last := len(chunks) - 1
		chunks[last] += rest[:opened]
		if filled, named := bound[rest[opened+2:closed]]; named {
			chunks[last] += filled
		} else {
			chunks = append(chunks, "")
		}
		rest = rest[closed+1:]
	}

	chunks[len(chunks)-1] += rest
	return chunks
}

// matchesChunks says whether what arrived is what those chunks describe: the literal text in order,
// anchored at both ends, with something non-empty standing in every hole between them.
func matchesChunks(chunks []string, carried string) bool {
	if len(chunks) == 1 {
		return carried == chunks[0]
	}
	rest, anchored := strings.CutPrefix(carried, chunks[0])
	if !anchored {
		return false
	}

	for _, chunk := range chunks[1 : len(chunks)-1] {
		// A hole stands before this chunk, and nothing is not something, so the search starts past
		// whatever fills it.
		if rest == "" {
			return false
		}
		found := strings.Index(rest[1:], chunk)
		if found < 0 {
			return false
		}
		rest = rest[1+found+len(chunk):]
	}

	last := chunks[len(chunks)-1]
	return len(rest) > len(last) && strings.HasSuffix(rest, last)
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

// headPadding is filler an answer's head is scripted with: how many lines it carries, and how many
// bytes of name and value each of them weighs.
type headPadding struct {
	lines int
	bytes int
}

// carried is filler of that shape, as headers an answer can be scripted with.
func (p headPadding) carried() map[string]string {
	filler := map[string]string{}
	for index := range p.lines {
		name := fmt.Sprintf("X-Filler-%d", index)
		filler[name] = strings.Repeat("v", max(p.bytes-len(name), 1))
	}
	return filler
}

// TestAHeadAboveTheCeilingsTheCorpusNamesIsRefused holds the head of an answer to the same bounds
// as its body.
//
// The head is written by the other end, so a client that bounds the body and not the headers has
// only moved where a server spends its caller's memory. Each shape below crosses exactly one of the
// ceilings the corpus names, and the refusal is read for the ceiling it names: a head built to
// cross the total and refused on the line count would have proved the opposite of what it is here
// for, and would have passed just as quietly.
func TestAHeadAboveTheCeilingsTheCorpusNamesIsRefused(t *testing.T) {
	contract := corpus[boundsContract](t, "bounds.json")
	headers := boundOf(t, contract, "max_response_headers")
	perLine := boundOf(t, contract, "max_header_bytes")
	whole := boundOf(t, contract, "max_head_bytes")

	for _, abusive := range []struct {
		name    string
		padding headPadding
		says    string
	}{
		{
			name:    "more headers than are read",
			padding: headPadding{lines: int(headers) + 8, bytes: 16},
			says:    fmt.Sprintf("above the %d read at most", headers),
		},
		{
			name:    "a header longer than is read",
			padding: headPadding{lines: 1, bytes: int(perLine) + 8},
			says:    fmt.Sprintf("above the %d bytes read at most", perLine),
		},
		{
			// A few wide lines rather than many narrow ones: this is the only one of the three that
			// bounds what a head costs, since a line count and a size per line multiply and neither
			// refuses a head that stays under both. Eight lines a quarter of the whole-head ceiling
			// each weigh twice that ceiling while sitting nowhere near the other two — a shape just
			// inside the line count would be refused on the count instead, the answer's own
			// `Content-Type`, `Content-Length` and `Date` being counted beside the filler.
			name:    "a whole head longer than is read",
			padding: headPadding{lines: 8, bytes: int(whole) / 4},
			says:    fmt.Sprintf("above the %d read at most", whole),
		},
	} {
		t.Run(abusive.name, func(t *testing.T) {
			api := listen(t)
			api.willAnswer(
				scriptedResponse{status: 200, body: map[string]any{}, headers: abusive.padding.carried()},
				ingested("a5b4dd60-6ab4-4bd6-9f0b-1a4f8a2a0104"),
			)

			_, err := client(api, promptOptions(4)).SendEvent(bounded(t), anEvent())
			if !errors.Is(err, hook0.ErrAnswerAboveABound) {
				t.Fatalf("a head above what this client reads was answered %v", err)
			}
			if !strings.Contains(err.Error(), abusive.says) {
				t.Errorf(
					"a head built to cross the ceiling of %s was refused as `%v`, which names another one",
					abusive.says, err,
				)
			}

			var refusal *hook0.SendError
			if errors.As(err, &refusal) && refusal.Attempts != 1 {
				t.Errorf("an answer this client will not read was drawn %d times", refusal.Attempts)
			}
		})
	}
}

// TestAHeadWellUnderTheCeilingIsRead drives an answer whose whole head weighs half of what a head
// may weigh.
//
// Only the refusal above the bound is a property this client owns. Whether a head just under it is
// read at all is settled by the runtime before this client is consulted, and the strictest runtime
// any target runs on draws its own line a little above the number the corpus names — so the
// accepting side is exercised well clear of that band, where every runtime agrees, and the band
// itself is left untested rather than pinned to the build of the day.
func TestAHeadWellUnderTheCeilingIsRead(t *testing.T) {
	contract := corpus[boundsContract](t, "bounds.json")
	padding := headPadding{lines: 8, bytes: int(boundOf(t, contract, "max_head_bytes")) / 16}

	eventId := "a5b4dd60-6ab4-4bd6-9f0b-1a4f8a2a0105"
	answer := ingested(eventId)
	answer.headers = padding.carried()

	api := listen(t)
	api.willAnswer(answer)

	sent, err := client(api, promptOptions(4)).SendEvent(bounded(t), anEvent())
	if err != nil {
		t.Fatalf("an answer carrying %d bytes of head was refused: %v", padding.lines*padding.bytes, err)
	}
	if sent != eventId {
		t.Errorf("the send answered `%s`, not the identifier the API ingested it under", sent)
	}
	if issued := api.requestCount(); issued != 1 {
		t.Errorf("an answer this client reads was drawn %d times", issued)
	}
}

// isTheOccasion is how each occasion the corpus declares reads against one request that reached the
// API. Every occasion the corpus names is looked up here, so one added there stops this suite until
// it is answered rather than passing under whatever this client happened to send.
//
// What decides is the request as the API received it, not the call that produced it: a body is a
// body because one arrived.
var isTheOccasion = map[string]func(receivedRequest) bool{
	"every request":             func(receivedRequest) bool { return true },
	"a request carrying a body": func(request receivedRequest) bool { return request.body != "" },
}

// TestEveryRequestCarriesWhatTheCorpusSaysItDoes reads back what actually reached the socket, on
// each of the occasions the corpus declares.
//
// A representation a client forgets to ask for costs nothing until the API serves a second one, at
// which point it costs everything, which is exactly the kind of divergence nobody notices by hand.
// The occasion is half of the contract: a header the corpus carries on requests that have a body is
// one a bodiless request has nothing to declare, and a suite that only ever sends a body would never
// find out either way.
func TestEveryRequestCarriesWhatTheCorpusSaysItDoes(t *testing.T) {
	contract := corpus[requestContract](t, "request.json")
	if len(contract.Headers) == 0 {
		t.Fatal("the corpus names no header a request carries")
	}
	if len(contract.Occasions) == 0 {
		t.Fatal("the corpus declares no occasion a header is carried on")
	}
	for _, occasion := range contract.Occasions {
		if _, answered := isTheOccasion[occasion]; !answered {
			t.Fatalf(
				"the corpus declares the occasion `%s`, which this suite cannot tell one request by",
				occasion,
			)
		}
	}
	for _, header := range contract.Headers {
		if _, answered := isTheOccasion[header.When]; !answered {
			t.Fatalf(
				"the corpus carries `%s` on `%s`, which is no occasion this suite can tell one request by",
				header.Name, header.When,
			)
		}
	}

	// Declaring an event type the application does not have reads what it declares and then writes
	// the one it lacks: a request carrying no body, and one carrying a body, which between them are
	// every occasion the corpus declares.
	api := listen(t)
	api.willAnswer(
		scriptedResponse{status: 200, body: []any{}},
		scriptedResponse{status: 201, body: map[string]any{}},
	)

	options := promptOptions(4)
	_, err := client(api, options).UpsertEventTypes(bounded(t), []string{"auth.user.create"})
	if err != nil {
		t.Fatalf("declaring an event type failed: %v", err)
	}

	// The holes this suite can speak for: the credential this client was built with, the target
	// reading the corpus, and the retry policy that client was configured with — read back off that
	// policy rather than written out here, so the case cannot agree with a client that states a
	// schedule nobody configured. What is left over is a hole no suite fills without reimplementing
	// the client it is testing.
	policy := options.RetryPolicy
	bound := map[string]string{
		"token":      token,
		"language":   "go",
		"attempts":   strconv.Itoa(policy.Attempts()),
		"backoff_ms": strconv.FormatInt(policy.InitialBackoff.Milliseconds(), 10),
		"ceiling_ms": strconv.FormatInt(policy.MaxBackoff.Milliseconds(), 10),
		"budget_ms":  strconv.FormatInt(policy.MaxTotalDelay.Milliseconds(), 10),
	}

	exercised := map[string]bool{}
	for index, request := range api.requests() {
		for _, occasion := range contract.Occasions {
			if isTheOccasion[occasion](request) {
				exercised[occasion] = true
			}
		}

		for _, header := range contract.Headers {
			// Get compares the name as HTTP does, without regard to the case it is written in.
			written := request.headers.Get(header.Name)
			carried := isTheOccasion[header.When](request)

			if !carried {
				if written != "" {
					t.Errorf(
						"request %d (%s %s) carried `%s: %s`, which the shared contract carries on `%s` alone: %s",
						index, request.method, request.target, header.Name, written, header.When, header.Reason,
					)
				}
				continue
			}

			chunks := templateChunks(header.Value, bound)
			if !matchesChunks(chunks, written) {
				t.Errorf(
					"request %d (%s %s) carried `%s: %s` where the shared contract says `%s` on `%s`: %s",
					index, request.method, request.target, header.Name, written, header.Value, header.When, header.Reason,
				)
			}

			// A value with a hole this suite cannot fill is one the client composed out of what the
			// platform told it, and what the platform says is as long as it feels like.
			if len(chunks) > 1 && len(written) > contract.MaxComposedBytes {
				t.Errorf(
					"request %d (%s %s) carried %d bytes of `%s`, above the %d the shared contract cuts a composed value to",
					index, request.method, request.target, len(written), header.Name, contract.MaxComposedBytes,
				)
			}
		}
	}

	// A contract held against requests that never happened is a contract held against nothing.
	for _, occasion := range contract.Occasions {
		if !exercised[occasion] {
			t.Errorf(
				"nothing this suite sent is `%s`, so what the corpus carries on that occasion was held to nothing",
				occasion,
			)
		}
	}
}

// TestAPolicyAskingForMoreAttemptsThanTheCapStatesTheCap holds the options header to what the
// client will do rather than to what it was handed.
//
// This is the one place where those two come apart, and so the one place the header can be read two
// ways. A client stating the number it was asked for puts a reader on watch for a burst that cannot
// arrive, so the cap is what goes on the wire. The cap is read from the corpus, so every target
// answers to one number.
func TestAPolicyAskingForMoreAttemptsThanTheCapStatesTheCap(t *testing.T) {
	capped := int(boundOf(t, corpus[boundsContract](t, "bounds.json"), "max_attempts_cap"))

	api := listen(t)
	api.willAnswer(ingested("a5b4dd60-6ab4-4bd6-9f0b-1a4f8a2a0003"))

	if _, err := client(api, promptOptions(capped+1)).SendEvent(bounded(t), anEvent()); err != nil {
		t.Fatalf("a send under a policy asking for more attempts than the cap failed: %v", err)
	}

	stated := api.requests()[0].headers.Get("Hook0-Client-Options")
	if wanted := fmt.Sprintf("attempts=%d,", capped); !strings.HasPrefix(stated, wanted) {
		t.Errorf(
			"a policy asking for %d attempts states `%s`, where the cap is %d",
			capped+1, stated, capped,
		)
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
//
// Both sides are discovered: the names come out of the corpus and the values out of this client, so
// a ceiling added there and applied nowhere here is named as missing rather than quietly skipped by
// a case that only ever checks the ones it already knew about.
func TestTheBoundsAreTheOnesTheCorpusNames(t *testing.T) {
	contract := corpus[boundsContract](t, "bounds.json")
	if len(contract.Bounds) == 0 {
		t.Fatal("the corpus names no bound at all")
	}

	options := hook0.DefaultOptions()
	policy := options.RetryPolicy
	applied := map[string]int64{
		"max_attempts":         int64(policy.MaxAttempts),
		"max_attempts_cap":     int64(hook0.MaxAttemptsCap),
		"initial_backoff_ms":   policy.InitialBackoff.Milliseconds(),
		"max_backoff_ms":       policy.MaxBackoff.Milliseconds(),
		"max_total_delay_ms":   policy.MaxTotalDelay.Milliseconds(),
		"request_timeout_ms":   options.RequestTimeout.Milliseconds(),
		"max_payload_bytes":    int64(options.MaxPayloadBytes),
		"max_response_bytes":   options.MaxResponseBytes,
		"max_response_headers": int64(hook0.MaxResponseHeaders),
		"max_header_bytes":     int64(hook0.MaxHeaderBytes),
		"max_head_bytes":       int64(hook0.MaxHeadBytes),
	}

	for name, named := range contract.Bounds {
		carried, applies := applied[name]
		if !applies {
			t.Errorf("the corpus names the bound `%s`, which this client does not apply", name)
			continue
		}
		if carried != named {
			t.Errorf("`%s` is %d here and %d in the shared contract", name, carried, named)
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
