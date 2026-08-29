// What the dashboard shows under "Verify a webhook", for Go.
//
// Sending is only half of what a reader has come to do, and it is the easier half. This is the one
// the SDK reference calls the half most often got wrong by hand, so the dashboard shows it beside
// the send rather than leaving it to be found later.
//
// The secret is read from the environment on purpose. The dashboard cannot know which subscription
// a reader means — outside the onboarding it loads none, and an application may have several — so it
// points at the subscription instead of guessing one, and no second secret is put on screen.
//
// Read the markers as in `dashboard_send.go`: `hook0:snippet` is what is displayed, everything
// outside it is what makes the file compile. The package clause is outside it because this file
// shares a package with the send example, and because a handler is dropped into a reader's own
// program rather than pasted as a whole one.

package main

// hook0:snippet:begin
import (
	"io"
	"net/http"
	"os"
	"time"

	hook0 "github.com/hook0/hook0-go/v2"
)

// Verify against the *raw* body: one that has been parsed and serialised again no longer hashes to
// what was signed, which is why the bytes are read here rather than left to a decoder. The tolerance
// is bilateral, so a delivery dated too far ahead is refused exactly like one dated too far behind.
func handleWebhook(w http.ResponseWriter, r *http.Request) {
	body, err := io.ReadAll(http.MaxBytesReader(w, r.Body, 1024*1024))
	if err != nil {
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	// The secret of the subscription being verified, which the dashboard links to rather than
	// prints. A variable nobody exported and one exported empty are the same defect and are
	// answered together: an empty secret hashes every genuine delivery to the wrong code, so the
	// handler would answer every one of them the way it answers a forgery and nothing would say
	// why. Told apart from a forgery here instead.
	secret := os.Getenv("HOOK0_SUBSCRIPTION_SECRET")
	if secret == "" {
		http.Error(w, "HOOK0_SUBSCRIPTION_SECRET is not set", http.StatusInternalServerError)
		return
	}

	err = hook0.VerifyWebhookSignature(
		r.Header.Get("X-Hook0-Signature"),
		body,
		r.Header,
		secret,
		5*time.Minute,
	)
	if err != nil {
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	// act on the delivery
	w.WriteHeader(http.StatusOK)
}

// hook0:snippet:end
