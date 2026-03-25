## `hook0 init`

Set up your first profile

**Usage:** `hook0 init [OPTIONS]`

###### **Options:**

* `--non-interactive` — Skip interactive prompts and use defaults
* `--event-type <EVENT_TYPE>` — Event type to create



## `hook0 login`

Authenticate with an Application Secret

**Usage:** `hook0 login [OPTIONS]`

###### **Options:**

* `--secret <SECRET>` — Application Secret (UUID token)
* `--api-url <API_URL>` — API URL

  Default value: `https://app.hook0.com/api/v1`
* `-n`, `--profile-name <PROFILE_NAME>` — Profile name to save credentials

  Default value: `default`
* `--application-id <APPLICATION_ID>` — Application ID (required - the Application Secret is tied to this application)



## `hook0 logout`

Remove stored credentials

**Usage:** `hook0 logout [OPTIONS]`

###### **Options:**

* `-n`, `--profile-name <PROFILE_NAME>` — Profile name to remove credentials for
* `--all` — Remove all stored credentials



## `hook0 whoami`

Display current application and profile

**Usage:** `hook0 whoami`



## `hook0 listen`

Receive webhooks locally via tunnel

**Usage:** `hook0 listen [OPTIONS] [TARGET]`

###### **Arguments:**

* `<TARGET>` — Local URL or port to forward webhooks to (auto-detects if not specified) Examples: 3000, http://localhost:3000/webhooks

###### **Options:**

* `--relay-url <RELAY_URL>` — Hooks relay server URL (WebSocket endpoint)

  Default value: `wss://play.hook0.com/ws`
* `--token <TOKEN>` — Token to use (if not provided, a new one will be generated)
* `--ping-interval <PING_INTERVAL>` — Ping interval in seconds

  Default value: `30`
* `--insecure` — Disable TLS certificate verification (for self-signed certs)
* `--allow-external` — Allow forwarding to non-localhost targets (external URLs)
* `--no-tui` — Disable full-screen TUI mode (use plain log output instead)



## `hook0 example`

Send a sample webhook to test your setup

**Usage:** `hook0 example [OPTIONS]`

###### **Options:**

* `--target <TARGET>` — Target URL to forward webhooks to (default: built-in echo server)
* `--relay-url <RELAY_URL>` — Relay server WebSocket URL

  Default value: `wss://play.hook0.com/ws`
* `--token <TOKEN>` — Token for the webhook URL (auto-generated if not specified)
* `--ping-interval <PING_INTERVAL>` — Ping interval in seconds

  Default value: `30`
* `--insecure` — Allow insecure TLS connections



## `hook0 event`

Manage webhook events

**Usage:** `hook0 event <COMMAND>`

###### **Subcommands:**

* `send` — Send a new event
* `list` — List events
* `get` — Get event details



## `hook0 event send`

Send a new event

**Usage:** `hook0 event send [OPTIONS] --label <LABEL> <EVENT_TYPE>`

###### **Arguments:**

* `<EVENT_TYPE>` — Event type (e.g., user.account.created)

###### **Options:**

* `-d`, `--payload <PAYLOAD>` — JSON payload
* `-f`, `--payload-file <PAYLOAD_FILE>` — Read payload from file
* `-l`, `--label <LABEL>` — Labels in key=value format (required, can be repeated)
* `--event-id <EVENT_ID>` — Custom event ID (UUID, auto-generated if not provided)
* `--content-type <CONTENT_TYPE>` — Content type (default: application/json)

  Default value: `application/json`



## `hook0 event list`

List events

**Usage:** `hook0 event list [OPTIONS]`

###### **Options:**

* `--event-type <EVENT_TYPE>` — Filter by event type
* `--status <STATUS>` — Filter by status (waiting, pending, in_progress, successful, failed)
* `--since <SINCE>` — Filter events since (e.g., 1h, 24h, 7d)
* `--until <UNTIL>` — Filter events until
* `-l`, `--label <LABEL>` — Filter by label (key=value, can be repeated)
* `--limit <LIMIT>` — Maximum number of events to return

  Default value: `50`
