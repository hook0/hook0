package com.hook0.client;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.net.InetAddress;
import java.net.ServerSocket;
import java.net.Socket;
import java.nio.charset.StandardCharsets;
import java.time.Duration;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.concurrent.atomic.AtomicInteger;

/**
 * A Hook0 API on a loopback port, and what every case that talks to one is written against.
 *
 * <p>Every case goes over a real socket: the request the client builds, the headers it sets, the way it reads an answer
 * and the way it gives up on one are all the real ones. Nothing here stands in for a part of the client, so a case that
 * passes says the client works rather than that it was called.
 *
 * <p>Plain {@link ServerSocket}, speaking as much HTTP/1.1 as one exchange needs. Everything it reads is bounded: a
 * suite that hangs has to fail rather than hold a pipeline until the runner gives up on it.
 */
final class FakeApi implements AutoCloseable {

  /** No case talks to anything but a loopback socket, so none of them has a reason to take this long. */
  static final Duration TIMEOUT = Duration.ofSeconds(20);

  /** No request a case makes is anywhere near this large; the cap bounds what one connection reads. */
  private static final int MAX_REQUEST_BODY_BYTES = 64 * 1024;

  /** Longest request line or header line the server reads. */
  private static final int MAX_LINE_BYTES = 8 * 1024;

  /** Most header lines the server reads out of one request. */
  private static final int MAX_HEADERS = 64;

  /** Most connections one case opens, which bounds what the server holds at once. */
  private static final int MAX_CONNECTIONS = 64;

  /** What the API answers to one request, in the order the case scripted it. */
  record Scripted(int status, String body, Duration heldFor, Map<String, String> headers) {

    static Scripted of(int status, Object body) {
      return new Scripted(status, Json.write(body), Duration.ZERO, Map.of());
    }

    static Scripted of(int status, Object body, Duration heldFor) {
      return new Scripted(status, Json.write(body), heldFor, Map.of());
    }

    static Scripted of(int status, Object body, Map<String, String> headers) {
      return new Scripted(status, Json.write(body), Duration.ZERO, headers);
    }
  }

  /** A request the API received, in the order it received it. */
  record Received(String verb, String target, Map<String, String> headers, String body) {

    Object json() {
      return Json.parse(body);
    }
  }

  private final ServerSocket server;
  private final List<Received> received = new CopyOnWriteArrayList<>();
  private final List<Scripted> scripted = new CopyOnWriteArrayList<>();
  private final List<Thread> serving = new CopyOnWriteArrayList<>();
  private final AtomicInteger answered = new AtomicInteger();
  private final Thread accepting;

  FakeApi() {
    try {
      server = new ServerSocket(0, MAX_CONNECTIONS, InetAddress.getByName("127.0.0.1"));
    } catch (IOException unusable) {
      throw new IllegalStateException("no loopback port could be listened on", unusable);
    }
    accepting = new Thread(this::serve, "fake-api");
    accepting.setDaemon(true);
    accepting.start();
  }

  /** Where the client reaches this API. */
  String baseUrl() {
    return "http://127.0.0.1:" + server.getLocalPort();
  }

  /** Queues the answers the case expects the client to draw, in order. */
  void willAnswer(Scripted... answers) {
    scripted.addAll(List.of(answers));
  }

  /** Every request this API received, in the order it received them. */
  List<Received> received() {
    return List.copyOf(received);
  }

  @Override
  public void close() {
    try {
      server.close();
    } catch (IOException ignored) {
      // The port is already gone, which is what closing it was for.
    }
    for (Thread thread : serving) {
      join(thread);
    }
    join(accepting);
  }

  private static void join(Thread thread) {
    try {
      thread.join(TIMEOUT.toMillis());
    } catch (InterruptedException interrupted) {
      Thread.currentThread().interrupt();
    }
  }

  private void serve() {
    while (!server.isClosed()) {
      Socket socket;
      try {
        socket = server.accept();
      } catch (IOException closed) {
        return;
      }
      if (serving.size() >= MAX_CONNECTIONS) {
        throw new IllegalStateException("a case opened more than " + MAX_CONNECTIONS + " connections");
      }
      Thread exchange = new Thread(() -> answer(socket), "fake-api-exchange");
      exchange.setDaemon(true);
      serving.add(exchange);
      exchange.start();
    }
  }

  private void answer(Socket socket) {
    try (Socket open = socket) {
      open.setSoTimeout((int) TIMEOUT.toMillis());
      exchange(open);
    } catch (IOException gaveUp) {
      // The client gave up waiting and closed the connection, which is the very thing an answer a
      // case asks the API to sit on is scripted to make it do.
    }
  }

  private void exchange(Socket socket) throws IOException {
    InputStream reading = socket.getInputStream();
    Received request = readRequest(reading);
    received.add(request);

    Scripted answer = next();
    if (!answer.heldFor().isZero()) {
      try {
        Thread.sleep(answer.heldFor().toMillis());
      } catch (InterruptedException interrupted) {
        Thread.currentThread().interrupt();
        return;
      }
    }

    byte[] body = answer.body().getBytes(StandardCharsets.UTF_8);
    StringBuilder head = new StringBuilder();
    head.append("HTTP/1.1 ").append(answer.status()).append(" Answer\r\n");
    head.append("Content-Type: application/json\r\n");
    head.append("Content-Length: ").append(body.length).append("\r\n");
    for (Map.Entry<String, String> header : answer.headers().entrySet()) {
      head.append(header.getKey()).append(": ").append(header.getValue()).append("\r\n");
    }
    head.append("Connection: close\r\n\r\n");

    OutputStream writing = socket.getOutputStream();
    writing.write(head.toString().getBytes(StandardCharsets.ISO_8859_1));
    writing.write(body);
    writing.flush();
  }

  private static Received readRequest(InputStream reading) throws IOException {
    String[] line = readLine(reading).split(" ", 3);
    Map<String, String> headers = new LinkedHashMap<>();
    for (int held = 0; held < MAX_HEADERS; held++) {
      String header = readLine(reading).strip();
      if (header.isEmpty()) {
        break;
      }
      int at = header.indexOf(':');
      if (at > 0) {
        headers.put(header.substring(0, at).strip().toLowerCase(Locale.ROOT), header.substring(at + 1).strip());
      }
    }

    int length = Integer.parseInt(headers.getOrDefault("content-length", "0"));
    if (length > MAX_REQUEST_BODY_BYTES) {
      throw new IOException("a case sent more than " + MAX_REQUEST_BODY_BYTES + " bytes");
    }
    byte[] body = reading.readNBytes(length);
    return new Received(
        line.length > 0 ? line[0] : "",
        line.length > 1 ? line[1] : "",
        Map.copyOf(headers),
        new String(body, StandardCharsets.UTF_8));
  }

  private static String readLine(InputStream reading) throws IOException {
    List<Byte> held = new ArrayList<>();
    while (held.size() < MAX_LINE_BYTES) {
      int one = reading.read();
      if (one < 0) {
        if (held.isEmpty()) {
          throw new IOException("the connection closed mid-request");
        }
        break;
      }
      if (one == '\n') {
        break;
      }
      held.add(Byte.valueOf((byte) one));
    }

    byte[] line = new byte[held.size()];
    for (int index = 0; index < line.length; index++) {
      line[index] = held.get(index).byteValue();
    }
    return new String(line, StandardCharsets.ISO_8859_1);
  }

  private Scripted next() {
    int at = answered.getAndIncrement();
    if (at < scripted.size()) {
      return scripted.get(at);
    }
    return Scripted.of(500, Map.of("error", "the case scripted no answer for this request"));
  }
}
