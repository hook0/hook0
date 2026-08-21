# frozen_string_literal: true

# What the API declares, read out of the document the generator was run against.
#
# Nothing here names an operation. The document is the one place that says which requests exist, so
# it is what the suite holds the generated half to: an operation the API grows is one more entry
# here the moment the snapshot carries it, and one it drops takes its entry with it.

require "json"

module Hook0Test
  # One operation the API document declares, as a request has to look to be it.
  DeclaredOperation = Struct.new(:verb, :template, :required_query, :optional_query) do
    # How this operation reads in a message, which is what names it when one fails.
    def named
      "#{verb} #{template}"
    end

    # Every name the query may carry, whether the operation requires it or not.
    def query_names(with_optional)
      (with_optional ? required_query + optional_query : required_query).sort
    end

    # Whether a request landed on this operation.
    def matches?(request)
      return false unless request.verb == verb

      wanted = template.split("/", -1)
      sent = request.target.split("?", 2).first.split("/", -1)
      return false unless wanted.size == sent.size

      wanted.zip(sent).all? do |declared, segment|
        # A parameter stands for a segment that is there; an empty one is the trailing slash of
        # another path rather than a value.
        declared.start_with?("{") && declared.end_with?("}") ? !segment.empty? : declared == segment
      end
    end
  end

  # Everything the OpenAPI snapshot says about the operations an SDK is built out of.
  module ApiDocument
    # The tag that marks an operation as part of the surface an SDK exposes, which is the rule the
    # generator applies — see `SDK_TAG` in `clients/sdkgen/src/snapshot.rs`.
    SDK_TAG = "sdk"

    # The methods a request line can carry, which is what tells an operation from the rest.
    VERBS = %w[get put post delete options head patch trace].freeze

    # No checkout nests this gem deeper than this below its root.
    MAX_ANCESTORS = 8

    # Largest document read back, far above what the API's snapshot ever is.
    MAX_DOCUMENT_BYTES = 8 * 1024 * 1024

    # Every operation an SDK is built out of.
    #
    # A document that marks nothing public exposes all of itself, and one that marks anything
    # exposes what it marked. Both are what the generator does with the tag.
    def self.operations
      @operations ||= begin
        found = declared
        raise "the API document declares no operation at all" if found.empty?

        public_ones = found.select { |public_one, _| public_one }
        (public_ones.empty? ? found : public_ones).map { |_, operation| operation }
      end
    end

    # Which operation of the document a request landed on, refusing one that is none or many.
    def self.reached(request)
      matched = operations.select { |operation| operation.matches?(request) }
      unless matched.size == 1
        raise "`#{request.verb} #{request.target}` is #{matched.size} of the operations the document declares"
      end

      matched.first
    end

    def self.declared
      document.fetch("paths").flat_map do |template, item|
        item.filter_map do |verb, operation|
          next unless VERBS.include?(verb)

          [(operation["tags"] || []).include?(SDK_TAG), of(template, verb, operation)]
        end
      end
    end
    private_class_method :declared

    def self.of(template, verb, operation)
      query = (operation["parameters"] || []).select { |parameter| parameter["in"] == "query" }
      required, optional = query.partition { |parameter| parameter["required"] == true }

      DeclaredOperation.new(
        verb.upcase,
        template,
        required.map { |parameter| parameter.fetch("name") }.sort,
        optional.map { |parameter| parameter.fetch("name") }.sort
      )
    end
    private_class_method :of

    # The OpenAPI document the generator was run against, out of the repository holding it.
    def self.document
      at = __dir__
      MAX_ANCESTORS.times do
        candidate = File.join(at, "api", "openapi.snapshot.json")
        return read(candidate) if File.file?(candidate)

        at = File.dirname(at)
      end

      raise "no `api/openapi.snapshot.json` within #{MAX_ANCESTORS} directories of #{__dir__}"
    end
    private_class_method :document

    def self.read(path)
      size = File.size(path)
      raise "#{path} is #{size} bytes long, above the #{MAX_DOCUMENT_BYTES} read back" if size > MAX_DOCUMENT_BYTES

      JSON.parse(File.read(path, encoding: "UTF-8"))
    end
    private_class_method :read
  end
end