* `--page <PAGE>` — Page number

  Default value: `1`



## `hook0 event get`

Get event details

**Usage:** `hook0 event get [OPTIONS] <EVENT_ID>`

###### **Arguments:**

* `<EVENT_ID>` — Event ID

###### **Options:**

* `--attempts` — Show request attempts for this event



## `hook0 event-type`

Manage event types

**Usage:** `hook0 event-type <COMMAND>`

###### **Subcommands:**

* `create` — Create a new event type
* `list` — List event types
* `delete` — Delete an event type



## `hook0 event-type create`

Create a new event type

**Usage:** `hook0 event-type create [OPTIONS] [NAME]`

###### **Arguments:**

* `<NAME>` — Event type name (e.g., user.account.created) or individual components

###### **Options:**

* `-s`, `--service <SERVICE>` — Service name (alternative to full name)
* `-r`, `--resource <RESOURCE>` — Resource type name (alternative to full name)
* `-b`, `--verb <VERB>` — Verb name (alternative to full name)



## `hook0 event-type list`

List event types

**Usage:** `hook0 event-type list [OPTIONS]`

###### **Options:**

* `--service <SERVICE>` — Filter by service name



## `hook0 event-type delete`

Delete an event type

**Usage:** `hook0 event-type delete [OPTIONS] <NAME>`

###### **Arguments:**

* `<NAME>` — Event type name (e.g., user.account.created)

###### **Options:**

* `-y`, `--yes` — Skip confirmation prompt



## `hook0 subscription`

Manage subscriptions

**Usage:** `hook0 subscription <COMMAND>`

###### **Subcommands:**

* `create` — Create a new subscription
* `list` — List subscriptions
* `get` — Get subscription details
* `update` — Update a subscription
* `delete` — Delete a subscription
* `enable` — Enable a subscription
* `disable` — Disable a subscription



## `hook0 subscription create`

Create a new subscription

**Usage:** `hook0 subscription create [OPTIONS] --url <URL> --events <EVENTS> --label <LABEL>`

###### **Options:**

* `-u`, `--url <URL>` — Webhook endpoint URL
* `-e`, `--events <EVENTS>` — Event types to subscribe to (required, comma-separated or repeated)
* `-l`, `--label <LABEL>` — Labels in key=value format (required, can be repeated)
* `--method <METHOD>` — HTTP method (default: POST)

  Default value: `POST`
* `-H`, `--header <HEADER>` — Custom headers in key=value format (can be repeated)
* `-d`, `--description <DESCRIPTION>` — Description
* `--disabled` — Create disabled



## `hook0 subscription list`

List subscriptions

**Usage:** `hook0 subscription list [OPTIONS]`

###### **Options:**

* `-l`, `--label <LABEL>` — Filter by label (key=value, can be repeated)
* `--enabled` — Show only enabled subscriptions
* `--disabled` — Show only disabled subscriptions



## `hook0 subscription get`

Get subscription details

**Usage:** `hook0 subscription get <SUBSCRIPTION_ID>`

###### **Arguments:**

* `<SUBSCRIPTION_ID>` — Subscription ID



## `hook0 subscription update`

Update a subscription

**Usage:** `hook0 subscription update [OPTIONS] <SUBSCRIPTION_ID>`

###### **Arguments:**

* `<SUBSCRIPTION_ID>` — Subscription ID

###### **Options:**

* `-u`, `--url <URL>` — Webhook endpoint URL
* `-e`, `--events <EVENTS>` — Event types to subscribe to (replaces existing)
* `-l`, `--label <LABEL>` — Labels in key=value format (replaces existing)
* `--method <METHOD>` — HTTP method
* `-H`, `--header <HEADER>` — Custom headers (replaces existing)
* `-d`, `--description <DESCRIPTION>` — Description
* `--enable` — Enable the subscription
* `--disable` — Disable the subscription



