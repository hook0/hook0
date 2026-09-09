module hook0.smoke/go

go 1.25.13

require github.com/hook0/hook0-go/v3 v3.0.0

// What is exercised is the client in this repository, not whatever the proxy answers with.
replace github.com/hook0/hook0-go/v3 => ../../../clients/go
