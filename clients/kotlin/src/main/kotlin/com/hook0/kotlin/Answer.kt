package com.hook0.kotlin

import java.util.Locale

/**
 * What the API answered one request, whether or not it answered a success.
 *
 * Header names are lowercased and a later value wins over an earlier one under the same name, so a
 * caller reads a header without knowing which case the server wrote it in.
 *
 * @property status what the API answered under
 * @property headers what it answered beside the body
 * @property body the body it answered, which is the empty text when it answered none
 */
data class Answer(val status: Int, val headers: Map<String, String>, val body: String) {

  /**
   * Whether the API read this as a success.
   *
   * @return whether the status is one of the successes
   */
  fun successful(): Boolean = status >= LOWEST_SUCCESS && status < LOWEST_REDIRECTION

  /**
   * One header the answer carried, looked up without regard to case.
   *
   * @param name the header to look for
   * @return its value, or nothing when the answer carried none
   */
  fun header(name: String): String? = headers[name.lowercase(Locale.ROOT)]

  private companion object {
    /** Lowest status a response is read as a success under. */
    private const val LOWEST_SUCCESS = 200

    /** Lowest status that is no longer a success. */
    private const val LOWEST_REDIRECTION = 300
  }
}
