#!/usr/bin/env bash
# dependency: npm i nodemon -g

nodemon -w src/ -e "css,properties,json,js,ftl,scss" --exec "bash ./build-and-restart-keycloak.sh"
