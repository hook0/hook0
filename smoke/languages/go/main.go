// The Go client against a Hook0 that is really running.
//
// Three things the loopback suite cannot ask: whether an application secret the API minted is
// accepted, whether a second send under an identifier already ingested is reported as the conflict
// it is, and whether a signature the output worker computed verifies. Everything else about this
// client is settled by the suite beside the client.
package main

import (
	"context"
	"fmt"
	"net/http"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"time"

	hook0 "github.com/hook0/hook0/clients/go"
)

// How long the two sends together are given.
const sendingWithin = 2 * time.Minute

func main() {
	if err := smoke(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}

func smoke() error {
	apiURL, err := setting("HOOK0_API_URL")
	if err != nil {
		return err
	}
	applicationID, err := setting("HOOK0_APPLICATION_ID")
	if err != nil {
		return err
	}
	token, err := setting("HOOK0_TOKEN")
	if err != nil {
		return err
	}
	eventType, err := setting("HOOK0_EVENT_TYPE")
	if err != nil {
		return err
	}
	delivery, err := setting("HOOK0_DELIVERY")
	if err != nil {
		return err
	}

	client := hook0.NewClient(apiURL, applicationID, token, hook0.DefaultOptions())
	ctx, done := context.WithTimeout(context.Background(), sendingWithin)
	defer done()

	sent, err := client.SendEvent(ctx, event(eventType, ""))
	if err != nil {
		return fmt.Errorf("the instance refused the first send: %w", err)
	}
	fmt.Printf("ingested %s\n", sent)

	_, err = client.SendEvent(ctx, event(eventType, sent))
	if err == nil {
		return fmt.Errorf("sending the same event twice was accepted twice")
	}
	if !strings.Contains(err.Error(), hook0.AlreadyIngested) {
		return fmt.Errorf("the second send failed without naming %s: %w", hook0.AlreadyIngested, err)
	}
	fmt.Printf("the second send reported %s\n", hook0.AlreadyIngested)

	if err := verify(delivery); err != nil {
		return err
	}
	fmt.Println("the signature the instance produced verifies")
	return nil
}

// The event both sends carry, under the identifier the caller names.
func event(eventType string, eventID string) hook0.Event {
	return hook0.Event{
		EventType:          eventType,
		Payload:            `{"from":"the go smoke"}`,
		PayloadContentType: "application/json",
		Labels:             map[string]string{"language": "go"},
		EventId:            eventID,
	}
}

// Verifies what the output worker really delivered, with this client's own verification.
func verify(delivery string) error {
	read := func(part string) (string, error) {
		what, err := os.ReadFile(filepath.Join(delivery, part))
		if err != nil {
			return "", fmt.Errorf("reading the delivered %s: %w", part, err)
		}
		return string(what), nil
	}

	signature, err := read("signature")
	if err != nil {
		return err
	}
	secret, err := read("secret")
	if err != nil {
		return err
	}
	written, err := read("tolerance")
	if err != nil {
		return err
	}
	tolerance, err := strconv.Atoi(strings.TrimSpace(written))
	if err != nil {
		return fmt.Errorf("the tolerance is not a number of seconds: %w", err)
	}
	body, err := os.ReadFile(filepath.Join(delivery, "body"))
	if err != nil {
		return fmt.Errorf("reading the delivered body: %w", err)
	}
	lines, err := read("headers")
	if err != nil {
		return err
	}

	headers := http.Header{}
	for _, line := range strings.Split(lines, "\n") {
		name, value, found := strings.Cut(line, ": ")
		if found {
			headers.Add(name, value)
		}
	}

	if err := hook0.VerifyWebhookSignature(
		strings.TrimSpace(signature),
		body,
		headers,
		strings.TrimSpace(secret),
		time.Duration(tolerance)*time.Second,
	); err != nil {
		return fmt.Errorf("the signature the instance produced was refused: %w", err)
	}
	return nil
}

// A setting the harness passes, or a refusal naming it: a smoke that ran without one would report a
// failure of the client for something the harness never handed it.
func setting(name string) (string, error) {
	value := os.Getenv(name)
	if value == "" {
		return "", fmt.Errorf("%s is not set", name)
	}
	return value, nil
}
