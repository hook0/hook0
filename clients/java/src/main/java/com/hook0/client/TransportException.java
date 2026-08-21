package com.hook0.client;

/**
 * A request the API never answered, and what caused that.
 *
 * <p>The three causes are told apart because only one of them could end differently. A request that got no answer — a
 * connection refused or reset, an attempt out of time, a body that stopped mid-way — says nothing about whether the API
 * acted on it, which is exactly why a send carries an identifier the client chose itself, and why repeating it is safe
 * and worth doing. An answer that crossed a ceiling this client set for itself draws the same answer the second time,
 * and reading it again four times over costs the caller four times as much for the same failure. A URL nothing can be
 * sent to was never sent at all, and a repetition builds the same unusable request, turning a misconfiguration into a
 * message that accuses the network.
 *
 * <p>What decides is the cause, never the type that happened to carry it: the same {@code IOException} stands for a
 * reset connection and for a host that does not resolve, and a client sorting by type retries a typo four times.
 *
 * <p>The names are the ones the shared conformance corpus gives them, so the verdict a client applies and the verdict
 * that corpus writes down are the same words.
 */
public final class TransportException extends Hook0Exception {

  private static final long serialVersionUID = 1L;

  /** The API was reached for and answered nothing this client could read to its end. */
  public static final String NO_ANSWER = "no_answer";

  /** The API answered, and what it answered crossed a ceiling this client set for itself. */
  public static final String ANSWER_ABOVE_A_BOUND = "answer_above_a_bound";

  /** There is nowhere to send the request, so nothing was sent. */
  public static final String UNUSABLE_API_URL = "unusable_api_url";

  private final String causeName;
  private final boolean retryable;

  private TransportException(String detail, String causeName, boolean retryable) {
    super(detail);
    this.causeName = causeName;
    this.retryable = retryable;
  }

  /**
   * Which of the corpus's causes this is.
   *
   * @return the name the shared contract gives the cause
   */
  public String causeName() {
    return causeName;
  }

  /**
   * Whether repeating the request that met this could end differently.
   *
   * @return whether a second attempt is worth issuing
   */
  public boolean retryable() {
    return retryable;
  }

  /**
   * The API was reached for and answered nothing this client could read to its end.
   *
   * @param detail what to say about the failure
   * @return the failure to raise
   */
  public static TransportException noAnswer(String detail) {
    return new TransportException(detail, NO_ANSWER, true);
  }

  /**
   * The API answered, and what it answered crossed a ceiling this client set for itself.
   *
   * @param detail what to say about the failure
   * @return the failure to raise
   */
  public static TransportException answerAboveABound(String detail) {
    return new TransportException(detail, ANSWER_ABOVE_A_BOUND, false);
  }

  /**
   * There is nowhere to send the request, so nothing was sent.
   *
   * @param detail what to say about the failure
   * @return the failure to raise
   */
  public static TransportException unusableApiUrl(String detail) {
    return new TransportException(detail, UNUSABLE_API_URL, false);
  }
}
