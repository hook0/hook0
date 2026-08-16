module hook0.smoke/go

go 1.24

require github.com/hook0/hook0-go v0.0.0

// What is exercised is the client in this repository, not whatever the proxy answers with.
replace github.com/hook0/hook0-go => ../../../clients/go
