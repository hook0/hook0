package com.hook0.client;

import java.io.ByteArrayOutputStream;
import java.net.URI;
import java.net.URLEncoder;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionException;
import java.util.concurrent.CompletionStage;
import java.util.concurrent.Flow;

/**
 * How a request reaches the API, over {@link HttpClient}, and what a server on the other end is not allowed to cost.
 *
 * <p>Nothing here reaches for a third-party HTTP library. Everything a server controls is bounded: how long one
 * exchange may take, how large the head of an answer may be, how many header lines it may carry, how long one of them
 * may be, and how many bytes of body are read off the socket.
 *
 * <p>Every one of those is this client's own doing, because what the runtime does is neither enough nor fixed. What
 * {@link HttpClient} bounds, measured rather than looked up in {@code TransportTest}, is the head of an answer and
 * nothing else: 393216 bytes, settled by {@code jdk.http.maxHeaderSize}, which is unset by default and which an
 * application can move. It bounds neither the number of header lines — several hundred are read without complaint —
 * nor the length of one of them, and it bounds the body not at all: {@code BodyHandlers.ofString()} buffers whatever a
 * server cares to send. So the head is held against this client's own ceilings before a byte of body is read, and the
 * body is read through the subscriber below, which stops at its ceiling instead of at the end.
 *
 * <p>The one thing left to the runtime is a head above <em>its</em> ceiling, which it refuses before this client is
 * consulted. That is why the ceiling here sits far under it: everything this client could be responsible for refusing
 * is refused here, and a head large enough for the runtime to intervene is one this client would have refused anyway.
 *
 * <p>Redirects are never followed. A redirect is a server saying where to send the credential next, and this client
 * has no way to know that somewhere else deserves it.
 */
public final class HttpTransport implements Transport, AutoCloseable {

  /** What a request body says it carries, and what an answer is asked for in. */
  public static final String JSON_MEDIA_TYPE = "application/json";

  /**
   * Which release of this artefact is talking to the API.
   *
   * <p>A jar carries no build file to read the number back out of at runtime, so it is written here; the conformance
   * suite holds it against the version {@code pom.xml} publishes, so the two cannot drift apart.
   */
  static final String VERSION = "1.1.0";

  /** The schemes this transport reaches. */
  private static final List<String> SCHEMES = List.of("http", "https");

  /** What one header line costs beyond its name and its value: the `: ` between them and the break after. */
  private static final int LINE_OVERHEAD = 4;

  /**
   * Longest each part this client composes its {@code User-Agent} out of may be, in characters.
   *
   * <p>The runtime and the operating system are described by the platform rather than by this artefact, so their
   * length is not this artefact's to guarantee: they are cut here so that the header cannot grow with whatever the
   * platform feels like saying. Every part is also stripped of anything the grammar of the header uses as punctuation,
   * so a platform cannot forge a shape it does not have.
   */
  private static final int MAX_USER_AGENT_PART_CHARS = 64;

  /** Which SDK, at which version, on which runtime and operating system, is talking to the API. */
  private static final String USER_AGENT = userAgent();

  private final String apiUrl;
  private final String token;
  private final Options options;
  private final HttpClient http;

  /**
   * Builds one on the API it reaches and the bounds it is held to.
   *
   * @param apiUrl base API URL of a Hook0 instance, such as {@code https://app.hook0.com/api/v1}
   * @param token an authentication token valid for that API
   * @param options the bounds one exchange is held to
   */
  public HttpTransport(String apiUrl, String token, Options options) {
    this.apiUrl = apiUrl;
    this.token = token;
    this.options = options;
    this.http =
        HttpClient.newBuilder()
            .connectTimeout(options.requestTimeout())
            .followRedirects(HttpClient.Redirect.NEVER)
            .build();
  }

  @Override
  public Answer request(String method, String path, List<QueryParameter> query, Object body) {
    HttpRequest built = built(method, path, query, body);
    try {
      return answered(http.send(built, handler()));
    } catch (InterruptedException interrupted) {
      Thread.currentThread().interrupt();
      throw TransportException.noAnswer("the attempt was interrupted before the API answered");
    } catch (Exception failure) {
      throw carried(failure);
    }
  }

