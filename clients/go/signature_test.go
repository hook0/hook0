// What a webhook has to carry to be accepted, and every reason one is refused.
//
// The codes below are fixed vectors, computed once outside this package. A test that signed with
// this package and verified with it would pass whatever the two agreed on; these hold the
// implementation against an answer it had no say in.

package hook0_test

import (
	"errors"
	"net/http"
	"strconv"
	"strings"
	"testing"
	"time"

	hook0 "github.com/hook0/hook0-go/v2"
)

const (
	subscriptionSecret = "a-subscription-secret"
	signedPayload      = `{"event":"user.created"}`
	signedAt           = 1_800_000_000

	// bodyCode covers the moment and the body alone.
	bodyCode = "d17d66b66fca89390c5b967c45e8928fc732db07a0aabe8167b1e98213081ffe"

	// headersCode covers the moment, `x-event-id x-delivery-id`, their values in that order, and
	// the body.
	headersCode = "19a6fb8f6581715b241a93af02a58611c3b0ac7b747a8d2a5b120ee418d0c347"

	// reversedHeadersCode covers the same two headers named and valued the other way round.
	reversedHeadersCode = "bf5f4e5fe0c9143510ae192f983316a09ed392d0104d2ce0faa3d7bce3687acb"

	tolerance = 5 * time.Minute
)

// signedMoment is what the fixed vectors were signed at.
func signedMoment() time.Time {
	return time.Unix(signedAt, 0)
}

// deliveredHeaders are the headers the `v1` vectors cover.
func deliveredHeaders() http.Header {
	headers := http.Header{}
	headers.Set("X-Event-Id", "evt-1")
	headers.Set("X-Delivery-Id", "dlv-1")
	headers.Set("Content-Type", "application/json")
	return headers
}

func verify(signature string, headers http.Header, at time.Time) error {
	return hook0.VerifyWebhookSignatureAt(
		signature,
		[]byte(signedPayload),
		headers,
		subscriptionSecret,
		tolerance,
		at,
	)
}

func TestABodyOnlySignatureIsAccepted(t *testing.T) {
	if err := verify("t="+strconv.Itoa(signedAt)+",v0="+bodyCode, http.Header{}, signedMoment()); err != nil {
		t.Errorf("a delivery signed over its body alone was refused: %v", err)
	}
}

func TestASignatureCoveringHeadersIsAccepted(t *testing.T) {
	signature := "t=" + strconv.Itoa(signedAt) + ",h=x-event-id x-delivery-id,v1=" + headersCode

	if err := verify(signature, deliveredHeaders(), signedMoment()); err != nil {
		t.Errorf("a delivery signed over its headers and its body was refused: %v", err)
	}
}

func TestTheStrongerSchemeIsTheOneVerified(t *testing.T) {
	// A sender offering a body-only code that is right and a header code that is wrong is a sender
	// trying to have the weaker of the two accepted.
	downgraded := "t=" + strconv.Itoa(signedAt) +
		",h=x-event-id x-delivery-id,v0=" + bodyCode + ",v1=" + reversedHeadersCode

	if err := verify(downgraded, deliveredHeaders(), signedMoment()); !errors.Is(err, hook0.ErrSignatureMismatch) {
		t.Errorf("the weaker of two schemes was accepted on the strength of it being offered: %v", err)
	}

	// The other way round: the header code is the right one, and the body-only code beside it is
	// not looked at.
	upgraded := "t=" + strconv.Itoa(signedAt) +
		",h=x-event-id x-delivery-id,v0=" + reversedHeadersCode + ",v1=" + headersCode

	if err := verify(upgraded, deliveredHeaders(), signedMoment()); err != nil {
		t.Errorf("a delivery whose stronger code is the right one was refused: %v", err)
	}
}

