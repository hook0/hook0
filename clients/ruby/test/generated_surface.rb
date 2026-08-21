# frozen_string_literal: true

# Everything the generator wrote, found by looking at what it wrote, and one value of each.
#
# Ruby spells a UUID, a moment and a name all as `String`, so what tells them apart is what the
# generator wrote beside them: the documentation of a constructor names the type of every member,
# and the documentation of an operation names both the type of every argument and the wire name it
# travels under. Nothing here lists a schema, a member or an operation — a value of anything the
# generator writes is built by reading what it wrote about it.

require "date"
require "time"

module Hook0Test
  # What the generator declared, and one value of each thing it declared.
  module GeneratedSurface
    # What every string-shaped member of a value is given.
    #
    # A UUID, because Ruby spells a UUID as a `String` and the reader for one refuses anything else:
    # a member that turns out to be a UUID round-trips, and one that is free text carries a UUID
    # through unchanged.
    MODEL_TEXT = "3f2504e0-4f89-41d3-9a0c-0305e82c3301"

    # What every string-shaped argument of an operation is given. It carries the two characters a
    # path segment may not leave as they are, so a value reaching a path proves it was escaped.
    ARGUMENT_TEXT = "a value/with a space"

    # What a member the document describes nothing about carries, kept as it arrived.
    AN_OPAQUE_VALUE = { "the document" => %w[describes none of this] }.freeze

    # No schema the API declares nests anywhere near this deep.
    MAX_DEPTH = 8

    # No file the generator writes is longer than this, which bounds what is read back to find a
    # docstring in it.
    MAX_SOURCE_LINES = 50_000

    # What the generator wrote about one argument or member: the type it declared it as, and what
    # it said about it beside that — the wire name it travels under, and the closed list it is one
    # of when Ruby spells that list as a plain `String`.
    Documented = Struct.new(:type, :rest) do
      # The same thing said about a type one level in, which is what an item of a list or a value
      # of a map is described by.
      def of(carried)
        Documented.new(carried, rest)
      end
    end

    # Every value the generator wrote a reader for.
    def self.models
      found = declared.select { |named| named.respond_to?(:from_json) }
      raise "the generator wrote no value with a reader" if found.empty?

      found
    end

    # Every closed list of strings the generator wrote.
    def self.enumerations
      found = declared.select { |named| named.respond_to?(:member?) }
      raise "the generator wrote no closed list of strings" if found.empty?

      found
    end

    # Every group of operations the generator wrote, which is every class it hands a transport to.
    def self.groups
      found = declared.select do |named|
        named.is_a?(Class) && named.instance_method(:initialize).parameters == [%i[req transport]]
      end
      raise "the generator wrote no group of operations at all" if found.empty?

      found
    end

    # Every operation one group carries, under the name it is called by.
    def self.operations_of(group)
      group.public_instance_methods(false).sort
    end

    # Everything the generator declared into its namespace, by the name it declared it under.
    def self.declared
      Hook0::Generated.constants.map { |named| Hook0::Generated.const_get(named) }.grep(Module)
    end

    # One value of a schema the API declares, with every member it may leave out set or not.
    def self.model(declared_class, optionals)
      built(declared_class, optionals, 0)
    end

    # What one operation is asked with: everything it requires, and what it does not as asked for.
    #
    # Answers the arguments it takes in order and the ones it takes by name, which is how Ruby
    # spells the two halves of what the document calls required and optional.
    def self.arguments(group, name, optionals)
      documented = params_of(group.instance_method(name))
      ordered = []
      named = {}
      group.instance_method(name).parameters.each do |kind, argument|
        next if kind == :key && !optionals

        value = value_for(documented.fetch(argument), optionals, ARGUMENT_TEXT, 0)
        kind == :key ? named[argument] = value : ordered << value
      end

      [ordered, named]
    end

    # What a method answers, as the generator wrote it: the type of the value, and whether the
    # operation answers a list of them rather than one. Nothing at all when it answers nothing.
    def self.answered(group, name)
      written = returned_of(group.instance_method(name))
      return nil if written.nil? || written == "void"

      listed = written.match(/\AArray<(.+)>\z/)
      listed ? [listed[1], true] : [written, false]
    end

    # One value of the type the generator wrote, as a member of a value carries it.
    def self.value_of(type, optionals)
      value_for(Documented.new(type, ""), optionals, MODEL_TEXT, 0)
    end

    # What the generator wrote about every argument or member, by the name Ruby calls it: the type
    # it declared it as, and what it said about it beside that.
    def self.params_of(unbound)
      docstring_of(unbound).filter_map do |line|
        found = line.match(/\A@param\s+(\S+)\s+\[(.+?)\](.*)\z/m)
        [found[1].to_sym, Documented.new(found[2].strip, found[3].strip)] if found
      end.to_h
    end

    # The wire name every argument or member travels under, by the name Ruby calls it.
    def self.wire_names_of(unbound)
      docstring_of(unbound).filter_map do |line|
        found = line.match(/\A@param\s+(\S+)\s+\[.+?\]\s+carries\s+`([^`]+)`/m)
        [found[1].to_sym, found[2]] if found
      end.to_h
    end

    def self.returned_of(unbound)
      found = docstring_of(unbound).find { |line| line.start_with?("@return ") }
      found&.match(/\A@return\s+\[(.+?)\]/)&.[](1)
    end
    private_class_method :returned_of

    # What the generator wrote above a method, one tag per entry with its continuations folded in.
    def self.docstring_of(unbound)
      @docstrings ||= {}
      @docstrings[unbound.owner.name.to_s + unbound.name.to_s] ||= begin
        path, line = unbound.source_location
        raise "#{unbound.owner}##{unbound.name} was not written down anywhere" if path.nil?

        folded(comments_above(path, line))
      end
    end
    private_class_method :docstring_of

    def self.comments_above(path, line)
      @sources ||= {}
      lines = (@sources[path] ||= File.readlines(path, chomp: true))
      raise "#{path} is longer than the #{MAX_SOURCE_LINES} lines read back" if lines.size > MAX_SOURCE_LINES

      block = []
      index = line - 2
      while index >= 0 && lines[index].strip.start_with?("#")
        block.unshift(lines[index].strip.sub(/\A#\s?/, "").strip)
        index -= 1
      end

      block
    end
    private_class_method :comments_above

    # A tag the generator folded over several lines is one entry again.
    def self.folded(block)
      block.each_with_object([]) do |line, tags|
        line.start_with?("@") || tags.empty? ? tags << line : tags[-1] = "#{tags.last} #{line}"
      end
    end
    private_class_method :folded

    def self.value_for(documented, optionals, text, depth)
      raise "`#{documented.type}` nests more than #{MAX_DEPTH} deep" if depth > MAX_DEPTH

      carried = peeled(documented.type)
      listed = carried.match(/\AArray<(.+?)>\z/)
      return [value_for(documented.of(listed[1]), optionals, text, depth + 1)] if listed

      keyed = carried.match(/\AHash\{[^=]+=>\s*(.+?)\}\z/)
      return { "a key" => value_for(documented.of(keyed[1]), optionals, text, depth + 1) } if keyed

      scalar(carried, documented, optionals, text, depth)
    end
    private_class_method :value_for

    def self.scalar(carried, documented, optionals, text, depth)
      case carried
      when "String" then of_a_closed_list(documented.rest) || text
      when "Integer" then 12
      when "Float" then 1.5
      when "Boolean" then true
      when "Object" then AN_OPAQUE_VALUE
      when "Time" then Time.utc(2026, 1, 2, 3, 4, 5)
      when "Date" then Date.new(2026, 1, 2)
      else built(Hook0::Generated.const_get(carried), optionals, depth + 1)
      end
    end
    private_class_method :scalar

    # One of the values a closed list declares, when what the generator wrote names one.
    #
    # Which value it is does not matter; that it is one of them does.
    def self.of_a_closed_list(rest)
      found = rest.match(/one of\s+`([A-Za-z0-9_]+)::VALUES`/)
      return nil if found.nil?

      Hook0::Generated.const_get(found[1])::VALUES.first
    end
    private_class_method :of_a_closed_list

    # What a value is when it is there. Whether it may be absent is the generator's `nil:` default
    # rather than the `, nil` it writes beside the type, so only the type is read here.
    def self.peeled(type)
      type.sub(/,\s*nil\z/, "")
    end
    private_class_method :peeled

    def self.built(declared_class, optionals, depth)
      constructor = declared_class.instance_method(:initialize)
      documented = params_of(constructor)
      held = constructor.parameters.to_h do |kind, name|
        [name, kind == :key && !optionals ? nil : value_for(documented.fetch(name), optionals, MODEL_TEXT, depth)]
      end

      declared_class.new(**held)
    end
    private_class_method :built
  end
end
