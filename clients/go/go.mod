// The module is reached by URL, so the path is the address it is fetched at rather than a name of
// its own. That address is the GitHub mirror this directory is pushed to, whose root is the module
// — so the tag publishing a version is a plain `vX.Y.Z` on the mirror, cut from the `sdk-vX.Y.Z`
// the other clients are released under. `ci/release-packages` refuses a release where this path and
// that mirror have come apart.
module github.com/hook0/hook0-go

// Sending an event, verifying a signature and reading what the API answers are all done with the
// standard library, so installing this can never drag a transitive dependency into an application
// that only wanted to send an event. Nothing below is a `require`, and nothing ever should be.
go 1.24
