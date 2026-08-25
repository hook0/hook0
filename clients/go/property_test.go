// What holds for every input, rather than for the ones a case happened to pick.
//
// Three things are checked here. A retry schedule never spends more than the policy that produced it
// allows, whichever way the randomness fell. Reading a signature header answers with one of the
// reasons this package declares, whatever text reached the endpoint, and never with a panic. And a
// value written back the way the API reads it is the value that was read.
//
// Each of them is a fuzz target with a corpus committed beside it, so the counterexamples that were
// worth finding run as ordinary cases on every pipeline, and the search itself runs under a
// deadline rather than until somebody stops it.

package hook0_test

import (
	"bytes"
	"encoding/json"
	"errors"
	"net/http"
	"testing"
	"time"

	hook0 "github.com/hook0/hook0-go/v2"
	"github.com/hook0/hook0-go/v2/generated"
)

// refusals are every reason this package declines to verify a webhook. A refusal that is none of
// them is one a caller cannot tell apart from another.
var refusals = []error{
	hook0.ErrSignatureUnreadable,
	hook0.ErrHeaderNotDelivered,
	hook0.ErrSignatureMismatch,
	hook0.ErrSignatureOutsideTolerance,
}

func FuzzRetryScheduleStaysWithinEveryBound(f *testing.F) {
	f.Fuzz(func(
		t *testing.T,
		maxAttempts int,
		initialBackoff int64,
		maxBackoff int64,
		maxTotalDelay int64,
		firstDraw float64,
		secondDraw float64,
	) {
		policy := hook0.RetryPolicy{
			MaxAttempts:    maxAttempts,
			InitialBackoff: time.Duration(initialBackoff),
			MaxBackoff:     time.Duration(maxBackoff),
			MaxTotalDelay:  time.Duration(maxTotalDelay),
		}
		delays := policy.Delays([]float64{firstDraw, secondDraw})

		attempts := policy.Attempts()
		if attempts < 1 || attempts > hook0.MaxAttemptsCap {
			t.Fatalf("a policy of %d attempts makes %d of them", maxAttempts, attempts)
		}
		if len(delays) > attempts-1 {
			t.Fatalf("%d attempts are spaced out by %d delays", attempts, len(delays))
		}

		budget := max(policy.MaxTotalDelay, 0)
		ceiling := max(policy.MaxBackoff, 0)

		// Written as what is left of the budget rather than as a running sum, so the check itself
		// cannot carry two durations past what one can hold.
		var spent time.Duration
		for index, delay := range delays {
			if delay < 0 {
				t.Fatalf("retry %d waits for %s", index+1, delay)
			}
			if delay > policy.BackoffCeiling(index+1) {
				t.Fatalf("retry %d waits %s, above its ceiling of %s", index+1, delay, policy.BackoffCeiling(index+1))
			}
			if delay > ceiling {
				t.Fatalf("retry %d waits %s, above the %s no delay exceeds", index+1, delay, ceiling)
			}
			if delay > budget-spent {
				t.Fatalf("the schedule spends more than the %s budget its delays share", budget)
			}
			spent += delay
		}

		// A schedule never hurries up as it goes: the ceiling of a retry never sits below the one
		// before it.
		previous := policy.BackoffCeiling(1)
		for retry := 2; retry <= attempts; retry++ {
			current := policy.BackoffCeiling(retry)
			if current < previous {
				t.Fatalf("retry %d has a ceiling of %s where retry %d had %s", retry, current, retry-1, previous)
			}
			previous = current
		}
	})
}

func FuzzReadingASignatureAnswersOneOfTheDeclaredRefusals(f *testing.F) {
	f.Fuzz(func(t *testing.T, header string, payload []byte) {
		// Parsing on its own must answer the same way: everything below it depends on it not
		// answering something a caller cannot name.
		if _, err := hook0.ParseSignature(header); err != nil && !errors.Is(err, hook0.ErrSignatureUnreadable) {
			t.Fatalf("reading %q answered %v, which is not a reason this package declares", header, err)
		}

		err := hook0.VerifyWebhookSignatureAt(
			header,
			payload,
			deliveredHeaders(),
			subscriptionSecret,
			tolerance,
			signedMoment(),
		)
		if err == nil {
			return
		}
		for _, declared := range refusals {
			if errors.Is(err, declared) {
				return
			}
		}
		t.Fatalf("verifying %q answered %v, which is not a reason this package declares", header, err)
	})
}

func FuzzAGeneratedTypeWritesBackWhatItRead(f *testing.F) {
	f.Fuzz(func(t *testing.T, document []byte) {
		// Two types, picked for the shapes they cover between them: the error contract's own, whose
		// discriminant is a closed list and whose last member is a value the document does not
		// describe, and the deepest one the API declares, which nests objects, an identifier, and
		// members it does not require.
		roundTrips[generated.Problem](t, document)
		roundTrips[generated.ApplicationInfo](t, document)
	})
}