  @Override
  public CompletableFuture<Answer> requestAsync(String method, String path, List<QueryParameter> query, Object body) {
    HttpRequest built;
    try {
      built = built(method, path, query, body);
    } catch (TransportException unusable) {
      return CompletableFuture.failedFuture(unusable);
    }

    return http.sendAsync(built, handler())
        .handle(
            (answer, failure) -> {
              if (failure != null) {
                throw new CompletionException(carried(failure));
              }
              return answered(answer);
            });
  }

  @Override
  public void close() {
    http.close();
  }

  /** The failure this transport reports for whatever the JDK reported, read by its cause and not by its type. */
  private static TransportException carried(Throwable failure) {
    Throwable walked = failure;
    for (int depth = 0; walked != null && depth < 8; depth++) {
      if (walked instanceof TransportException already) {
        return already;
      }
      walked = walked.getCause();
    }
    String detail = failure.getMessage();
    return TransportException.noAnswer(detail == null ? failure.getClass().getSimpleName() : detail);
  }

  /** How {@link #USER_AGENT} is put together, out of what this artefact knows and what the platform says of itself. */
  private static String userAgent() {
    String runtime = clipped("java " + System.getProperty("java.version"));
    String os = clipped(System.getProperty("os.name") + " " + System.getProperty("os.arch"));
    return "hook0-client-java/" + clipped(VERSION) + " (" + runtime + "; " + os + ")";
  }

  /**
   * One part of the {@code User-Agent}, with everything the header's own grammar uses taken out of it and cut to
   * {@link #MAX_USER_AGENT_PART_CHARS}.
   */
  private static String clipped(String part) {
    StringBuilder kept = new StringBuilder();
    for (int index = 0; index < part.length() && kept.length() < MAX_USER_AGENT_PART_CHARS; index++) {
      char one = part.charAt(index);
      if (one >= ' ' && one <= '~' && one != '(' && one != ')' && one != ';') {
        kept.append(one);
      }
    }
    return kept.toString();
  }

  private HttpRequest built(String method, String path, List<QueryParameter> query, Object body) {
    URI target = resolved(path, query);

    HttpRequest.Builder building =
        HttpRequest.newBuilder(target)
            .timeout(options.requestTimeout())
            .header("Authorization", "Bearer " + token)
            .header("Accept", JSON_MEDIA_TYPE)
            .header("User-Agent", USER_AGENT);

    if (body == null) {
      return building.method(method.toUpperCase(Locale.ROOT), HttpRequest.BodyPublishers.noBody()).build();
    }
    return building
        .header("Content-Type", JSON_MEDIA_TYPE)
        .method(
            method.toUpperCase(Locale.ROOT),
            HttpRequest.BodyPublishers.ofString(Json.write(body), StandardCharsets.UTF_8))
        .build();
  }

  /** Where a request lands: a path of its own replaces the base's, a relative one extends it. */
  private URI resolved(String path, List<QueryParameter> query) {
    URI base;
    URI target;
    try {
      String written = apiUrl == null ? "" : apiUrl;
      base = URI.create(written.endsWith("/") ? written : written + "/");
      target = base.resolve(path == null ? "" : path);
    } catch (IllegalArgumentException malformed) {
      throw TransportException.unusableApiUrl(malformed.getMessage());
    }
    if (!SCHEMES.contains(String.valueOf(target.getScheme()).toLowerCase(Locale.ROOT))
        || target.getHost() == null
        || target.getHost().isEmpty()) {
      throw TransportException.unusableApiUrl(
          "`" + target + "` is not somewhere this transport can send a request");
    }
    if (query.isEmpty()) {
      return target;
    }

    StringBuilder written = new StringBuilder(target.toString());
    written.append(target.getRawQuery() == null || target.getRawQuery().isEmpty() ? '?' : '&');
    for (int index = 0; index < query.size(); index++) {
      QueryParameter parameter = query.get(index);
      if (index > 0) {
        written.append('&');
      }
      written.append(URLEncoder.encode(parameter.name(), StandardCharsets.UTF_8));
      written.append('=');
      written.append(URLEncoder.encode(parameter.value() == null ? "" : parameter.value(), StandardCharsets.UTF_8));
    }
    try {
      return URI.create(written.toString());
    } catch (IllegalArgumentException malformed) {
      throw TransportException.unusableApiUrl(malformed.getMessage());
    }
  }

