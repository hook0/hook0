# frozen_string_literal: true

# What the dashboard shows under "Send an event", for Ruby.
#
# This file exists so that the snippet is held against the real client. A renamed keyword, a moved
# constructor or a dropped field turns `clients.ruby.check` red on the day it happens, which is the
# whole reason the snippet lives here rather than in the dashboard: one written by hand over there
# is backed by nothing and drifts in silence.
#
# Two pairs of markers say how it is read. `hook0:snippet` delimits what a reader is shown, so that
# anything this file needs only in order to be checked stays out of it. `hook0:label` delimits the
# one rendering of a label, which the dashboard repeats once per label the form carries and joins
# with the separator its manifest declares — the region carries no trailing separator of its own,
# and sits inside its container, so no label at all leaves a valid empty one.
#
# The `__HOOK0_*__` words are string literals, which is what lets a file full of them be read as
# Ruby. They never resolve to anything: this example is checked, never run.

# hook0:snippet:begin
require "hook0"

client = Hook0::Client.new(
  "__HOOK0_API_URL__",
  "__HOOK0_APPLICATION_ID__",
  "__HOOK0_TOKEN__"
)

event_id = client.send_event(
  Hook0::Event.new(
    event_type: "__HOOK0_EVENT_TYPE__",
    payload: "__HOOK0_PAYLOAD__",
    payload_content_type: "application/json",
    labels: {
      # hook0:label:begin
      "__HOOK0_LABEL_KEY__" => "__HOOK0_LABEL_VALUE__" # hook0:label:end
    }
  )
)

puts "ingested as #{event_id}"
# hook0:snippet:end
