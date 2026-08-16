// The module the Go examples of the SDK reference are built as.
//
// It reaches the client through a path rather than a version on purpose: what the pages must agree
// with is the client in this commit, not the one that happened to be published last.
module hook0.example/documentation

go 1.24

require github.com/hook0/hook0-go v0.0.0

replace github.com/hook0/hook0-go => {{client}}
