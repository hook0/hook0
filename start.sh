#!/usr/bin/env bash

(cd ./database && ./start-or-restart.sh) &
RUST_LOG=debug cargo watch -x "run --bin hook0-api" &
cd ./frontend && npm run serve
