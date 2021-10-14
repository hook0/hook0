# Hook0 — Webhook as a Service

Welcome to Hook0! Sign up to [hook0.com](https://www.hook0.com/) and start opening your SaaS to the web!

[[_TOC_]]

# Problems Hook.io solve

<details>
<summary markdown="span">Fine-grained subscriptions</summary>
Enable your users to subscribe to your events by setting up a webhook. They can choose which event types they want to receive.
</details>
<details>
<summary markdown="span">Multi subscriptions</summary>
Your users can register several webhooks, we will send events to all of them!
</details>
<details>
<summary markdown="span">Event scoping</summary>
Scope events to one or several levels of your application. Users, organizations, administrators, [insert your own], they can all handle subscriptions to their events.
</details>
<details>
<summary markdown="span">Dashboards</summary>
Either use Hook0 out-of-the-box dashboards to let your users see events that went through their subscriptions, or build your own with the API.
</details>
<details>
<summary markdown="span">Auto-Retry</summary>
If Hook0 can't reach a webhook, or if it does not respond with a success code, Hook0 will try again automatically.
</details>
<details>
<summary markdown="span">Failure notification</summary>
If after several retries Hook0 still can't successfuly reach a webhook, your user is notified by email.
</details>
<details>
<summary markdown="span">Events & responses persistence</summary>
Hook0 can keep track of every event your application sent it and of every webhook call. This can helps you debug things or act as an audit log !
</details>
<details>
<summary markdown="span">High availability</summary>
Hook0 won't miss the events you send it.
</details>
<details>
<summary markdown="span">GDPR Compliance</summary>
Hook0 is fully GDPR compliant and can easily execute a data processor agreement with your company if needed.
</details>
<details>
<summary markdown="span">Data Security</summary>
Hook0 utilizes best practices for data storage and encryption. We also offer single-tenant and on-premise deployment options.
</details>
<details>
<summary markdown="span">Designed for Enteprise Scale</summary>
Hook0 robust architecture automatically scales to handle thousands of requests per minute.
</details>
<details>
<summary markdown="span">Open-Source</summary>
No vendor-locking. Open to anyone. Fork it, twist it. Flip it. Join the community and help us build the best open source webhook server for applications. We're committed to give developers, small businesses and enterprises the power to connect with any web services.
</details>

# Getting Started

## Sign Up

First step, contact [Hook0 support](mailto:support@hook0.com) to receive your beta account.

At this point you will have:
- An [organization](#organizations) id
- A client id and client secret that can be used to make API calls for your [organization](#organizations)
- A username/password for your personal user, that has editor rights to your [organization](#organizations)
- A username/password for a “service” user, that has editor rights to your [organization](#organizations)


## API Authentication

Every call to Hook0’s API will require an [access token](#access-tokens). You can obtain one using the **client id** and **client secret** of your [organization](#organizations) and the credentials of one of its users (either a personal account or a service account).

The following code examples requires [jq](https://stedolan.github.io/jq/) and [curl](https://curl.se/). First step authenticate (change `USER_NAME`, `USER_PASSWORD`, `ORGANIZATION_ID`, `CLIENT_ID` and `CLIENT_SECRET` with the information Hook0 support sent you):


```shell
ACCESS_TOKEN=$(curl -v https://hook0-production.cloud-iam.com/auth/realms/production/protocol/openid-connect/token \
 -d "grant_type=password&client_id=${CLIENT_ID}&client_secret=${CLIENT_SECRET}&username=${USER_NAME}&password=${USER_PASSWORD}" | jq -r .access_token)

echo $ACCESS_TOKEN
```

Save the `ACCESS_TOKEN` you will need it for the next API calls.
{: .note}

## Create application

An application represents a SaaS. Let's call it `mycrm`.

In order to keep everything sorted, your user **account** can be linked to several [applications](#applications) inside a single [organization](#organizations).

The first thing you need to do is create your first application. Everything (webhook subscriptions and triggering) will then happen inside this application.


```shell
APPLICATION_NAME=mycrm

APPLICATION_ID=$(curl -v -H 'Content-type: application/json' \
-H "Authorization: Bearer ${ACCESS_TOKEN}" \
-d "{\"name\":\"${APPLICATION_NAME}\",\"organization_id\":\"${ORGANIZATION_ID}\"}" https://app.hook0.com/api/v1/applications | jq -r '.application_id')
```

Save the `APPLICATION_ID` you will need it for the next API calls.
{: .note}


## Create application secret

[Application secrets](#application-secrets) are another way of authenticating API calls. They differ from [access tokens](#access-tokens) in two ways:
1. [Application secrets](#application-secrets) are restricted to one [application](#applications) only
2. [Application secrets](#application-secrets) are simpler to use (only HTTP Basic Auth, no time-based expiration)

Let's create an [application secrets](#application-secrets) that we will name `mycrm-production-secret` for our `mycrm` [application](#applications).

```shell
SECRET_NAME=mycrm-production-secret

APPLICATION_SECRET=$(curl -v -H 'Content-type: application/json' \
-H "Authorization: Bearer ${ACCESS_TOKEN}" \
-d "{\"application_id\":\"${APPLICATION_ID}\",\"name\":\"${SECRET_NAME}\"}" https://app.hook0.com/api/v1/application_secrets  | jq -r '.token')

echo $APPLICATION_SECRET
```

## Create event types

Hook0 needs to know which kind of **events** your [application](#applications) might emit.

This will allow your users to choose which ones they want to subscribe to. Event types must follow this pattern: `service.resource_type.verb`.

Let’s create the event type `mycrm.customer.created`

```shell
SERVICE=mycrm
RESOURCE_TYPE=customer
VERB=created
curl -v -H 'Content-type: application/json' \
 -H "Authorization: Bearer ${ACCESS_TOKEN}" \
 -d "{\"application_id\":\"${APPLICATION_ID}\",\
   \"service\":\"${SERVICE}\",\
  \"resource_type\":\"${RESOURCE_TYPE}\", \
  \"verb\":\"${VERB}\"\
}" https://app.hook0.com/api/v1/event_types
```

And the event type `mycrm.deal.created`

```shell
SERVICE=mycrm
RESOURCE_TYPE=deal
VERB=created
curl -v -H 'Content-type: application/json' \
 -H "Authorization: Bearer ${ACCESS_TOKEN}" \
 -d "{\"application_id\":\"${APPLICATION_ID}\",\
   \"service\":\"${SERVICE}\", \
  \"resource_type\":\"${RESOURCE_TYPE}\", \
  \"verb\":\"${VERB}\" \
}" https://app.hook0.com/api/v1/event_types
```

## Create the first webhook subscription

Now that everything is in place, let’s set up a [subscription](#subscriptions) to one of our [event types](#event-types). This means that every time our [application](#applications) emits an event of this type, Hook0 will send an HTTP request to the URL you specified. In a normal workflow, this [subscription](#subscriptions) could be either created by you, an app developer, or by one of your users.

For this tutorial, we will use the [webhook.site](https://webhook.site) web service to simulate one of your customer webhook endpoint.

The HTTP call below create a webhook [subscription](#subscriptions) only for the `mycrm.customer.created` event. The webhook target will be called with the `HTTP` protocol and the HTTP `POST` verb.

```shell
WEBHOOK_ENDPOINT=https://webhook.site/313af2e0-de7c-4c7e-8e88-f3258b168d43

curl -v -H 'Content-type: application/json' \
-H "Authorization: Bearer ${ACCESS_TOKEN}" \
-d '{"application_id": "'${APPLICATION_ID}'", "description": "Each time mycrm emits a mycrm.customer.created event, this webhook will be called!!", "event_types": ["mycrm.customer.created"], "is_enabled": true, "metadata": {}, "label_key": "", "label_value": "", "target": {"type": "http", "method": "POST", "headers":{}, "url": "'${WEBHOOK_ENDPOINT}'"}}' https://app.hook0.com/api/v1/subscriptions
```

Running the code above, the console prints something like:

```json
{
  "subscription_id": "4b5c8bce-38cd-495d-8656-9d1e933cd1ac",
  ...
  "event_types": [
    "mycrm.customer.created"
  ],
  ...
  "secret": "02ddf3b2-cc43-4806-baac-6f9f771baca1",
  ...
  "created_at": "2021-09-10T14:06:23.601390Z"
}
```

A [subscription](#subscriptions) has been created with ID "4b5c8bce-38cd-495d-8656-9d1e933cd1ac".
It has a [subscription secret](#subscription-secrets) attribute for signing http request with HMAC.

> A Hashed Message Authentication Code (HMAC) is a cryptographic artifact for determining the authenticity and integrity of a message object, the usage of a symmetric key and a hash (message-digest). The HMAC might be founded on message-digest calculations along with the SHA256, MD5 etc. Ownership of an HMAC esteem does now not bargain the delicate realities as HMACs aren't reversible curios. This tool can be used as hmac sha256 online.

## Trigger the webhook

At this point, we have created an [application](#applications) (`mycrm`) in Hook0, declared which event types this application could emit (`mycrm.customer.created`, `mycrm.deal.created`) and one of our users has created a subscription (`4b5c8bce-38cd-495d...`). Now we need to actually emit some events from our application (`mycrm`) and see what happens!

When sending events from your application to Hook0, authentication is done using one of the [application secrets](#application-secrets) you created earlier.

```
# A sample event that mycrm app could generate
EVENT_ID=deaf709e-d765-4468-8e29-b77ddf9975ae
EVENT_TYPE=mycrm.customer.created
PAYLOAD=$(echo '{"customer_id": 10, "created_at": "1631284047401"}' | jq "@base64")
PAYLOAD_CONTENT_TYPE="application/json"
OCCURRED_AT="2021-09-10T16:24:52+02:00"

curl -v -H 'Content-type: application/json' \
-d '{"application_id": "'${APPLICATION_ID}'", "event_id": "'${EVENT_ID}'", "event_type": "'${EVENT_TYPE}'", "payload": '${PAYLOAD}', "payload_content_type": "'${PAYLOAD_CONTENT_TYPE}'", "occurred_at": "'${OCCURRED_AT}'", "application_secret": "'${APPLICATION_SECRET}'", "labels": {"": ""}}' https://app.hook0.com/api/v1/event
```

Hook0 API replies with a 201 CREATED HTTP status code and the following body:

```json
{
  "application_id": "0dc1d206-7f0d-4c9c-9582-ed66d00b12ce",
  "event_id": "deaf709e-d765-4468-8e29-b77ddf9975ae",
  "received_at": "2021-09-10T14:46:10.103942Z"
}
```


# Architecture

There are three components to Hook0:

- API: This is a Rust HTTP service that contains the core logic for webhooks persistence management.
- Frontend: Responsible for rendering the UI, rely entirely on Hook0 REST API.
- Output-worker: Each time Hook0 receive an [event](#events) the output-worker will call the corresponding [subscription targets](#subscriptions-targets).

# Concepts

## Overview

Get a high-level outline of Hook0 and the components it is built from.

The core of Hook0 control plane is the API server. The API server exposes an HTTP REST API that lets end [users](#user-accounts) and [applications](#applications) manage webhooks.

The Hook0 API lets you query and manipulate the state of API objects in Hook0 (for example: [Applications](#applications), [Events](#events), [Subscriptions](#subscriptions)).

Most operations can be performed through the kubectl command-line interface or other command-line tools, such as kubeadm, which in turn use the API. However, you can also access the API directly using REST calls.

## Architecture
The architectural concepts behind Hook0.

<!-- @todo -->

## Organizations

## User Accounts

## Access Tokens

## Applications

## Application Secrets

## Events

## Event Types

## Subscriptions

## Subscription Secrets

## Subscriptions Targets

Define where to send the [events](#events) to, it can be an HTTP endpoint or something else.

## Webhook

# Reference

## API Reference

Complete API Reference is available from Hook0 dashboard: https://app.hook0.com/api/documentation

## CLI

`hook0ctl` —  Main CLI tool for commands and sending [events](#events) and managing Hook0 [applications](#applications).

# License
Hook0 is free and the source is available. Versions are published under the [Server Side Public License (SSPL) v1](./LICENSE.txt).

The license allows the free right to use, modify, create derivative works, and redistribute, with three simple limitations:

- You may not provide the products to others as a managed service
- You may not circumvent the license key functionality or remove/obscure features protected by license keys
- You may not remove or obscure any licensing, copyright, or other notices

# Find us

- [Website](https://www.hook0.com/)
- [Updates on Twitter](https://twitter.com/hook0_)