  private HttpResponse.BodyHandler<String> handler() {
    return info -> new BoundedBody(options.maxResponseBytes(), refusedHead(info.headers()));
  }

  /**
   * What is wrong with the head of an answer, or nothing when it is inside every bound.
   *
   * <p>The count and the length of a line are looked at first, because either one refuses on the line that crosses it
   * rather than after the whole head has been read; the total is what actually bounds what a head can cost, since a
   * count and a size per line multiply.
   */
  private TransportException refusedHead(java.net.http.HttpHeaders headers) {
    int held = 0;
    long spent = 0;
    for (Map.Entry<String, List<String>> header : headers.map().entrySet()) {
      for (String value : header.getValue()) {
        held++;
        if (held > options.maxResponseHeaders()) {
          return TransportException.answerAboveABound(
              "the API answered more than the " + options.maxResponseHeaders() + " header lines read at most");
        }
        // The name, the `: ` between it and the value, the value, and the break that ends the line,
        // which is what one header actually costs on the wire.
        long line = (long) header.getKey().length() + value.length() + LINE_OVERHEAD;
        if (line > options.maxHeaderBytes()) {
          return TransportException.answerAboveABound(
              "the API answered a `"
                  + header.getKey().toLowerCase(Locale.ROOT)
                  + "` header above the "
                  + options.maxHeaderBytes()
                  + " bytes read at most");
        }
        spent += line;
        if (spent > options.maxHeadBytes()) {
          return TransportException.answerAboveABound(
              "the API answered a head above the " + options.maxHeadBytes() + " bytes read at most");
        }
      }
    }
    return null;
  }

  private static Answer answered(HttpResponse<String> answer) {
    Map<String, String> headers = new LinkedHashMap<>();
    for (Map.Entry<String, List<String>> header : answer.headers().map().entrySet()) {
      List<String> values = header.getValue();
      if (!values.isEmpty()) {
        headers.put(header.getKey().toLowerCase(Locale.ROOT), values.get(values.size() - 1).strip());
      }
    }
    return new Answer(answer.statusCode(), Map.copyOf(headers), answer.body());
  }

  /**
   * The body of an answer, up to what this transport agrees to hold.
   *
   * <p>The subscriber cancels at the ceiling rather than collecting to it and checking afterwards, so a server that
   * streams without end costs one buffer's worth of memory and not one body's worth.
   */
  private static final class BoundedBody implements HttpResponse.BodySubscriber<String> {

    private final long maxBytes;
    private final TransportException refused;
    private final ByteArrayOutputStream held = new ByteArrayOutputStream();
    private final CompletableFuture<String> text = new CompletableFuture<>();
    private Flow.Subscription subscription;

    BoundedBody(long maxBytes, TransportException refused) {
      this.maxBytes = maxBytes;
      this.refused = refused;
    }

    @Override
    public CompletionStage<String> getBody() {
      return text;
    }

    @Override
    public void onSubscribe(Flow.Subscription subscription) {
      this.subscription = subscription;
      if (refused != null) {
        subscription.cancel();
        text.completeExceptionally(refused);
        return;
      }
      subscription.request(Long.MAX_VALUE);
    }

    @Override
    public void onNext(List<ByteBuffer> buffers) {
      for (ByteBuffer buffer : buffers) {
        int length = buffer.remaining();
        if ((long) held.size() + length > maxBytes) {
          subscription.cancel();
          text.completeExceptionally(
              TransportException.answerAboveABound(
                  "the API answered more than the " + maxBytes + " bytes read at most"));
          return;
        }
        byte[] chunk = new byte[length];
        buffer.get(chunk);
        held.writeBytes(chunk);
      }
    }

    @Override
    public void onError(Throwable failure) {
      text.completeExceptionally(failure);
    }

    @Override
    public void onComplete() {
      text.complete(held.toString(StandardCharsets.UTF_8));
    }
  }
}
