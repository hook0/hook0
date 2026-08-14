// The module is reached by URL, so the path is the address the mirror answers at rather than a
// name of its own. A module in a subdirectory is published by a tag carrying that subdirectory —
// `clients/go/vX.Y.Z` — which is not the `sdk-vX.Y.Z` the other clients are released under.
module github.com/hook0/hook0/clients/go

// Sending an event, verifying a signature and reading what the API answers are all done with the
// standard library, so installing this can never drag a transitive dependency into an application
// that only wanted to send an event. Nothing below is a `require`, and nothing ever should be.
go 1.24
