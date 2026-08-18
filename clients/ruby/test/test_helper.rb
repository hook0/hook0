# frozen_string_literal: true

# A Hook0 API on a loopback port, and what every case is written against.
#
# Every case below goes over a real socket: the request the client builds, the headers it sets, the
# way it reads an answer and the way it gives up on one are all the real ones. Nothing here stands
# in for a part of the client, so a case that passes says the client works rather than that it was
# called.
#
# The server is a plain `TCPServer` speaking as much HTTP/1.1 as one exchange needs. WEBrick left
# the standard library in Ruby 3.0, and this suite is meant to run wherever the gem does — with
# nothing installed beside it.

require "json"
require "minitest/autorun"
require "socket"
require "time"

require "simplecov"

# Coverage is measured on every run rather than behind a flag, because the floors below are only
# enforced when it is. That keeps the gate inside the command the pipeline already runs rather than
# in a job or a script of its own. It has to be started before `hook0` is required, or the lines
# loaded before it would count as unreachable.
SimpleCov.start do
  enable_coverage :branch

  # What the gem ships, and only that. The suite itself is not part of the gem, and holding it to a
  # number would make the floor about the tests rather than about the client.
  add_filter %r{^/test/}

  # Every file the gem ships, whether or not a run happened to load one. Without this the
  # denominator is only what was required, so the last case that reaches a file going away would
  # take the file out of the report rather than show it uncovered — and the percentage would rise.
  # A floor over a denominator the suite can shrink is a floor that rewards deleting tests.
  #
  # One caveat, for a run on a laptop rather than in CI: SimpleCov merges results from repeated
  # runs for ten minutes, so a second run over a source tree that changed in between reports a
  # denominator belonging to neither. Delete `coverage/` before measuring. CI runs once in a fresh
  # container, so it never sees this, and turning merging off here costs a warning on every run.
  track_files "lib/**/*.rb"

  # What this suite reaches today, each truncated to the hundredth rather than rounded, so that a
  # floor can never sit above the run it was read from. Not headroom: every case here talks to a
  # loopback socket and the property cases are derandomised, so there is no run-to-run noise to
  # leave room for, and room would only buy silent decay.
  #
  # Everything the gem ships is at 100% but `runtime.rb`, which is short by seven lines: the arm
  # that refuses a value that is neither an integer nor a float, the refusal of a body above the
  # ceiling, and four arms of the `case` in `written` that no value the API declares reaches.
  # Raise these when the suite covers more; lowering one is a decision somebody has to write here.
  minimum_coverage line: 99.64, branch: 97.09

  # The same run read per file, so that a drop names the file it happened in rather than only moving
  # the total. One number applied to every file, so it is what the weakest of them reaches —
  # `runtime.rb`, for the reasons above — and not a per-file figure for each.
  #
  # Written in the block form rather than as `minimum_coverage_by_file`, which the version pinned
  # beside this file deprecates: the old spelling still works, and prints a deprecation on every
  # run for as long as it does.
  #
  # The figure is what the OLDEST-counting Ruby reports, not what the machine it was written on did,
  # because the denominator moves with the interpreter. `runtime.rb` is 90 of 97 lines on 3.2 and 89
  # of 96 on the 3.4 the pipeline runs: the same seven arms go uncovered either way, but one line 3.2
  # counts as relevant and covered is not counted at all by 3.4. Read off 3.2 this is 92.7835, off
  # 3.4 it is 92.7083, and a floor at the first fails every pipeline while passing every laptop.
  #
  # So it is the lower of the two, truncated to the hundredth. Anyone raising it has to check it
  # against the Ruby in `RUBY_VERSION`, not against the one they happen to have.
  coverage(:line) { minimum_per_file 92.70 }
end

$LOAD_PATH.unshift(File.expand_path("../lib", __dir__))

require "hook0"

