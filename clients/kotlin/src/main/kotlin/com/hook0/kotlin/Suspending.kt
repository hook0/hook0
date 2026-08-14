package com.hook0.kotlin

import java.util.concurrent.CompletableFuture
import java.util.concurrent.CompletionException
import java.util.concurrent.TimeUnit
import kotlin.coroutines.resume
import kotlin.coroutines.resumeWithException
import kotlin.coroutines.suspendCoroutine

/**
 * The two things a suspending call needs, built out of the standard library alone.
 *
 * This is why the suspending surface costs an application nothing. `suspend` is a feature of the
 * language, and the machinery under it — [suspendCoroutine], `Continuation`, `resume` — is part of
 * `kotlin-stdlib`; `kotlinx.coroutines` is a *library* on top of that, and a caller already has one
 * of its own, at a version it pinned. An SDK that named a version too would be asking every
 * application embedding it to reconcile the two, which is exactly the dependency conflict this
 * artefact is written to avoid. A caller suspends inside whichever runtime it brought.
 *
 * What that costs, stated plainly: a coroutine cancelled while one of these is waiting is not
 * propagated to the request, because a cancellable continuation is one of the things that live in
 * the library rather than in the language. The request is still bounded — by the timeout every
 * attempt is given — so a cancelled caller waits at most that long for a socket nobody is reading.
 */
internal object Suspending {

  /**
   * Suspends until that future has completed, and answers what it completed with.
   *
   * @param future what the runtime is working on
   * @param T what the future answers
   * @return what it answered
   */
  suspend fun <T> awaiting(future: CompletableFuture<T>): T = suspendCoroutine { continuation ->
    future.whenComplete { value, failure ->
      if (failure == null) {
        continuation.resume(value)
      } else {
        continuation.resumeWithException(unwrapped(failure))
      }
    }
  }

  /**
   * Suspends for that long, holding no thread while it does.
   *
   * @param millis how long to wait, in milliseconds
   */
  suspend fun pausing(millis: Long) {
    if (millis <= 0) {
      return
    }
    val delayed = CompletableFuture.delayedExecutor(millis, TimeUnit.MILLISECONDS)
    awaiting(CompletableFuture.supplyAsync({ }, delayed))
  }

  /**
   * The failure a future actually carried, out of the wrapper the runtime completed it with.
   *
   * @param failure what the future reported
   * @return what it was raised out of
   */
  fun unwrapped(failure: Throwable): Throwable {
    var walked = failure
    var depth = 0
    while (depth < MAX_WRAPPERS && walked is CompletionException) {
      walked = walked.cause ?: return walked
      depth++
    }
    return walked
  }

  /** How many wrappers one failure is unwrapped through before the walk gives up. */
  private const val MAX_WRAPPERS = 8
}