func TestTheOrderTheHeadersAreNamedInIsPartOfWhatIsSigned(t *testing.T) {
	// The same two headers, named the other way round, cover their values the other way round.
	signature := "t=" + strconv.Itoa(signedAt) + ",h=x-delivery-id x-event-id,v1=" + headersCode

	if err := verify(signature, deliveredHeaders(), signedMoment()); !errors.Is(err, hook0.ErrSignatureMismatch) {
		t.Errorf("the order the headers are covered in was not part of what was signed: %v", err)
	}

	reversed := "t=" + strconv.Itoa(signedAt) + ",h=x-delivery-id x-event-id,v1=" + reversedHeadersCode

	if err := verify(reversed, deliveredHeaders(), signedMoment()); err != nil {
		t.Errorf("a delivery covering its headers in the order it names them was refused: %v", err)
	}
}

func TestAHeaderTheSignatureCoversAndTheRequestDidNotCarry(t *testing.T) {
	signature := "t=" + strconv.Itoa(signedAt) + ",h=x-event-id x-missing,v1=" + headersCode

	err := verify(signature, deliveredHeaders(), signedMoment())
	if !errors.Is(err, hook0.ErrHeaderNotDelivered) {
		t.Fatalf("a header covered but not delivered was reported as %v", err)
	}
	// It is refused for being absent rather than for the code not matching, which is what says the
	// refusal happened before any code was computed.
	if errors.Is(err, hook0.ErrSignatureMismatch) {
		t.Error("an absent header was signed over as if it had been delivered")
	}
}

func TestACodeThatIsNotWholeHexadecimalIsRefusedRatherThanTruncated(t *testing.T) {
	for _, written := range []string{
		bodyCode[:len(bodyCode)-1] + "z",
		bodyCode[:len(bodyCode)-1],
		"",
	} {
		err := verify("t="+strconv.Itoa(signedAt)+",v0="+written, http.Header{}, signedMoment())
		if !errors.Is(err, hook0.ErrSignatureUnreadable) {
			t.Errorf("`v0=%s` was read as %v rather than refused", written, err)
		}
	}
}

func TestASignatureCarryingNeitherCodeIsRefused(t *testing.T) {
	if err := verify("t="+strconv.Itoa(signedAt)+",h=x-event-id", http.Header{}, signedMoment()); !errors.Is(err, hook0.ErrSignatureUnreadable) {
		t.Errorf("a signature offering no code at all was read as %v", err)
	}
}

func TestTheClockWindowIsBilateral(t *testing.T) {
	signature := "t=" + strconv.Itoa(signedAt) + ",v0=" + bodyCode

	tooOld := signedMoment().Add(tolerance + time.Second)
	if err := verify(signature, http.Header{}, tooOld); !errors.Is(err, hook0.ErrSignatureOutsideTolerance) {
		t.Errorf("a delivery signed longer ago than the tolerance was read as %v", err)
	}

	tooNew := signedMoment().Add(-tolerance - time.Second)
	if err := verify(signature, http.Header{}, tooNew); !errors.Is(err, hook0.ErrSignatureOutsideTolerance) {
		t.Errorf("a delivery dated further in the future than the tolerance was read as %v", err)
	}

	for _, at := range []time.Time{
		signedMoment().Add(tolerance - time.Second),
		signedMoment().Add(-tolerance + time.Second),
	} {
		if err := verify(signature, http.Header{}, at); err != nil {
			t.Errorf("a delivery inside the window was refused at %s: %v", at, err)
		}
	}
}

func TestAMomentThatIsNotANumberOfSeconds(t *testing.T) {
	for _, written := range []string{"not-a-moment", "", strings.Repeat("9", 200)} {
		err := verify("t="+written+",v0="+bodyCode, http.Header{}, signedMoment())
		if !errors.Is(err, hook0.ErrSignatureUnreadable) {
			t.Errorf("`t=%s` was read as %v rather than refused", written, err)
		}
	}
}

func TestASignatureLongerThanIsReadIsRefusedBeforeItIsSplit(t *testing.T) {
	// The header is written by whoever delivers the webhook, so what it costs to read is bounded
	// rather than left to them.
	long := "t=" + strconv.Itoa(signedAt) + ",v0=" + strings.Repeat("a", 9*1024)

	err := verify(long, deliveredHeaders(), signedMoment())

	if !errors.Is(err, hook0.ErrSignatureUnreadable) {
		t.Errorf("a signature of %d characters was read as %v", len(long), err)
	}
}

