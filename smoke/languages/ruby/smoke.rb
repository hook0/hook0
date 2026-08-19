# frozen_string_literal: true

# The Ruby client against a Hook0 that is really running.
#
# Two things happen here, and the second is the reason the first is worth having.
#
# The control: whether an application secret the API minted is accepted, whether a second send
# under an identifier already ingested is reported as the conflict it is, and whether a signature
# the output worker computed verifies. Those are the three questions no loopback suite can ask
# itself, because a suite that signs and verifies with the same sources only proves the sources
# agree with themselves.
#
# The surface: every operation the API document declares, driven through the generated layer
# against the same instance, and every model type it decodes out of a real answer.
# `clients/ruby/test` already drives all of them — against an API the suite itself writes, out of
# the same document the client was generated from. That proves the client matches the document. It
# cannot prove the document matches Hook0, and a field the API really answers under another name
# passes there and fails on a consumer's first call.

$LOAD_PATH.unshift(File.expand_path("../../../clients/ruby/lib", __dir__))

require "uri"
require "hook0"

# The conflict the API answers a duplicated ingestion with.
ALREADY_INGESTED = "EventAlreadyIngested"

# What this smoke labels everything it creates with, so that the subscription it makes and the
# event it sends find each other.
LANGUAGE = "ruby"

# Where the subscription this smoke creates points. Nothing listens there, deliberately: what a
# delivery proves is proved once, by the webhook the harness catches and every language verifies.
NOWHERE = "http://127.0.0.1:1/"

# What a paced instance answers.
TOO_MANY_REQUESTS = 429

# The most times one request is sent again after that answer.
PACED_AGAIN = 8

# The shortest this waits between two tries, and the longest whatever the answer asked for.
SHORTEST_PAUSE = 0.2
LONGEST_PAUSE = 10.0

# The most digits a `Retry-After` may carry before it is read as the ceiling above rather than as a
# number. A header is written by a server this smoke does not control.
MOST_DIGITS = 9

# The most lines of a refusal's backtrace printed, so that a failure says where it happened without
# the console becoming the report.
MOST_BACKTRACE_LINES = 20

