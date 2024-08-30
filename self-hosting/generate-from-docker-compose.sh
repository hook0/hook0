#!/usr/bin/env bash
kompose --provider=kubernetes -f ../../docker-compose.yaml convert -o deployments.yaml --with-kompose-annotation=false --namespace=hook0
kompose --provider kubernetes --file ../../docker-compose.yaml convert --with-kompose-annotation=false --namespace=hook0 --chart --out .
