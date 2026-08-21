package com.hook0.kotlin

import java.io.ByteArrayOutputStream
import java.net.URI
import java.net.URLEncoder
import java.net.http.HttpClient
import java.net.http.HttpHeaders
import java.net.http.HttpRequest
import java.net.http.HttpResponse
import java.nio.ByteBuffer
import java.nio.charset.StandardCharsets
import java.time.Duration
import java.util.Locale
import java.util.concurrent.CompletableFuture
import java.util.concurrent.CompletionStage
import java.util.concurrent.Flow

/**
 * How a request reaches the API, over [HttpClient], and what a server on the other end is not
 * allowed to cost.
 *
 * Nothing here reaches for a third-party HTTP library. Everything a server controls is bounded: how
 * long one exchange may take, how large the head of an answer may be, how many header lines it may
 * carry, how long one of them may be, and how many bytes of body are read off the socket.
 *
 * Every one of those is this client's own doing, because what the runtime does is neither enough nor
 * fixed. What [HttpClient] bounds, measured rather than looked up in `TransportTest`, is the head of
 * an answer and nothing else, settled by `jdk.http.maxHeaderSize`, which is unset by default and
 * which an application can move. It bounds neither the number of header lines — several hundred are
 * read without complaint — nor the length of one of them, and it bounds the body not at all. So the
 * head is held against this client's own ceilings before a byte of body is read, and the body is
 * read through the subscriber below, which stops at its ceiling instead of at the end.
 *
 * Redirects are never followed. A redirect is a server saying where to send the credential next, and
 * this client has no way to know that somewhere else deserves it.
 *
 * @param apiUrl base API URL of a Hook0 instance, such as `https://app.hook0.com/api/v1`
 * @param token an authentication token valid for that API
 * @param options the bounds one exchange is held to
 */
