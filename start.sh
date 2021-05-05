#!/usr/bin/env bash

docker-compose -f ./database/docker-compose.yml up &
RUST_LOG=debug cargo watch -x "run --bin hook0-api" &
cd ./frontend && npm run serve
