#!/usr/bin/env bash
rm -rf ./postgres && mkdir -p ./postgres && docker compose up --force-recreate