// roundTrips checks that a document this type could read is written back as the document it read.
//
// What is held against what is not the bytes that arrived but the bytes the type writes: a document
// may spell a number or an escape more ways than one, and the question is whether the value
// survives, not whether the sender's spelling does.
func roundTrips[Value any](t *testing.T, document []byte) {
	t.Helper()

	var read Value
	if err := json.Unmarshal(document, &read); err != nil {
		return
	}

	written, err := json.Marshal(read)
	if err != nil {
		t.Fatalf("a %T read out of %q cannot be written back: %v", read, document, err)
	}

	var again Value
	if err := json.Unmarshal(written, &again); err != nil {
		t.Fatalf("a %T writes back %q, which it cannot read: %v", read, written, err)
	}

	rewritten, err := json.Marshal(again)
	if err != nil {
		t.Fatalf("a %T read out of its own %q cannot be written back: %v", read, written, err)
	}
	if !bytes.Equal(written, rewritten) {
		t.Fatalf("a %T read out of %q writes back %q, then %q", read, document, written, rewritten)
	}
}

func TestTheDefaultScheduleDoublesUpToItsCeiling(t *testing.T) {
	policy := hook0.DefaultRetryPolicy()

	for _, expected := range []struct {
		retry   int
		ceiling time.Duration
	}{
		{1, 100 * time.Millisecond},
		{2, 200 * time.Millisecond},
		{3, 400 * time.Millisecond},
		{6, 2 * time.Second},
		{16, 2 * time.Second},
	} {
		if reached := policy.BackoffCeiling(expected.retry); reached != expected.ceiling {
			t.Errorf("retry %d has a ceiling of %s, not %s", expected.retry, reached, expected.ceiling)
		}
	}

	// A source of randomness that gives nothing asks for the whole ceiling, which is what makes the
	// budget the thing that cuts the schedule short.
	delays := policy.Delays(nil)
	if len(delays) != 3 {
		t.Fatalf("four attempts are spaced out by %d delays", len(delays))
	}

	var spent time.Duration
	for _, delay := range delays {
		spent += delay
	}
	if spent > policy.MaxTotalDelay {
		t.Errorf("the schedule spends %s of a %s budget", spent, policy.MaxTotalDelay)
	}
}

func TestADisabledPolicyWaitsForNothing(t *testing.T) {
	policy := hook0.DisabledRetryPolicy()

	if attempts := policy.Attempts(); attempts != 1 {
		t.Errorf("a disabled policy makes %d attempts", attempts)
	}
	if delays := policy.Delays([]float64{1, 1, 1}); len(delays) != 0 {
		t.Errorf("a disabled policy waits %v between its attempts", delays)
	}
}

func TestAPopulatedValueSurvivesBeingWrittenAndReadBack(t *testing.T) {
	document, err := json.Marshal(anApplication())
	if err != nil {
		t.Fatalf("the case cannot write the document it starts from: %v", err)
	}

	var read generated.ApplicationInfo
	if err := json.Unmarshal(document, &read); err != nil {
		t.Fatalf("the document the API answers cannot be read: %v", err)
	}

	written, err := json.Marshal(read)
	if err != nil {
		t.Fatalf("what was read cannot be written back: %v", err)
	}

	var again generated.ApplicationInfo
	if err := json.Unmarshal(written, &again); err != nil {
		t.Fatalf("what was written back cannot be read: %v", err)
	}

	if again.ApplicationId != read.ApplicationId ||
		again.Name != read.Name ||
		again.OrganizationId != read.OrganizationId ||
		again.Quotas.EventsPerDayLimit != read.Quotas.EventsPerDayLimit ||
		again.OnboardingSteps.Event != read.OnboardingSteps.Event {
		t.Errorf("the value changed on the way round: %+v became %+v", read, again)
	}
}

func TestAMemberTheDocumentDoesNotRequireIsAbsentRatherThanZero(t *testing.T) {
	var read generated.Problem
	if err := json.Unmarshal([]byte(`{"id":"NotFound","title":"t","detail":"d","status":404,"type":"u"}`), &read); err != nil {
		t.Fatalf("a problem document cannot be read: %v", err)
	}

	written, err := json.Marshal(read)
	if err != nil {
		t.Fatalf("a problem cannot be written back: %v", err)
	}
	if bytes.Contains(written, []byte(`"validation"`)) {
		t.Errorf("a member the API did not answer was written back anyway: %s", written)
	}
}

func TestVerifyingAgainstTheCurrentMomentRefusesAStaleDelivery(t *testing.T) {
	// The fixed vectors were signed long enough ago that no clock puts them inside the window.
	err := hook0.VerifyWebhookSignature(
		"t="+"1800000000"+",v0="+bodyCode,
		[]byte(signedPayload),
		http.Header{},
		subscriptionSecret,
		tolerance,
	)
	if !errors.Is(err, hook0.ErrSignatureOutsideTolerance) {
		t.Errorf("a delivery signed years ago was read as %v", err)
	}
}
