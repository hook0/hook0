package com.hook0.client;

import java.util.Map;

/**
 * What the API answered one request, whether or not it answered a success.
 *
 * <p>Header names are lowercased and a later value wins over an earlier one under the same name, so a caller reads a
 * header without knowing which case the server wrote it in.
 *
 * @param status what the API answered under
 * @param headers what it answered beside the body
 * @param body the body it answered
 */
public record Answer(int status, Map<String, String> headers, String body) {

  /**
   * Whether the API read this as a success.
   *
   * @return whether the status is one of the successes
   */
  public boolean successful() {
    return status >= 200 && status < 300;
  }

  /**
   * One header the answer carried, looked up without regard to case.
   *
   * @param name the header to look for
   * @return its value, or {@code null} when the answer carried none
   */
  public String header(String name) {
    return headers.get(name.toLowerCase(java.util.Locale.ROOT));
  }
}
