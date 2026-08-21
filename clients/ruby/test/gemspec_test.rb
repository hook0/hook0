# frozen_string_literal: true

# What installing this gem is allowed to drag in, which is nothing.
#
# The gem reaches the network, verifies signatures and decodes what the API answers with the
# standard library alone. That sentence is worth exactly as much as the guard behind it, so it is a
# case rather than a line in a pipeline: a `add_dependency` appearing in the gemspec fails here,
# wherever the suite runs.

require_relative "test_helper"

module Hook0Test
  class GemspecTest < Minitest::Test
    def gemspec
      Gem::Specification.load(File.expand_path("../hook0-client.gemspec", __dir__))
    end

    def test_the_gem_declares_no_runtime_dependency
      declared = gemspec.dependencies.select { |dependency| dependency.type == :runtime }

      assert_empty declared.map(&:name),
                   "the gem has grown a runtime dependency; it is meant to reach for the standard library alone"
    end

    def test_the_gem_ships_both_halves_of_what_it_is
      shipped = gemspec.files

      assert_includes shipped, "lib/hook0.rb"
      assert_includes shipped, "lib/hook0/signature.rb"
      assert_includes shipped, "lib/hook0/generated/all.rb"
      assert_includes shipped, "README.md"
    end

    def test_the_gem_is_released_under_the_version_the_library_names
      assert_equal Hook0::VERSION, gemspec.version.to_s
    end
  end
end
