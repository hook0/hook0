# frozen_string_literal: true

# The Ruby client against a Hook0 that is really running.
#
# Three things the loopback suite cannot ask: whether an application secret the API minted is
# accepted, whether a second send under an identifier already ingested is reported as the conflict
# it is, and whether a signature the output worker computed verifies. Everything else about this
# client is settled by `clients/ruby/test`.

$LOAD_PATH.unshift(File.expand_path("../../../clients/ruby/lib", __dir__))

require "hook0"

# The conflict the API answers a duplicated ingestion with.
ALREADY_INGESTED = "EventAlreadyIngested"

# A setting the harness passes, or a refusal naming it.
def setting(name)
  value = ENV[name].to_s
  abort("#{name} is not set") if value.empty?
  value
end

# The event both sends carry, under the identifier the caller names.
def event(event_type, event_id)
  Hook0::Event.new(
    event_type: event_type,
    payload: '{"from":"the ruby smoke"}',
    payload_content_type: "application/json",
    labels: { "language" => "ruby" },
    event_id: event_id
  )
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
  abort("sending the same event twice was accepted twice")
rescue Hook0::ClientError => refused
  said = refused.message.to_s
  abort("the second send failed without naming #{ALREADY_INGESTED}: #{said}") unless said.include?(ALREADY_INGESTED)
  puts "the second send reported #{ALREADY_INGESTED}"
end

verify(setting("HOOK0_DELIVERY"))
puts "the signature the instance produced verifies"
