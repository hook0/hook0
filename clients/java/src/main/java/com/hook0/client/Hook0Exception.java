package com.hook0.client;

/**
 * What every failure this SDK reports is a kind of.
 *
 * <p>Unchecked on purpose. Half of this client answers a {@link java.util.concurrent.CompletableFuture}, and a
 * checked exception cannot leave the lambda a future is completed through: the two surfaces would have ended up
 * reporting the same failure in two different ways, which is exactly the drift having two surfaces costs nothing.
 *
 * <p>Four kinds sit under it. {@link ClientException} is what this client refuses to do;
 * {@link TransportException} is a request that never produced an answer to read; {@link DecodeException} is an answer
 * that is not what the API declares it answers; and the generated problem hierarchy is what the API itself reported.
 */
public class Hook0Exception extends RuntimeException {

  private static final long serialVersionUID = 1L;

  /**
   * Builds one out of what went wrong.
   *
   * @param detail what to say about the failure
   */
  public Hook0Exception(String detail) {
    super(detail);
  }

  /**
   * Builds one out of what went wrong and what caused it.
   *
   * @param detail what to say about the failure
   * @param cause what this failure was raised out of
   */
  public Hook0Exception(String detail, Throwable cause) {
    super(detail, cause);
  }
}
