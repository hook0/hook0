// Verifying that a webhook came from Hook0, and that nothing in it changed on the way.
//
// A signature header names the moment it was signed and one or two message authentication codes
// over the body. The `v1` scheme also covers a list of request headers, so a receiver can tell apart
// two deliveries that carry the same body but not the same context; `v0` covers the body alone and
// is what an older sender still produces. When both are offered, `v1` is the one verified: accepting
// the weaker of two schemes on the strength of the sender offering it is how a downgrade works.
//
// Two things are refused before any code is computed. A header the signature says it covers but the
// request did not carry is refused outright, because signing over an absent value would let a sender
// drop a header and keep the signature valid. And a signature whose codes are not whole hexadecimal
// is refused rather than decoded as far as it goes: a decoder that stops at the first bad character
// compares a prefix, and a prefix of the right code is not the right code.
//
// The clock window is bilateral. A moment too far in the future is refused exactly like one too far
// in the past, so the window a given delivery is accepted in stays the width the caller asked for,
// whichever way a clock drifted.

package hook0

import (
	"crypto/hmac"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"net/http"
	"strconv"
	"strings"
	"time"
)

const (
	// maxSignatureBytes is the longest signature header read. The header is written by whoever
	// reached the endpoint, so its size is bounded before any of it is split, decoded or compared.
	maxSignatureBytes = 8 * 1024

	// maxSignatureParts is the most `key=value` parts one signature header is split into.
	maxSignatureParts = 32

	// maxCoveredHeaders is the most header names one signature covers.
	maxCoveredHeaders = 64

	// maxTimestampSeconds is the furthest from the epoch, in either direction, a signature's moment
	// may sit. Holding a moment thousands of years out against the current time is arithmetic no
	// caller expects to be doing.
	maxTimestampSeconds = 1_000_000_000_000

	// partSeparator is what separates one part of the signature header from the next.
	partSeparator = ","

	// partAssignator is what separates the name of a part from its value. Only the first one
	// counts: a value may hold further ones, and splitting on all of them would silently drop
	// everything past the second.
	partAssignator = "="

	// headerNameSeparator is what separates two header names inside the `h` part, and what they are
	// joined back with.
	headerNameSeparator = " "

	// messageSeparator is what separates the pieces of the message a code is computed over.
	messageSeparator = "."

	// timestampPart names the moment the delivery was signed, in whole seconds since the epoch.
	timestampPart = "t"

	// bodySchemePart carries the code covering the body alone.
	bodySchemePart = "v0"

	// headersSchemePart carries the code covering the covered headers and the body.
	headersSchemePart = "v1"

	// coveredHeadersPart lists the headers the `v1` code covers, in the order it covers them.
	coveredHeadersPart = "h"

	// headerNameCharacters is what a header name is written with, as RFC 9110 spells a token.
	headerNameCharacters = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!#$%&'*+-.^_`|~"
)

// Signature is a signature header, read into the pieces a verification needs.
type Signature struct {
	// Timestamp is the moment the delivery was signed, in whole seconds since the epoch.
	Timestamp int64
	// CoveredHeaders names the headers the stronger scheme covers, in the order it covers them and
	// lowercased.
	CoveredHeaders []string
	// BodyCode is the `v0` code, nil when the signature offers none.
	BodyCode []byte
	// HeadersCode is the `v1` code, nil when the signature offers none.
	HeadersCode []byte
}

// ParseSignature reads a signature header, refusing anything it cannot read whole.
func ParseSignature(signature string) (*Signature, error) {
	if len(signature) > maxSignatureBytes {
		return nil, fmt.Errorf(
			"%w: it is %d characters long, above the %d accepted",
			ErrSignatureUnreadable, len(signature), maxSignatureBytes,
		)
	}

	parts := strings.Split(signature, partSeparator)
	if len(parts) > maxSignatureParts {
		return nil, fmt.Errorf(
			"%w: it carries more than the %d parts accepted",
			ErrSignatureUnreadable, maxSignatureParts,
		)
	}

	read := make(map[string]string, len(parts))
	for _, part := range parts {
		name, value, found := strings.Cut(part, partAssignator)
		if !found {
			continue
		}
		read[strings.TrimSpace(name)] = strings.TrimSpace(value)
	}
	if len(read) < 2 {
		return nil, fmt.Errorf("%w: it carries neither a moment nor a code", ErrSignatureUnreadable)
	}

	timestamp, err := signatureTimestamp(read)
	if err != nil {
		return nil, err
	}
	covered, err := coveredHeaders(read)
	if err != nil {
		return nil, err
	}
	bodyCode, err := signatureCode(read, bodySchemePart)
	if err != nil {
		return nil, err
	}
	headersCode, err := signatureCode(read, headersSchemePart)
	if err != nil {
		return nil, err
	}
	if bodyCode == nil && headersCode == nil {
		return nil, fmt.Errorf(
			"%w: it carries neither a `%s` nor a `%s` code",
			ErrSignatureUnreadable, bodySchemePart, headersSchemePart,
		)
	}

	return &Signature{
		Timestamp:      timestamp,
		CoveredHeaders: covered,
		BodyCode:       bodyCode,
		HeadersCode:    headersCode,
	}, nil
}

