#!/usr/bin/env bash

# install dependencies
npm install

# copy mediakit
cp -v ../mediakit/logo/512x512.png ./src/main/resources/theme/hook0-redesign/email/img/logo.png
cp -v ../mediakit/logo/512x512.png ./src/main/resources/theme/hook0-redesign/login/resources/img/logo.png
cp -v ../mediakit/logo/favicon.ico ./src/main/resources/theme/hook0-redesign/login/resources/img/

# compile theme
npm run build



# package theme and wrap it in a .jar file
mvn package

# move theme to /tmp/theme/ folder for futur use by keycloak
mkdir -p /tmp/theme && cp -v target/*.jar /tmp/theme/