func TestASignatureCarryingMoreThanTheAcceptedPartsIsRefused(t *testing.T) {
	parts := []string{"t=" + strconv.Itoa(signedAt), "v0=" + bodyCode}
	for index := range 64 {
		parts = append(parts, "x"+strconv.Itoa(index)+"=a")
	}

	err := verify(strings.Join(parts, ","), deliveredHeaders(), signedMoment())

	if !errors.Is(err, hook0.ErrSignatureUnreadable) {
		t.Errorf("a signature of %d parts was read as %v", len(parts), err)
	}
}

func TestASignatureCoveringMoreHeadersThanAreAcceptedIsRefused(t *testing.T) {
	names := make([]string, 0, 128)
	for index := range 128 {
		names = append(names, "x-covered-"+strconv.Itoa(index))
	}
	signature := "t=" + strconv.Itoa(signedAt) + ",h=" + strings.Join(names, " ") + ",v1=" + headersCode

	err := verify(signature, deliveredHeaders(), signedMoment())

	if !errors.Is(err, hook0.ErrSignatureUnreadable) {
		t.Errorf("a signature covering %d headers was read as %v", len(names), err)
	}
}

func TestASignatureNamingNoMomentIsRefused(t *testing.T) {
	err := verify("v0="+bodyCode, deliveredHeaders(), signedMoment())

	if !errors.Is(err, hook0.ErrSignatureUnreadable) {
		t.Errorf("a signature naming no moment was read as %v", err)
	}
}

func TestAMomentFurtherFromTheEpochThanASignatureCanNameIsRefused(t *testing.T) {
	// Beyond this a moment is not a delivery that happened, and the arithmetic that compares it
	// against the clock is what a number this size is written to break.
	for _, seconds := range []string{"9223372036854", "-9223372036854"} {
		err := verify("t="+seconds+",v0="+bodyCode, deliveredHeaders(), signedMoment())

		if !errors.Is(err, hook0.ErrSignatureUnreadable) {
			t.Errorf("a signature naming %s seconds was read as %v", seconds, err)
		}
	}
}

func TestAHeaderDeliveredWithoutAValueIsNotOneASignatureCanCover(t *testing.T) {
	// A header key carrying no value at all is not the same as one carrying the empty value, and
	// signing over the first would let a sender drop a header and keep the signature valid.
	headers := deliveredHeaders()
	headers["X-Event-Id"] = nil

	err := verify("t="+strconv.Itoa(signedAt)+",h=x-event-id x-delivery-id,v1="+headersCode, headers, signedMoment())

	if !errors.Is(err, hook0.ErrHeaderNotDelivered) {
		t.Errorf("a header delivered without a value was read as %v", err)
	}
}

func TestASignatureNamingACodeButNoMomentIsRefused(t *testing.T) {
	// Two parts, so the signature is not the empty one, and neither of them the moment a code is
	// computed over.
	err := verify("v0="+bodyCode+",h=x-event-id", deliveredHeaders(), signedMoment())

	if !errors.Is(err, hook0.ErrSignatureUnreadable) {
		t.Errorf("a signature naming no moment was read as %v", err)
	}
}

func TestTheStrongerCodeIsAlsoRefusedWhenItIsNotWholeHexadecimal(t *testing.T) {
	// Both codes are read, and neither is truncated to whatever prefix happened to decode.
	err := verify("t="+strconv.Itoa(signedAt)+",h=x-event-id x-delivery-id,v1=not-hexadecimal", deliveredHeaders(), signedMoment())

	if !errors.Is(err, hook0.ErrSignatureUnreadable) {
		t.Errorf("a `v1` code that is not hexadecimal was read as %v", err)
	}
}

func TestASignatureCoveringAHeaderNameThatIsNotOneIsRefused(t *testing.T) {
	err := verify("t="+strconv.Itoa(signedAt)+",h=x-event@id x-delivery-id,v1="+headersCode, deliveredHeaders(), signedMoment())

	if !errors.Is(err, hook0.ErrSignatureUnreadable) {
		t.Errorf("a signature covering a name that is not a header name was read as %v", err)
	}
}