// Verify reports whether the code this signature carries is the one the secret produces.
//
// The stronger scheme wins when both are offered, and the comparison is made in constant time: one
// that gave up at the first differing byte would say, by how long it took, how much of a guess was
// right.
func (s *Signature) Verify(payload []byte, coveredValues []string, subscriptionSecret string) bool {
	code := hmac.New(sha256.New, []byte(subscriptionSecret))
	code.Write([]byte(strconv.FormatInt(s.Timestamp, 10)))
	code.Write([]byte(messageSeparator))

	if s.HeadersCode != nil {
		code.Write([]byte(strings.Join(s.CoveredHeaders, headerNameSeparator)))
		code.Write([]byte(messageSeparator))
		code.Write([]byte(strings.Join(coveredValues, messageSeparator)))
		code.Write([]byte(messageSeparator))
		code.Write(payload)
		return hmac.Equal(code.Sum(nil), s.HeadersCode)
	}

	if s.BodyCode != nil {
		code.Write(payload)
		return hmac.Equal(code.Sum(nil), s.BodyCode)
	}

	// Unreachable: a signature carrying neither code is refused while it is being read.
	return false
}

// signatureTimestamp answers the moment the signature names, which it is not a signature without.
func signatureTimestamp(read map[string]string) (int64, error) {
	written, carried := read[timestampPart]
	if !carried {
		return 0, fmt.Errorf("%w: it carries no `%s` part", ErrSignatureUnreadable, timestampPart)
	}

	seconds, err := strconv.ParseInt(written, 10, 64)
	if err != nil {
		return 0, fmt.Errorf("%w: `%s` is not a number of seconds", ErrSignatureUnreadable, written)
	}
	if seconds > maxTimestampSeconds || seconds < -maxTimestampSeconds {
		return 0, fmt.Errorf(
			"%w: its moment is further than %d seconds from the epoch",
			ErrSignatureUnreadable, maxTimestampSeconds,
		)
	}
	return seconds, nil
}

// signatureCode answers one of the codes a signature offers, decoded whole or not at all.
func signatureCode(read map[string]string, part string) ([]byte, error) {
	written, carried := read[part]
	if !carried {
		return nil, nil
	}

	decoded, err := hex.DecodeString(written)
	if err != nil || len(decoded) == 0 {
		return nil, fmt.Errorf("%w: the `%s` code is not hexadecimal", ErrSignatureUnreadable, part)
	}
	return decoded, nil
}

// coveredHeaders answers the headers the stronger scheme covers, in the order it covers them.
func coveredHeaders(read map[string]string) ([]string, error) {
	written, carried := read[coveredHeadersPart]
	if !carried || written == "" {
		return nil, nil
	}

	names := strings.Split(written, headerNameSeparator)
	if len(names) > maxCoveredHeaders {
		return nil, fmt.Errorf(
			"%w: it covers more than the %d headers accepted",
			ErrSignatureUnreadable, maxCoveredHeaders,
		)
	}

	covered := make([]string, 0, len(names))
	for _, name := range names {
		if name == "" || strings.ContainsFunc(name, isNotHeaderNameCharacter) {
			return nil, fmt.Errorf("%w: `%s` is not a header name", ErrSignatureUnreadable, name)
		}
		covered = append(covered, strings.ToLower(name))
	}
	return covered, nil
}

// isNotHeaderNameCharacter reports whether a character has no place in a header name.
func isNotHeaderNameCharacter(character rune) bool {
	return !strings.ContainsRune(headerNameCharacters, character)
}

// delivered answers the headers of the request under the names a signature refers to them by.
//
// A later value wins over an earlier one under the same name, which is what a map built by the
// caller would have done.
func delivered(headers http.Header) map[string]string {
	carried := make(map[string]string, len(headers))
	for name, values := range headers {
		if len(values) == 0 {
			continue
		}
		carried[strings.ToLower(name)] = values[len(values)-1]
	}
	return carried
}

// VerifyWebhookSignatureAt verifies a webhook against a moment the caller names.
//
//   - signature: the value of the `X-Hook0-Signature` header.
//   - payload: the raw body of the webhook request.
//   - headers: the headers of the webhook request.
//   - subscriptionSecret: the signing secret of the subscription the webhook was delivered for.
//   - tolerance: how far, in either direction, the moment the signature names may sit from
//     currentTime. Five minutes is a reasonable trade-off between tolerating clock drift and
//     bounding how long a captured delivery can be replayed.
//   - currentTime: what to hold the signature's moment against.
//
// Every reason a webhook is refused is one of the sentinels this package declares, so errors.Is
// tells a missing header from a code that does not match from a moment out of the window.
func VerifyWebhookSignatureAt(
	signature string,
	payload []byte,
	headers http.Header,
	subscriptionSecret string,
	tolerance time.Duration,
	currentTime time.Time,
) error {
	parsed, err := ParseSignature(signature)
	if err != nil {
		return err
	}

	carried := delivered(headers)
	coveredValues := make([]string, 0, len(parsed.CoveredHeaders))
	for _, name := range parsed.CoveredHeaders {
		value, found := carried[name]
		if !found {
			return fmt.Errorf("%w: `%s`", ErrHeaderNotDelivered, name)
		}
		coveredValues = append(coveredValues, value)
	}

	if !parsed.Verify(payload, coveredValues, subscriptionSecret) {
		return ErrSignatureMismatch
	}

	drift := currentTime.Sub(time.Unix(parsed.Timestamp, 0))
	if drift < 0 {
		drift = -drift
	}
	if drift > tolerance {
		return fmt.Errorf(
			"%w: it was made %s from now, outside the %s accepted",
			ErrSignatureOutsideTolerance, drift, tolerance,
		)
	}

	return nil
}

// VerifyWebhookSignature verifies a webhook against the current moment.
//
// See VerifyWebhookSignatureAt for what each argument is.
func VerifyWebhookSignature(
	signature string,
	payload []byte,
	headers http.Header,
	subscriptionSecret string,
	tolerance time.Duration,
) error {
	return VerifyWebhookSignatureAt(signature, payload, headers, subscriptionSecret, tolerance, time.Now())
}