## `hook0 subscription delete`

Delete a subscription

**Usage:** `hook0 subscription delete [OPTIONS] <SUBSCRIPTION_ID>`

###### **Arguments:**

* `<SUBSCRIPTION_ID>` — Subscription ID

###### **Options:**

* `-y`, `--yes` — Skip confirmation prompt



## `hook0 subscription enable`

Enable a subscription

**Usage:** `hook0 subscription enable <SUBSCRIPTION_ID>`

###### **Arguments:**

* `<SUBSCRIPTION_ID>` — Subscription ID



## `hook0 subscription disable`

Disable a subscription

**Usage:** `hook0 subscription disable <SUBSCRIPTION_ID>`

###### **Arguments:**

* `<SUBSCRIPTION_ID>` — Subscription ID



## `hook0 application`

Manage applications

**Usage:** `hook0 application <COMMAND>`

###### **Subcommands:**

* `list` — List applications
* `get` — Get application details
* `switch` — Switch to a different application
* `current` — Show current application



## `hook0 application list`

List applications

**Usage:** `hook0 application list [OPTIONS]`

###### **Options:**

* `--organization-id <ORGANIZATION_ID>` — Organization ID (uses default if not specified)



## `hook0 application get`

Get application details

**Usage:** `hook0 application get [APPLICATION_ID]`

###### **Arguments:**

* `<APPLICATION_ID>` — Application ID (uses default if not specified)



## `hook0 application switch`

Switch to a different application

**Usage:** `hook0 application switch <APPLICATION_ID>`

###### **Arguments:**

* `<APPLICATION_ID>` — Application ID to switch to



## `hook0 application current`

Show current application

**Usage:** `hook0 application current`



## `hook0 replay`

Replay failed events

**Usage:** `hook0 replay [OPTIONS] [EVENT_ID]`

###### **Arguments:**

* `<EVENT_ID>` — Event ID to replay

###### **Options:**

* `--all` — Replay all events matching criteria (requires --confirm)
* `--status <STATUS>` — Filter by status (failed, successful, etc.)
* `--since <SINCE>` — Filter events since (e.g., 1h, 24h, 7d)
* `--until <UNTIL>` — Filter events until (e.g., 1h, 24h, 7d)
* `--event-type <EVENT_TYPE>` — Filter by event type
* `--dry-run` — Dry run - show what would be replayed without actually replaying
* `--confirm` — Confirm bulk replay operation
* `--limit <LIMIT>` — Maximum number of events to replay

  Default value: `100`



## `hook0 config`

Manage configuration and profiles

**Usage:** `hook0 config <COMMAND>`

###### **Subcommands:**

* `list` — List all profiles
* `show` — Show current configuration
* `set-default` — Set default profile
* `remove` — Remove a profile
* `path` — Show configuration file path



## `hook0 config list`

List all profiles

**Usage:** `hook0 config list`



## `hook0 config show`

Show current configuration

**Usage:** `hook0 config show`



## `hook0 config set-default`

Set default profile

**Usage:** `hook0 config set-default <PROFILE>`

###### **Arguments:**

* `<PROFILE>` — Profile name to set as default



## `hook0 config remove`

Remove a profile

**Usage:** `hook0 config remove [OPTIONS] <PROFILE>`

###### **Arguments:**

* `<PROFILE>` — Profile name to remove

###### **Options:**

* `-y`, `--yes` — Skip confirmation



## `hook0 config path`

Show configuration file path

**Usage:** `hook0 config path`



## `hook0 completion`

Generate shell completion scripts

**Usage:** `hook0 completion <SHELL>`

###### **Arguments:**

* `<SHELL>` — Shell to generate completions for

  Possible values: `bash`, `elvish`, `fish`, `powershell`, `zsh`