# What every generated method is issued through, waiting out a paced instance.
#
# Hook0 paces callers per credential, and a flow driving three dozen operations one after another
# is exactly what that is for. The answer says the request was not processed and is safe to send
# again after the delay it names, so this waits and sends it again rather than handing the caller a
# problem that says nothing about the operation it was asking about.
#
# It wraps the transport the gem ships rather than replacing it: `deliver` is what that transport
# offers a caller who needs what the answer carried beside its body, which is precisely the delay.
class Paced
  def initialize(inner)
    @inner = inner
  end

  # The shape the generated half reads, which is the status and the bytes.
  def request(method, path, query = [], body = nil)
    sent = 1
    loop do
      status, headers, payload = @inner.deliver(method, path, query, body)
      return [status, payload] if status != TOO_MANY_REQUESTS || sent > PACED_AGAIN

      sleep(pause(headers))
      sent += 1
    end
  end

  private

  # How long the answer says to wait, held between a floor and a ceiling of this smoke's own.
  #
  # The floor is there because the header counts in whole seconds and the delay being waited out is
  # a fraction of one, so a truthful `Retry-After: 0` would otherwise mean sending the same request
  # again immediately, forever. The ceiling is there because a header is written by a server this
  # smoke does not control.
  def pause(headers)
    written = headers["retry-after"].to_s.strip
    asked = written.match?(/\A\d{1,#{MOST_DIGITS}}\z/) ? Integer(written).to_f : SHORTEST_PAUSE

    asked.clamp(SHORTEST_PAUSE, LONGEST_PAUSE)
  end
end

# A setting the harness passes, or a refusal naming it: a smoke that ran without one would report a
# failure of the client for something the harness never handed it.
def setting(name)
  value = ENV[name].to_s
  raise "#{name} is not set" if value.empty?

  value
end

# The event both sends carry, under the identifier the caller names.
def event(event_type, event_id)
  Hook0::Event.new(
    event_type: event_type,
    payload: '{"from":"the ruby smoke"}',
    payload_content_type: "application/json",
    labels: { "language" => LANGUAGE },
    event_id: event_id
  )
end

# The same event, twice, under the identifier the API minted for the first of them.
def send_twice
  client = Hook0::Client.new(
    setting("HOOK0_API_URL"),
    setting("HOOK0_APPLICATION_ID"),
    setting("HOOK0_TOKEN")
  )
  event_type = setting("HOOK0_EVENT_TYPE")

  sent = client.send_event(event(event_type, nil))
  puts "ingested #{sent}"

  begin
    client.send_event(event(event_type, sent))
    raise "sending the same event twice was accepted twice"
  rescue Hook0::ClientError => e
    said = e.message.to_s
    raise "the second send failed without naming #{ALREADY_INGESTED}: #{said}" unless said.include?(ALREADY_INGESTED)
  end
  puts "the second send reported #{ALREADY_INGESTED}"
end

# Reports one operation the flow goes on to use the answer of, which has to be a success.
def read(operation)
  answered =
    begin
      yield
    rescue StandardError => e
      raise "#{operation}: the flow needs what it answers, and it answered #{e.class}: #{e.message}"
    end

  puts "exercised #{operation} accepted"
  answered
end

# Reports one operation driven for its own sake, whichever way the instance answered it.
#
# A success and a problem are both complete round trips through the generated layer: the request was
# composed, the instance answered, and this client read the answer. What is neither — the API not
# reached, a body this client cannot read, a problem it does not know — stops the smoke, because
# none of those say the client and the instance agree on anything.
def exercised(operation)
  begin
    yield
  rescue Hook0::Generated::ProblemError => e
    problem = e.problem
    raise "#{operation}: what came back names no problem this client knows: #{e.message}" if problem.nil?

    puts "exercised #{operation} refused:#{problem.id}"
    return
  rescue StandardError => e
    raise "#{operation}: #{e.class}: #{e.message}"
  end

  puts "exercised #{operation} accepted"
end

# Reports one generated model type as decoded out of a real answer.
#
# The value is taken rather than only named, so the line cannot outlive what it is about. Every value
# below is a member the document marks as always there, so `nil` is the one thing it cannot be —
# which is what makes this refuse rather than decorate.
def decoded(model, value)
  raise "#{model} was not decoded out of what the API answered" if value.nil?

  puts "decoded #{model}"
end

# The instance without the path the hand-written half is built with.
#
# The generated half composes paths that already carry `/api/v1`, since the API document's own
# server URL is the bare origin. Handing this transport the whole of `HOOK0_API_URL` happens to
# reach the same request, because a path of its own replaces the base's — but that is how one
# language joins two URLs rather than a contract, and the TypeScript client was posting to
# `/api/event` until the first live run found it. So this points at the origin, which is what the
# contract says.
def origin_of(api_url)
  parsed = URI.parse(api_url)
  port = parsed.port == parsed.default_port ? "" : ":#{parsed.port}"

  "#{parsed.scheme}://#{parsed.host}#{port}"
end

# Every operation the API document declares, driven against the instance in the order a consumer
# would: what it needs is created, read and listed, updated, and destroyed last.
#
# Two credentials, because the API takes two and one of them cannot do everything. An application
# secret is scoped to the application it belongs to; what belongs to the organization — listing its
# applications, everything about service tokens, its per-day counts — needs the organization-scoped
# token beside it.
def surface
  origin = origin_of(setting("HOOK0_API_URL"))
  application = setting("HOOK0_APPLICATION_ID")
  organization = setting("HOOK0_ORGANIZATION_ID")
  seeded = setting("HOOK0_SEEDED_APPLICATION_ID")
  labels = { "language" => LANGUAGE }

  held = Paced.new(Hook0::Transport.new(origin, setting("HOOK0_TOKEN")))
  organization_wide = Paced.new(Hook0::Transport.new(origin, setting("HOOK0_SERVICE_TOKEN")))

  applications = Hook0::Generated::ApplicationsApi.new(held)
  secrets = Hook0::Generated::ApplicationSecretsApi.new(held)
  event_types = Hook0::Generated::EventTypesApi.new(held)
  subscriptions = Hook0::Generated::SubscriptionsApi.new(held)
  events = Hook0::Generated::EventsApi.new(held)
  events_per_day = Hook0::Generated::EventsPerDayApi.new(held)
  instance = Hook0::Generated::InstanceApi.new(held)
  quotas = Hook0::Generated::QuotasApi.new(held)
  payload_content_types = Hook0::Generated::PayloadContentTypesApi.new(held)
  error_catalogue = Hook0::Generated::ErrorsApi.new(held)

  organization_applications = Hook0::Generated::ApplicationsApi.new(organization_wide)
  organization_events_per_day = Hook0::Generated::EventsPerDayApi.new(organization_wide)
  request_attempts = Hook0::Generated::RequestAttemptsApi.new(organization_wide)
  responses = Hook0::Generated::ResponseApi.new(organization_wide)
  service_tokens = Hook0::Generated::ServiceTokenApi.new(organization_wide)

  # What the instance says about itself, which is what an application asks before it has anything of
  # its own: how it is configured, what it will let this account do, what a payload may be, and
  # every problem it can report.
  decoded("InstanceConfig", read("instance.get") { instance.get })

  allowed = read("quotas.get") { quotas.get }
  decoded("QuotasResponseLimits", allowed.limits)
  decoded("QuotasResponse", allowed)

  exercised("payload_content_types.list") { payload_content_types.list }

  catalogue = read("errors.list") { error_catalogue.list }
  raise "the instance published an empty catalogue of the problems it can report" if catalogue.empty?

  decoded("ProblemId", catalogue.first.id)
  decoded("Problem", catalogue.first)

  # The application this smoke owns. One per language, so that the three deletions at the end of this
  # flow are real deletions rather than something eleven other smokes have to live with.
  info = read("applications.get") { applications.get(application) }
  decoded("ApplicationInfoConsumption", info.consumption)
  decoded("ApplicationInfoQuotas", info.quotas)
  decoded("ApplicationInfoOnboardingStepsEvent", info.onboarding_steps.event)
  decoded("ApplicationInfoOnboardingStepsEventType", info.onboarding_steps.event_type)
  decoded("ApplicationInfoOnboardingStepsSubscription", info.onboarding_steps.subscription)
  decoded("ApplicationInfoOnboardingSteps", info.onboarding_steps)
  decoded("ApplicationInfo", info)

  renamed = read("applications.update") do
    applications.update(
      application,
      Hook0::Generated::ApplicationPost.new(
        name: "the application the ruby smoke drives",
        organization_id: organization
      )
    )
  end
  decoded("Application", renamed)

  # The organization's, so the organization credential. Listing what an account has is the first
  # thing a console does.
  exercised("applications.list") { organization_applications.list(organization) }

  # This one is driven with the *application* secret on purpose, and it is the flow's one refusal.
  # Creating an application is the organization's business and an application secret is not the
  # organization's, so the instance answers a problem document and this client reads it — which is
  # the half of the client that nothing else here would exercise.
  exercised("applications.create") do
    applications.create(
      Hook0::Generated::ApplicationPost.new(
        name: "an application the ruby smoke's application secret may not create",
        organization_id: organization
      )
    )
  end

  # A second secret, so that the one this smoke is authenticating with is never the one it revokes.
  # Deleting that one succeeds and then locks the flow out of everything below.
  minted = read("applicationSecrets.create") do
    secrets.create(
      Hook0::Generated::ApplicationSecretPost.new(
        application_id: application,
        name: "a secret the ruby smoke minted"
      )
    )
  end
  decoded("ApplicationSecret", minted)

  exercised("applicationSecrets.read") { secrets.read(application) }
  exercised("applicationSecrets.update") do
    secrets.update(
      minted.token,
      Hook0::Generated::ApplicationSecretPost.new(
        application_id: application,
        name: "a secret the ruby smoke renamed"
      )
    )
  end
  exercised("applicationSecrets.delete") { secrets.delete(minted.token, application) }

  # An event type of this smoke's own, rather than the one the harness declared: what is created here
  # is what is subscribed to, sent, replayed and deleted below.
  declared = read("eventTypes.create") do
    event_types.create(
      Hook0::Generated::EventTypePost.new(
        application_id: application,
        resource_type: "smoke",
        service: LANGUAGE,
        verb: "ran"
      )
    )
  end
  decoded("EventType", declared)

  exercised("eventTypes.get") { event_types.get(declared.event_type_name, application) }
  exercised("eventTypes.list") { event_types.list(application) }

  target = Hook0::Generated::SubscriptionPostTarget.new(
    headers: {},
    method_: "POST",
    type: "http",
    url: NOWHERE
  )
  subscription = read("subscriptions.create") do
    subscriptions.create(
      Hook0::Generated::SubscriptionPost.new(
        application_id: application,
        event_types: [declared.event_type_name],
        is_enabled: true,
        target: target,
        description: "what the ruby smoke subscribes to its own events with",
        labels: labels
      )
    )
  end
  decoded("SubscriptionTarget", subscription.target)
  decoded("Subscription", subscription)

  exercised("subscriptions.get") { subscriptions.get(subscription.subscription_id) }
  exercised("subscriptions.list") { subscriptions.list(application) }
  exercised("subscriptions.update") do
    subscriptions.update(
      subscription.subscription_id,
      Hook0::Generated::SubscriptionPost.new(
        application_id: application,
        event_types: [declared.event_type_name],
        is_enabled: true,
        target: target,
        description: "what the ruby smoke renamed it to",
        labels: labels
      )
    )
  end

  # The event the subscription above selects, sent through the generated layer rather than through
  # send_event: the hand-written half has its own three questions above, and this is the operation
  # the document declares.
  ingested = read("events.ingest") do
    events.ingest(
      Hook0::Generated::EventPost.new(
        application_id: application,
        event_type: declared.event_type_name,
        labels: labels,
        occurred_at: Time.now.utc,
        payload: '{"from":"the ruby smoke"}',
        payload_content_type: "application/json",
        event_id: Hook0.generate_event_id
      )
    )
  end
  decoded("IngestedEvent", ingested)

  decoded("EventWithPayload", read("events.get") { events.get(ingested.event_id, application) })

  listed = read("events.list") { events.list(application) }
  raise "the instance ingested an event and then listed none" if listed.empty?

  decoded("Event", listed.first)

  exercised("events.replay") do
    events.replay(ingested.event_id, Hook0::Generated::ReplayEvent.new(application_id: application))
  end

  # This application was created a moment ago and the counts come out of a view the instance
  # refreshes on a cycle of its own, so this answers a list with nothing in it — which is an answer,
  # and one a client has to be able to read.
  exercised("events_per_day.list_for_application") { events_per_day.list_for_application(application) }

  # The organization's counts do have something in them: the harness waited for the instance to
  # refresh them before running any of this, precisely so that the type they are answered with is one
  # a client decodes rather than one nothing ever produces.
  per_day = read("events_per_day.list_for_organization") do
    organization_events_per_day.list_for_organization(organization)
  end
  raise "the organization has ingested events and its per-day counts are empty" if per_day.empty?

  decoded("EventsPerDayEntry", per_day.first)

  # An attempt and a response exist only once the output worker has finished a delivery. The harness
  # waited for one, in the application it caught the shared delivery from, and handed the ids on — so
  # this reads them back with the organization credential rather than waiting again.
  exercised("requestAttempts.read") { request_attempts.read(seeded) }

  attempted = read("requestAttempts.get") do
    request_attempts.get(setting("HOOK0_REQUEST_ATTEMPT_ID"), seeded)
  end
  decoded("RequestAttemptEvent", attempted.event)
  decoded("RequestAttemptSubscription", attempted.subscription)
  decoded("RequestAttemptStatusType", attempted.status.type)
  decoded("RequestAttemptStatus", attempted.status)
  decoded("RequestAttempt", attempted)

  decoded("Response", read("response.get") { responses.get(setting("HOOK0_RESPONSE_ID"), seeded) })

  # Service tokens belong to the organization, so they are minted, read and revoked with the
  # organization credential. The one revoked below is the one minted here — never the one this half
  # of the flow is authenticating with.
  issued = read("serviceToken.create") do
    service_tokens.create(
      Hook0::Generated::ServiceTokenPost.new(
        name: "a token the ruby smoke minted",
        organization_id: organization
      )
    )
  end
  decoded("ServiceToken", issued)

  exercised("serviceToken.list") { service_tokens.list(organization) }
  exercised("serviceToken.get") { service_tokens.get(issued.token_id, organization) }
  exercised("serviceToken.edit") do
    service_tokens.edit(
      issued.token_id,
      Hook0::Generated::ServiceTokenPost.new(
        name: "a token the ruby smoke renamed",
        organization_id: organization
      )
    )
  end
  exercised("serviceToken.delete") { service_tokens.delete(issued.token_id, organization) }

  # Destroyed in the order the instance can accept: the subscription that references the event type,
  # then the event type, then the application — which is last because the secret this whole flow
  # authenticates with stops authenticating the moment its application is gone.
  exercised("subscriptions.delete") { subscriptions.delete(subscription.subscription_id, application) }
  exercised("eventTypes.delete") { event_types.delete(declared.event_type_name, application) }
  exercised("applications.delete") { applications.delete(application) }
end

# Verifies what the output worker really delivered, with this client's own verification.
def verify(delivery)
  headers = {}
  File.read(File.join(delivery, "headers")).each_line do |line|
    name, value = line.chomp.split(": ", 2)
    headers[name] = value unless value.nil?
  end

  Hook0.verify_webhook_signature(
    File.read(File.join(delivery, "signature")).strip,
    File.binread(File.join(delivery, "body")),
    headers,
    File.read(File.join(delivery, "secret")).strip,
    Integer(File.read(File.join(delivery, "tolerance")).strip)
  )
end

begin
  send_twice
  surface

  # Last, and on purpose: it needs no instance at all, so it still answers after the flow above has
  # deleted the application it was run against.
  verify(setting("HOOK0_DELIVERY"))
  puts "the signature the instance produced verifies"
rescue StandardError => e
  warn("#{e.class}: #{e.message}")
  warn(e.backtrace.take(MOST_BACKTRACE_LINES).join("\n")) unless e.backtrace.nil?
  exit(1)
end
