package com.hook0.kotlin

import java.time.Duration
import java.util.UUID
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicReference
import kotlin.coroutines.Continuation
import kotlin.coroutines.CoroutineContext
import kotlin.coroutines.EmptyCoroutineContext
import kotlin.coroutines.startCoroutine

/**
 * The two surfaces this client offers, as one thing a case can be written against.
 *
 * Every case that sends an event runs twice, once through each. That is the whole point of the type:
 * a defect fixed in one surface and left standing in the other is exactly what having two costs, and
 * the only way to find one is to put the same case through both.
 *
 * The suspending one is driven by [awaiting], which starts a coroutine and waits for it on the
 * calling thread. That is written out of the standard library on purpose, and it is the same reason
 * the SDK itself names no coroutine library: if the suite needed one to drive the suspending
 * surface, the surface would not really work without one.
 */
enum class Surface {
  BLOCKING {
    override fun send(client: Hook0Client, event: Event): UUID = client.sendEvent(event)

    override fun upsertEventTypes(client: Hook0Client, eventTypes: List<String>): List<String> =
      client.upsertEventTypes(eventTypes)
  },
  SUSPENDING {
    override fun send(client: Hook0Client, event: Event): UUID = awaiting { client.sendEventSuspending(event) }

    override fun upsertEventTypes(client: Hook0Client, eventTypes: List<String>): List<String> =
      awaiting { client.upsertEventTypesSuspending(eventTypes) }
  };

  abstract fun send(client: Hook0Client, event: Event): UUID

  abstract fun upsertEventTypes(client: Hook0Client, eventTypes: List<String>): List<String>

  companion object {
    /** Longest a suspending call is given before the suite reads it as stuck rather than slow. */
    private val PATIENCE: Duration = Duration.ofSeconds(60)

    /**
     * Runs a suspending call to its end and answers what it answered, raising what it raised.
     *
     * @param block the suspending call to drive
     * @param T what it answers
     * @return what it answered
     */
    fun <T> awaiting(block: suspend () -> T): T {
      val outcome = AtomicReference<Result<T>>()
      val done = CountDownLatch(1)

      block.startCoroutine(
        object : Continuation<T> {
          override val context: CoroutineContext = EmptyCoroutineContext

          override fun resumeWith(result: Result<T>) {
            outcome.set(result)
            done.countDown()
          }
        }
      )

      check(done.await(PATIENCE.toMillis(), TimeUnit.MILLISECONDS)) {
        "a suspending call did not finish inside the ${PATIENCE.toSeconds()} seconds it is given"
      }
      return outcome.get().getOrThrow()
    }
  }
}
