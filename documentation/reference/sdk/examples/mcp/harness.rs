// The rest of the file, for every MCP example of the SDK reference.
//
// A snippet on the page is written for a reader: it leaves out the imports, it assumes a token is
// already in the environment, and it stops before the boilerplate. Each region below is the file
// that snippet would live in, with a hole where it goes. The page points at one by name on the
// fence, so what a snippet is standing on is one word away from the snippet itself.
//
// Every region becomes its own file under `src/bin`, which is why every region that is not already
// a complete program supplies its own `fn main() {}`: a Rust binary refuses to build without one,
// and none of these functions is ever meant to run.

// HARNESS serve
EXAMPLE
// END HARNESS

// HARNESS tools
fn main() {
    EXAMPLE
}
// END HARNESS

// HARNESS dispatch
use std::collections::HashMap;

fn main() {}

// The name a client asked for, and the arguments it filled the schema in with.
fn asked(name: &str, arguments: &HashMap<String, String>) {
    EXAMPLE
}
// END HARNESS
