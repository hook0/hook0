# frozen_string_literal: true

require_relative "lib/hook0/version"

Gem::Specification.new do |spec|
  spec.name = "hook0-client"
  spec.version = Hook0::VERSION
  spec.summary = "Ruby SDK for Hook0, open-source Webhooks as a service for SaaS"
  spec.description = <<~TEXT
    Send events to Hook0, upsert the event types your application uses, verify the signature of an
    incoming webhook, and call every operation the API declares through generated, documented
    classes. Sending is idempotent and retried under bounds the caller sets.
  TEXT
  spec.authors = ["David Sferruzza", "François-Guillaume Ribreau"]
  spec.email = ["david@hook0.com", "fg@hook0.com"]
  spec.license = "MIT"

  spec.homepage = "https://www.hook0.com/"
  spec.metadata = {
    "homepage_uri" => "https://www.hook0.com/",
    "documentation_uri" => "https://documentation.hook0.com/",
    "source_code_uri" => "https://gitlab.com/hook0/hook0",
    "rubygems_mfa_required" => "true"
  }

  spec.required_ruby_version = ">= 3.1"

  spec.files = Dir["lib/**/*.rb"] + Dir["assets/*.svg"] + ["README.md"]
  spec.require_paths = ["lib"]

  # The SDK reaches the network, verifies signatures and decodes what the API answers with nothing
  # but the standard library, so installing it can never drag a transitive dependency into an
  # application that only wanted to send an event. A runtime dependency appearing here fails
  # `test/gemspec_test.rb`, which is what keeps that sentence true rather than aspirational.
  spec.add_development_dependency "minitest", ">= 5.25"
  spec.add_development_dependency "rubocop", "~> 1.89"
end