module Hook0Test
  # No case talks to anything but a loopback socket, so none of them has a reason to take this long.
  TIMEOUT = 20

  # No request a case makes is anywhere near this large; the cap bounds what one connection reads.
  MAX_REQUEST_BODY_BYTES = 64 * 1024

  # Longest request line or header line the server reads.
  MAX_LINE_BYTES = 8 * 1024

  # Most header lines the server reads out of one request.
  MAX_HEADERS = 64

  # Most connections one case opens, which bounds what the server holds at once.
  MAX_CONNECTIONS = 64

  # The shape a UUID has, whichever version it carries.
  UUID_PATTERN = /\A\h{8}-\h{4}-\h{4}-\h{4}-\h{12}\z/

  # What the API answers to one request, in the order the case scripted it.
  ScriptedResponse = Struct.new(:status, :body, :held_for, :headers) do
    def initialize(status, body, held_for = 0.0, headers = {})
      super
    end
  end

  # A request the API received, in the order it received it.
  #
  # The verb is `verb` rather than `method`, which every object already answers to: a member of that
  # name would replace it, and a case asking what a request was is not asking for a `Method`.
  ReceivedRequest = Struct.new(:verb, :target, :headers, :body) do
    def json
      JSON.parse(body)
    end
  end

  # A Hook0 API listening on a loopback port for the lifetime of one case.
  class FakeApi
    attr_reader :received

    def initialize
      @server = TCPServer.new("127.0.0.1", 0)
      @received = []
      @scripted = []
      @answered = 0
      @lock = Mutex.new
      @serving = []
      @thread = Thread.new { serve }
      @thread.abort_on_exception = false
    end

    # Where the client reaches this API.
    def base_url
      "http://127.0.0.1:#{@server.addr[1]}"
    end

    # Queues the answers the case expects the client to draw, in order.
    def will_answer(*responses)
      @lock.synchronize { @scripted.concat(responses) }
    end

    # The event identifier request number `index` carried, as the API read it.
    def event_id_of(index)
      raise "expected at least #{index + 1} requests, got #{@received.size}" if index >= @received.size

      @received[index].json["event_id"]
    end

    def close
      @server.close
      @thread.join(TIMEOUT)
      @serving.each { |serving| serving.join(TIMEOUT) }
    end

    private

    # One connection per thread, so that an answer a case asks the API to sit on holds that
    # exchange and no other — which is what a client retrying a timed-out attempt walks into.
    def serve
      loop do
        socket = @server.accept
        @lock.synchronize do
          raise IOError, "a case opened more than #{MAX_CONNECTIONS} connections" if @serving.size >= MAX_CONNECTIONS

          @serving << Thread.new { answer(socket) }
        end
      end
    rescue IOError, SystemCallError
      nil
    end

    def answer(socket)
      exchange(socket)
    rescue IOError, SystemCallError
      # The client gave up waiting and closed the connection, which is the very thing a held answer
      # is scripted to make it do.
      nil
    ensure
      socket.close unless socket.closed?
    end

    def exchange(socket)
      request = read_request(socket)
      @lock.synchronize { @received << request }

      scripted = next_response
      sleep(scripted.held_for) if scripted.held_for.positive?

      answer = JSON.generate(scripted.body)
      carried = scripted.headers.to_h.map { |name, value| "#{name}: #{value}\r\n" }.join
      socket.write(
        "HTTP/1.1 #{scripted.status} Answer\r\n" \
        "Content-Type: application/json\r\n" \
        "Content-Length: #{answer.bytesize}\r\n" \
        "#{carried}" \
        "Connection: close\r\n\r\n#{answer}"
      )
    end

    def read_request(socket)
      verb, target = read_line(socket).split(" ", 3)
      headers = {}
      MAX_HEADERS.times do
        line = read_line(socket).strip
        break if line.empty?

        name, _, value = line.partition(":")
        headers[name.strip.downcase] = value.strip
      end

      length = headers.fetch("content-length", "0").to_i
      raise IOError, "a case sent more than #{MAX_REQUEST_BODY_BYTES} bytes" if length > MAX_REQUEST_BODY_BYTES

      ReceivedRequest.new(verb, target, headers, length.positive? ? socket.read(length) : "")
    end

    def read_line(socket)
      line = socket.gets("\n", MAX_LINE_BYTES)
      raise IOError, "the connection closed mid-request" if line.nil?

      line
    end

    def next_response
      @lock.synchronize do
        scripted = @scripted[@answered]
        @answered += 1
        scripted || ScriptedResponse.new(500, { "error" => "the case scripted no answer for this request" })
      end
    end
  end

  # What every case that talks to an API is built on.
  class ApiCase < Minitest::Test
    def setup
      @api = FakeApi.new
    end

    def teardown
      @api.close
    end

    # A schedule short enough that a case spends its time on requests rather than on waiting.
    #
    # Its budget sits far above what its delays add up to, so the number of attempts a case observes
    # is the one its policy asked for rather than the one its budget allowed.
    def retries(max_attempts: 4)
      Hook0::RetryPolicy.new(
        max_attempts: max_attempts,
        initial_backoff: 0.005,
        max_backoff: 0.005,
        max_total_delay: 1.0
      )
    end

    def options(max_attempts: 4, request_timeout: 5.0, **bounds)
      Hook0::Options.new(retry_policy: retries(max_attempts: max_attempts), request_timeout: request_timeout, **bounds)
    end

    def client(chosen = options)
      Hook0::Client.new(@api.base_url, "app-123", "token-xyz", chosen)
    end

    def an_event(**overrides)
      Hook0::Event.new(
        event_type: "auth.user.create",
        payload: '{"email": "test@example.com"}',
        payload_content_type: "application/json",
        labels: { "environment" => "production" },
        **overrides
      )
    end

    def ingested(event_id)
      ScriptedResponse.new(
        201,
        { "application_id" => "app-123", "event_id" => event_id, "received_at" => "2026-01-01" }
      )
    end

    # What the API says when it refuses a request, in the shape every Hook0 failure takes.
    def refusal(status, problem, headers = {})
      ScriptedResponse.new(
        status,
        {
          "id" => problem,
          "status" => status,
          "title" => "refused",
          "detail" => "what the corpus scripted",
          "type" => "https://hook0.com/documentation/errors/#{problem}"
        },
        0.0,
        headers
      )
    end

    def already_ingested
      ScriptedResponse.new(
        409,
        {
          "id" => "EventAlreadyIngested",
          "title" => "Event already Ingested",
          "detail" => "This event was previously ingested and recorded inside Hook0 service.",
          "status" => 409,
          "type" => "https://documentation.hook0.com/problems"
        }
      )
    end

    def server_error
      ScriptedResponse.new(500, { "id" => "InternalServerError", "status" => 500 })
    end
  end

  # Where the shared contract every SDK is held to sits, from the directory this suite runs out of.
  CORPUS = File.expand_path("../../conformance", __dir__)

  # Largest document of the corpus read back. The corpus is committed, so one above this is one that
  # grew out of shape rather than one somebody meant.
  MAX_CORPUS_BYTES = 512 * 1024

  # One document of the shared contract, bounded before it is parsed.
  def self.contract(name)
    path = File.join(CORPUS, name)
    size = File.size(path)
    raise "#{path} is #{size} bytes long, above the #{MAX_CORPUS_BYTES} read back" if size > MAX_CORPUS_BYTES

    JSON.parse(File.read(path, encoding: "UTF-8"))
  end

  # The holes of the request document that the retry policy a case built its client with answers.
  #
  # Read off that policy rather than written out: a literal would agree with a client that had
  # drifted alongside this file, and it would be wrong the moment a case builds a client on another
  # policy. The two conversions are the ones the header itself is specified in — the attempts a
  # policy actually makes, which is what it asked for after its own cap, and each of its durations
  # in whole milliseconds.
  def self.stated_by(policy)
    {
      "attempts" => policy.attempts.to_s,
      "backoff_ms" => (policy.initial_backoff_in_force * 1000).round.to_s,
      "ceiling_ms" => (policy.max_backoff_in_force * 1000).round.to_s,
      "budget_ms" => (policy.max_total_delay_in_force * 1000).round.to_s
    }
  end

  # What a value of the shared contract is made of, once the holes a suite can speak for are filled
  # in.
  #
  # A value is a template: `${name}` is a hole and everything around it is literal. A hole named in
  # `bound` becomes part of the literal text around it; one that is not is a hole no suite can fill
  # without reimplementing the client it is testing, and it separates two chunks. A template whose
  # holes are all bound is therefore one chunk, and the whole value is that chunk.
  def self.template_chunks(template, bound)
    chunks = [+""]
    rest = template

    while (opened = rest.index("${"))
      closed = rest.index("}", opened)
      break if closed.nil?

      chunks.last << rest[0...opened]
      name = rest[(opened + 2)...closed]
      if bound.key?(name)
        chunks.last << bound[name]
      else
        chunks << +""
      end
      rest = rest[(closed + 1)..]
    end

    chunks.last << rest
    chunks
  end

  # Whether what arrived is what those chunks describe: the literal text in order, anchored at both
  # ends, with something non-empty standing in every hole between them.
  def self.matches_chunks?(chunks, carried)
    return carried == chunks.first if chunks.size == 1
    return false unless carried.start_with?(chunks.first)

    rest = carried[chunks.first.length..]
    chunks[1...-1].each do |chunk|
      # A hole stands before this chunk, and nothing is not something, so the search starts past
      # whatever fills it.
      found = rest.empty? ? nil : rest[1..].index(chunk)
      return false if found.nil?

      rest = rest[(1 + found + chunk.length)..]
    end

    rest.length > chunks.last.length && rest.end_with?(chunks.last)
  end

  # The counter-examples worth keeping, committed beside the properties they broke.
  #
  # One JSON value per line, so that a header carrying a comma, a newline or nothing at all is read
  # back exactly as it was written down.
  def self.corpus(name)
    path = File.expand_path("regressions/#{name}.jsonl", __dir__)
    File.readlines(path, chomp: true).reject(&:empty?).map { |line| JSON.parse(line) }
  end
end
