# frozen_string_literal: true

# Every operation the API declares, driven over a socket and held to what the document declares it
# as: the path it filled in, the query it assembled, the credential it carried, and the value it
# read back out of what the API answered.
#
# Nothing here names an operation, a group or an argument. The groups are found on the generated
# module, their methods and the arguments each one takes are read off what the generator wrote about
# them, and what every call is held to is read off the API document the generator was run against.

require_relative "api_document"
require_relative "generated_surface"
require_relative "test_helper"

module Hook0Test
  class GeneratedOperationsTest < ApiCase
    # Every operation the document declares issues the request it is declared as, and reads it back.
    #
    # Asked twice, in a case each so that neither runs into the other's connections: once giving
    # every argument an operation may be asked with, once giving only the ones it requires, which is
    # what says an argument left out leaves the query it would have filled empty. The closing
    # assertion is the one that grows — an operation the API adds fails it until something drives it.
    def test_every_operation_the_document_declares_is_reached_with_everything_it_may_carry
      assert_every_operation_is_reached(true)
    end

    def test_every_operation_the_document_declares_is_reached_with_only_what_it_requires
      assert_every_operation_is_reached(false)
    end

    private

    def assert_every_operation_is_reached(optionals)
      reached = GeneratedSurface.groups.flat_map { |group| drive(group, optionals) }

      assert_equal ApiDocument.operations.map(&:named).sort, reached.uniq.sort,
                   "the generated groups reach other operations than the document declares"
    end

    # One group of operations, every one of them driven over a socket, and what they reached.
    def drive(group, optionals)
      reaching = group.new(Hook0::Transport.new(@api.base_url, "token-xyz"))

      GeneratedSurface.operations_of(group).map do |name|
        ordered, named = GeneratedSurface.arguments(group, name, optionals)
        answered, expected = answer_for(group, name, optionals)
        @api.will_answer(ScriptedResponse.new(200, answered))

        read = named.empty? ? reaching.public_send(name, *ordered) : reaching.public_send(name, *ordered, **named)

        held(group, name, read, expected, optionals).named
      end
    end

    # What one operation put on the wire, held to what the document declares it as.
    def held(group, name, read, expected, optionals)
      request = @api.received.last
      declared = ApiDocument.reached(request)
      named = "#{group}##{name}"

      if expected.nil?
        assert_nil read, "#{named} answered something although the operation answers nothing"
      else
        assert_equal expected, read, "#{named} did not read back what the API answered"
      end
      assert_equal "Bearer token-xyz", request.headers["authorization"], named
      assert_equal "application/json", request.headers["accept"], named
      assert_path_filled_in(declared, request, named)
      assert_query_assembled(declared, request, group, name, optionals)
      declared
    end

    def assert_path_filled_in(declared, request, named)
      sent = request.target.split("?", 2).first.split("/", -1)
      declared.template.split("/", -1).each_with_index do |segment, index|
        next unless segment.start_with?("{")

        # The value lands in the path escaped, so that nothing in it can name a segment the
        # operation never had.
        assert_equal "a%20value%2Fwith%20a%20space", sent[index], "#{named} left `#{segment}` unescaped"
      end
    end

    def assert_query_assembled(declared, request, group, name, optionals)
      carried = query_of(request)

      assert_equal declared.query_names(optionals), carried.keys.sort,
                   "#{group}##{name} assembled a query the document does not declare"
      carried.each_value do |values|
        assert_equal [GeneratedSurface::ARGUMENT_TEXT], values, "#{group}##{name} carried a value altered"
      end
    end

    def query_of(request)
      written = request.target.split("?", 2)[1]
      return {} if written.nil? || written.empty?

      written.split("&").each_with_object({}) do |pair, carried|
        name, value = pair.split("=", 2)
        (carried[unescaped(name)] ||= []) << unescaped(value.to_s)
      end
    end

    # The inverse of what the transport writes a query with, which is form encoding: a space
    # travels as `+` there, and everything else that is not left alone travels percent-escaped.
    def unescaped(written)
      written.tr("+", " ").gsub(/%\h{2}/) { |escaped| escaped[1..].to_i(16).chr }.force_encoding(Encoding::UTF_8)
    end

    # What the API answers one operation, and the value that operation is expected to read out of it.
    def answer_for(group, name, optionals)
      answers = GeneratedSurface.answered(group, name)
      return [nil, nil] if answers.nil?

      type, listed = answers
      value = GeneratedSurface.value_of(type, optionals)
      listed ? [[written_form(value)], [value]] : [written_form(value), value]
    end

    def written_form(value)
      value.class.respond_to?(:from_json) ? value.to_h : value
    end
  end
end
