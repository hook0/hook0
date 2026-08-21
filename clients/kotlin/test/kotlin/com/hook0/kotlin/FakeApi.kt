package com.hook0.kotlin

import java.io.IOException
import java.io.InputStream
import java.net.InetAddress
import java.net.ServerSocket
import java.net.Socket
import java.nio.charset.StandardCharsets
import java.time.Duration
import java.util.Locale
import java.util.concurrent.CopyOnWriteArrayList
import java.util.concurrent.atomic.AtomicInteger

/**
 * A Hook0 API on a loopback port, and what every case that talks to one is written against.
 *
 * Every case goes over a real socket: the request the client builds, the headers it sets, the way it
 * reads an answer and the way it gives up on one are all the real ones. Nothing here stands in for a
 * part of the client, so a case that passes says the client works rather than that it was called.
 *
 * Plain [ServerSocket], speaking as much HTTP/1.1 as one exchange needs. Everything it reads is
 * bounded: a suite that hangs has to fail rather than hold a pipeline until the runner gives up on
 * it.
 */
class FakeApi : AutoCloseable {

  /** What the API answers to one request, in the order the case scripted it. */
  data class Scripted(
    val status: Int,
    val body: String,
    val heldFor: Duration = Duration.ZERO,
    val headers: Map<String, String> = emptyMap()
  ) {
    companion object {
      fun of(status: Int, body: Any?, headers: Map<String, String> = emptyMap()): Scripted =
        Scripted(status, Json.write(body), Duration.ZERO, headers)
    }
  }

  /** A request the API received, in the order it received it. */
  data class Received(val verb: String, val target: String, val headers: Map<String, String>, val body: String) {
    fun json(): Any? = Json.parse(body)
  }

  private val server =
    ServerSocket(0, MAX_CONNECTIONS, InetAddress.getByName("127.0.0.1"))
  private val received = CopyOnWriteArrayList<Received>()
  private val scripted = CopyOnWriteArrayList<Scripted>()
  private val serving = CopyOnWriteArrayList<Thread>()
  private val answered = AtomicInteger()
  private val accepting = Thread(::serve, "fake-api").apply { isDaemon = true }

  init {
    accepting.start()
  }

  /** Where the client reaches this API. */
  fun baseUrl(): String = "http://127.0.0.1:${server.localPort}"

  /** Queues the answers the case expects the client to draw, in order. */
  fun willAnswer(vararg answers: Scripted) {
    scripted.addAll(answers.toList())
  }

  /** Every request this API received, in the order it received them. */
  fun received(): List<Received> = received.toList()

  override fun close() {
    try {
      server.close()
    } catch (closed: IOException) {
      // The port is already gone, which is what closing it was for.
    }
    for (thread in serving) {
      join(thread)
    }
    join(accepting)
  }

  private fun serve() {
    while (!server.isClosed) {
      val socket =
        try {
          server.accept()
        } catch (closed: IOException) {
          return
        }
      check(serving.size < MAX_CONNECTIONS) {
        "a case opened more than $MAX_CONNECTIONS connections"
      }
      val exchange = Thread({ answer(socket) }, "fake-api-exchange").apply { isDaemon = true }
      serving.add(exchange)
      exchange.start()
    }
  }

  private fun answer(socket: Socket) {
    try {
      socket.use { open ->
        open.soTimeout = TIMEOUT.toMillis().toInt()
        exchange(open)
      }
    } catch (gaveUp: IOException) {
      // The client gave up waiting and closed the connection, which is the very thing an answer a
      // case asks the API to sit on is scripted to make it do.
    }
  }

  private fun exchange(socket: Socket) {
    val request = readRequest(socket.getInputStream())
    received.add(request)

    val answer = next()
    if (!answer.heldFor.isZero) {
      try {
        Thread.sleep(answer.heldFor.toMillis())
      } catch (interrupted: InterruptedException) {
        Thread.currentThread().interrupt()
        return
      }
    }

    val body = answer.body.toByteArray(StandardCharsets.UTF_8)
    val head = StringBuilder()
    head.append("HTTP/1.1 ").append(answer.status).append(" Answer\r\n")
    head.append("Content-Type: application/json\r\n")
    head.append("Content-Length: ").append(body.size).append("\r\n")
    for ((name, value) in answer.headers) {
      head.append(name).append(": ").append(value).append("\r\n")
    }
    head.append("Connection: close\r\n\r\n")

    val writing = socket.getOutputStream()
    writing.write(head.toString().toByteArray(StandardCharsets.ISO_8859_1))
    writing.write(body)
    writing.flush()
  }

  private fun next(): Scripted {
    val at = answered.getAndIncrement()
    if (at < scripted.size) {
      return scripted[at]
    }
    return Scripted.of(500, mapOf("error" to "the case scripted no answer for this request"))
  }

  companion object {
    /** No case talks to anything but a loopback socket, so none of them has a reason to take this long. */
    val TIMEOUT: Duration = Duration.ofSeconds(20)

    /** No request a case makes is anywhere near this large; the cap bounds what one connection reads. */
    private const val MAX_REQUEST_BODY_BYTES = 64 * 1024

    /** Longest request line or header line the server reads. */
    private const val MAX_LINE_BYTES = 8 * 1024

    /** Most header lines the server reads out of one request. */
    private const val MAX_HEADERS = 64

    /** Most connections one case opens, which bounds what the server holds at once. */
    private const val MAX_CONNECTIONS = 64

    private fun join(thread: Thread) {
      try {
        thread.join(TIMEOUT.toMillis())
      } catch (interrupted: InterruptedException) {
        Thread.currentThread().interrupt()
      }
    }

    private fun readRequest(reading: InputStream): Received {
      val line = readLine(reading).split(" ", limit = 3)
      val headers = LinkedHashMap<String, String>()
      for (held in 0 until MAX_HEADERS) {
        val header = readLine(reading).trim()
        if (header.isEmpty()) {
          break
        }
        val at = header.indexOf(':')
        if (at > 0) {
          headers[header.substring(0, at).trim().lowercase(Locale.ROOT)] =
            header.substring(at + 1).trim()
        }
      }

      val length = (headers["content-length"] ?: "0").toInt()
      if (length > MAX_REQUEST_BODY_BYTES) {
        throw IOException("a case sent more than $MAX_REQUEST_BODY_BYTES bytes")
      }
      val body = reading.readNBytes(length)
      return Received(
        line.getOrElse(0) { "" },
        line.getOrElse(1) { "" },
        headers.toMap(),
        String(body, StandardCharsets.UTF_8)
      )
    }

    private fun readLine(reading: InputStream): String {
      val held = ArrayList<Byte>()
      while (held.size < MAX_LINE_BYTES) {
        val one = reading.read()
        if (one < 0) {
          if (held.isEmpty()) {
            throw IOException("the connection closed mid-request")
          }
          break
        }
        if (one == '\n'.code) {
          break
        }
        held.add(one.toByte())
      }
      return String(held.toByteArray(), StandardCharsets.ISO_8859_1)
    }
  }
}