class HttpTransport(
  private val apiUrl: String,
  private val token: String,
  private val options: Options = Options.defaults()
) : Transport,
  AutoCloseable {

  /** The retry policy this transport was built with, in the shape an API reads it off the wire. */
  private val clientOptions: String = clientOptions(options.retryPolicy)

  private val http: HttpClient =
    HttpClient.newBuilder()
      .connectTimeout(options.requestTimeout)
      .followRedirects(HttpClient.Redirect.NEVER)
      .build()

  override fun request(method: String, path: String, query: List<QueryParameter>, body: Any?): Answer {
    val built = built(method, path, query, body)
    return try {
      answered(http.send(built, handler()))
    } catch (interrupted: InterruptedException) {
      Thread.currentThread().interrupt()
      throw TransportException.noAnswer(
        "the attempt was interrupted before the API answered: ${interrupted.message}"
      )
    } catch (failure: Exception) {
      throw carried(failure)
    }
  }

  override suspend fun requestSuspending(
    method: String,
    path: String,
    query: List<QueryParameter>,
    body: Any?
  ): Answer {
    val built = built(method, path, query, body)
    return try {
      answered(Suspending.awaiting(http.sendAsync(built, handler())))
    } catch (failure: Exception) {
      throw carried(failure)
    }
  }

  override fun close() {
    http.close()
  }

  private fun built(method: String, path: String, query: List<QueryParameter>, body: Any?): HttpRequest {
    val target = resolved(path, query)

    val building =
      HttpRequest.newBuilder(target)
        .timeout(options.requestTimeout)
        .header("Authorization", "Bearer $token")
        .header("Accept", JSON_MEDIA_TYPE)
        .header("User-Agent", USER_AGENT)
        .header("Hook0-Client-Options", clientOptions)

    if (body == null) {
      return building
        .method(method.uppercase(Locale.ROOT), HttpRequest.BodyPublishers.noBody())
        .build()
    }
    return building
      .header("Content-Type", JSON_MEDIA_TYPE)
      .method(
        method.uppercase(Locale.ROOT),
        HttpRequest.BodyPublishers.ofString(Json.write(body), StandardCharsets.UTF_8)
      )
      .build()
  }

  /** Where a request lands: a path of its own replaces the base's, a relative one extends it. */
  private fun resolved(path: String, query: List<QueryParameter>): URI {
    val target =
      try {
        val written = if (apiUrl.endsWith("/")) apiUrl else "$apiUrl/"
        URI.create(written).resolve(path)
      } catch (malformed: IllegalArgumentException) {
        throw TransportException.unusableApiUrl(malformed.message ?: "the API URL is not a URL")
      }

    val scheme = target.scheme?.lowercase(Locale.ROOT)
    val host = target.host
    if (scheme !in SCHEMES || host == null || host.isEmpty()) {
      throw TransportException.unusableApiUrl(
        "`$target` is not somewhere this transport can send a request"
      )
    }
    if (query.isEmpty()) {
      return target
    }

    val written = StringBuilder(target.toString())
    val raw = target.rawQuery
    written.append(if (raw == null || raw.isEmpty()) '?' else '&')
    for ((index, parameter) in query.withIndex()) {
      if (index > 0) {
        written.append('&')
      }
      written.append(URLEncoder.encode(parameter.name, StandardCharsets.UTF_8))
      written.append('=')
      written.append(URLEncoder.encode(parameter.value, StandardCharsets.UTF_8))
    }
    return try {
      URI.create(written.toString())
    } catch (malformed: IllegalArgumentException) {
      throw TransportException.unusableApiUrl(malformed.message ?: "the query is not one")
    }
  }

  private fun handler(): HttpResponse.BodyHandler<String> = HttpResponse.BodyHandler { info ->
    BoundedBody(options.maxResponseBytes, refusedHead(info.headers()))
  }

  /**
   * What is wrong with the head of an answer, or nothing when it is inside every bound.
   *
   * The count and the length of a line are looked at first, because either one refuses on the line
   * that crosses it rather than after the whole head has been read; the total is what actually
   * bounds what a head can cost, since a count and a size per line multiply.
   */
  private fun refusedHead(headers: HttpHeaders): TransportException? {
    var held = 0
    var spent = 0L
    for ((name, values) in headers.map()) {
      for (value in values) {
        held++
        if (held > options.maxResponseHeaders) {
          return TransportException.answerAboveABound(
            "the API answered more than the ${options.maxResponseHeaders} header lines read at most"
          )
        }
        // The name, the `: ` between it and the value, the value, and the break that ends the line,
        // which is what one header actually costs on the wire.
        val line = name.length.toLong() + value.length + LINE_OVERHEAD
        if (line > options.maxHeaderBytes) {
          return TransportException.answerAboveABound(
            "the API answered a `${name.lowercase(Locale.ROOT)}` header above the " +
              "${options.maxHeaderBytes} bytes read at most"
          )
        }
        spent += line
        if (spent > options.maxHeadBytes) {
          return TransportException.answerAboveABound(
            "the API answered a head above the ${options.maxHeadBytes} bytes read at most"
          )
        }
      }
    }
    return null
  }

  /**
   * The body of an answer, up to what this transport agrees to hold.
   *
   * The subscriber cancels at the ceiling rather than collecting to it and checking afterwards, so a
   * server that streams without end costs one buffer's worth of memory and not one body's worth.
   */
  private class BoundedBody(private val maxBytes: Long, private val refused: TransportException?) :
    HttpResponse.BodySubscriber<String> {

    private val held = ByteArrayOutputStream()
    private val text = CompletableFuture<String>()
    private var subscription: Flow.Subscription? = null

    override fun getBody(): CompletionStage<String> = text

    override fun onSubscribe(subscription: Flow.Subscription) {
      this.subscription = subscription
      val refusal = refused
      if (refusal != null) {
        subscription.cancel()
        text.completeExceptionally(refusal)
        return
      }
      subscription.request(Long.MAX_VALUE)
    }

    override fun onNext(item: MutableList<ByteBuffer>) {
      for (buffer in item) {
        val length = buffer.remaining()
        if (held.size().toLong() + length > maxBytes) {
          subscription?.cancel()
          text.completeExceptionally(
            TransportException.answerAboveABound(
              "the API answered more than the $maxBytes bytes read at most"
            )
          )
          return
        }
        val chunk = ByteArray(length)
        buffer.get(chunk)
        held.writeBytes(chunk)
      }
    }

    override fun onError(failure: Throwable) {
      text.completeExceptionally(failure)
    }

    override fun onComplete() {
      text.complete(held.toString(StandardCharsets.UTF_8))
    }
  }

  companion object {
    /** What a request body says it carries, and what an answer is asked for in. */
    const val JSON_MEDIA_TYPE = "application/json"

    /**
     * Which release of this artefact is talking to the API.
     *
     * A jar carries no build file to read the number back out of at runtime, so it is written here;
     * the conformance suite holds it against the version `pom.xml` publishes, so the two cannot
     * drift apart.
     */
    internal const val VERSION = "1.1.0"

    /** The schemes this transport reaches. */
    private val SCHEMES = listOf("http", "https")

    /**
     * What one header line costs beyond its name and its value: the `: ` between them and the break
     * after.
     */
    private const val LINE_OVERHEAD = 4

    /**
     * Longest each part this client composes its `User-Agent` out of may be, in characters.
     *
     * The runtime and the operating system are described by the platform rather than by this
     * artefact, so their length is not this artefact's to guarantee: they are cut here so that the
     * header cannot grow with whatever the platform feels like saying. Every part is also stripped
     * of anything the grammar of the header uses as punctuation, so a platform cannot forge a shape
     * it does not have.
     */
    private const val MAX_USER_AGENT_PART_CHARS = 64

    /** Which SDK, at which version, on which runtime and operating system, is talking to the API. */
    private val USER_AGENT = userAgent()

    /** How [USER_AGENT] is put together, out of what this artefact knows and what the platform says of itself. */
    private fun userAgent(): String {
      val runtime = clipped("kotlin ${KotlinVersion.CURRENT}")
      val os = clipped("${System.getProperty("os.name")} ${System.getProperty("os.arch")}")
      return "hook0-client-kotlin/${clipped(VERSION)} ($runtime; $os)"
    }

    /**
     * One part of the `User-Agent`, with everything the header's own grammar uses taken out of it
     * and cut to [MAX_USER_AGENT_PART_CHARS].
     */
    private fun clipped(part: String): String =
      part.filter { it in ' '..'~' && it != '(' && it != ')' && it != ';' }.take(MAX_USER_AGENT_PART_CHARS)

    /**
     * How the retry policy of a transport is put together for the wire.
     *
     * The grammar is the one `X-Hook0-Signature` already uses — parts joined by `,`, each cut at its
     * first `=` — so this header costs no parser that is not written twice over already.
     *
     * What is stated is the policy in force, not what a send went on to do with it: a policy
     * allowing one attempt still states its delays, because they are what it holds, and an API
     * reading `attempts=1` already knows none of them will be waited. In force is also after this
     * client's own clamps — [RetryPolicy.attempts] rather than [RetryPolicy.maxAttempts] — since the
     * capped number is the one the API's traffic will show, and a thousand would send a reader
     * looking for a burst that cannot happen.
     */
    private fun clientOptions(policy: RetryPolicy): String {
      val backoff = policy.initialBackoffMillis
      val ceiling = policy.maxBackoffMillis
      val budget = policy.maxTotalDelayMillis
      return "attempts=${policy.attempts()},backoff=$backoff,ceiling=$ceiling,budget=$budget"
    }

    private fun answered(answer: HttpResponse<String>): Answer {
      val headers = LinkedHashMap<String, String>()
      for ((name, values) in answer.headers().map()) {
        if (values.isNotEmpty()) {
          headers[name.lowercase(Locale.ROOT)] = values[values.size - 1].trim()
        }
      }
      return Answer(answer.statusCode(), headers.toMap(), answer.body())
    }

    /**
     * The failure this transport reports for whatever the runtime reported, read by its cause and
     * not by its type.
     */
    private fun carried(failure: Throwable): TransportException {
      var walked: Throwable? = failure
      var depth = 0
      while (walked != null && depth < MAX_WRAPPERS) {
        if (walked is TransportException) {
          return walked
        }
        walked = walked.cause
        depth++
      }
      return TransportException.noAnswer(failure.message ?: failure.javaClass.simpleName)
    }

    /** How many wrappers one failure is unwrapped through before the walk gives up. */
    private const val MAX_WRAPPERS = 8
  }
}
