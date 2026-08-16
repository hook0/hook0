// The rest of the file, for every Go example of the SDK reference.
//
// A snippet on a page is written for a reader: it leaves out the imports, it assumes a client is
// already built, and it names an application id and a token without saying where they came from.
// Each region below is the file that snippet would live in, with a hole where it goes. The page
// points at one by name on the fence, so what a snippet is standing on is one word away from the
// snippet itself.
//
// This file never compiles as it stands. Each region becomes a package of its own, under a
// directory named after the example, which is why every region opens with its own `package`.

// HARNESS import
package {{name}}

EXAMPLE

// What the import above is for. Go refuses a file that imports something it never reaches, and a
// page showing an import line alone is showing exactly that.
var _ = hook0.DefaultOptions

// END HARNESS

// HARNESS program
EXAMPLE

// What the program above was handed before it started.
var (
	applicationId = "0d0ea1e0-8b1f-4f4d-9c2a-1b5c9a3d7e21"
	token         = "a-service-token"
)

// END HARNESS

// HARNESS event
package {{name}}

import (
	"time"

	hook0 "github.com/hook0/hook0/clients/go"
)

// The value the page shows, held so that every field of it is checked against the client.
var event =
	EXAMPLE

// END HARNESS

// HARNESS options
package {{name}}

import (
	"time"

	hook0 "github.com/hook0/hook0/clients/go"
)

var (
	apiURL        = "https://app.hook0.com/api/v1"
	applicationId = "0d0ea1e0-8b1f-4f4d-9c2a-1b5c9a3d7e21"
	token         = "a-service-token"
)

// The client the page configures, returned so that Go does not refuse a file for building one
// nothing uses.
func configured() *hook0.Client {
	EXAMPLE

	return client
}

// END HARNESS

// HARNESS handler
package {{name}}

EXAMPLE

// Where the handler above reads the secret of the subscription it serves.
var subscriptionSecret = "a-subscription-secret"

// END HARNESS

// HARNESS matching
package {{name}}

import (
	"errors"
	"log"

	hook0 "github.com/hook0/hook0/clients/go"
)

// Both of these are reached by some of the snippets that drop in below and by none of the others,
// and Go refuses a file carrying an import nothing uses.
var (
	_ = errors.Is
	_ = log.Printf
	_ = hook0.ErrSignatureMismatch
)

// Whatever a send or a verification answered, taken apart the way the page shows.
func inspect(err error) {
	EXAMPLE
}

// END HARNESS

// HARNESS upsert
package {{name}}

import (
	"context"

	hook0 "github.com/hook0/hook0/clients/go"
)

// The event types an application declares before it sends anything of them.
func declare(ctx context.Context, client *hook0.Client) ([]string, error) {
	EXAMPLE

	return created, err
}

// END HARNESS

// HARNESS file
package {{name}}

EXAMPLE

// What the file above was handed before it started.
var token = "a-service-token"

// END HARNESS
