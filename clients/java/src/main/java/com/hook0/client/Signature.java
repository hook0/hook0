package com.hook0.client;

import java.io.ByteArrayOutputStream;
import java.nio.charset.StandardCharsets;
import java.security.InvalidKeyException;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.regex.Pattern;
import javax.crypto.Mac;
import javax.crypto.spec.SecretKeySpec;

/**
 * A signature header, read into the pieces a verification needs.
 *
 * <p>A signature names the moment it was signed and one or two message authentication codes over the body. The
 * {@code v1} scheme also covers a list of request headers, so a receiver can tell apart two deliveries that carry the
 * same body but not the same context; {@code v0} covers the body alone and is what an older sender still produces. When
 * both are offered, {@code v1} is the one verified: accepting the weaker of two schemes on the strength of the sender
 * offering it is how a downgrade works.
 *
 * <p>Two things are refused before any code is computed. A header the signature says it covers but the request did not
 * carry is refused outright, because signing over an absent value would let a sender drop a header and keep the
 * signature valid. And a signature whose codes are not whole hexadecimal is refused rather than decoded as far as it
 * goes: a decoder that stops at the first bad character compares a prefix, and a prefix of the right code is not the
 * right code.
 *
 * @param timestamp the moment the delivery was signed, in whole seconds since the Unix epoch
 * @param coveredHeaders the headers the stronger scheme covers, lowercased and in order
 * @param bodyCode the {@code v0} code, decoded, or {@code null} when the sender offered none
 * @param headersCode the {@code v1} code, decoded, or {@code null} when the sender offered none
 */
public record Signature(long timestamp, List<String> coveredHeaders, byte[] bodyCode, byte[] headersCode) {

  /**
   * Longest signature header read.
   *
   * <p>The header is written by whoever reached the endpoint, so its size is bounded before any of it is split, decoded
   * or compared.
   */
  public static final int MAX_SIGNATURE_CHARS = 8 * 1024;

  /** Most {@code key=value} parts one signature header is split into. */
  public static final int MAX_SIGNATURE_PARTS = 32;

  /** Most header names one signature covers. */
  public static final int MAX_COVERED_HEADERS = 64;

  /** Furthest from the epoch, in either direction, a signature's moment may sit. */
  public static final long MAX_TIMESTAMP = 1_000_000_000_000L;

  /** Longest a moment may be written as, which bounds it before it is read as a number at all. */
  private static final int MAX_TIMESTAMP_CHARS = 20;

  /** What separates one part of the signature header from the next. */
  private static final String PART_SEPARATOR = ",";

  /**
   * What separates the name of a part from its value.
   *
   * <p>Only the first one counts: a value may hold further ones, and splitting on all of them would silently drop
   * everything past the second.
   */
  private static final char PART_ASSIGNATOR = '=';

  /** What separates two header names inside the {@code h} part, and what they are joined back with. */
  private static final String HEADER_NAME_SEPARATOR = " ";

  /** What separates the pieces of the message a code is computed over. */
  private static final String MESSAGE_SEPARATOR = ".";

  /** Part naming the moment the delivery was signed, in whole seconds since the Unix epoch. */
  private static final String TIMESTAMP_PART = "t";

  /** Part carrying the code covering the body alone. */
  private static final String BODY_SCHEME_PART = "v0";

  /** Part carrying the code covering the covered headers and the body. */
  private static final String HEADERS_SCHEME_PART = "v1";

  /** Part listing the headers the {@code v1} code covers, in the order it covers them. */
  private static final String COVERED_HEADERS_PART = "h";

  /** What a whole number of seconds reads as. */
  private static final Pattern WHOLE_SECONDS = Pattern.compile("\\A-?[0-9]+\\z");

  /** What a code reads as: whole pairs of hexadecimal digits, and nothing else. */
  private static final Pattern WHOLE_HEXADECIMAL = Pattern.compile("\\A(?:[0-9A-Fa-f][0-9A-Fa-f])+\\z");

  /** What a header name is written with, as RFC 9110 spells a token. */
  private static final Pattern HEADER_NAME = Pattern.compile("\\A[A-Za-z0-9!#$%&'*+\\-.^_`|~]+\\z");

  /** What the codes are computed with. */
  private static final String DIGEST = "HmacSHA256";

  /**
   * Reads a signature header, refusing anything it cannot read whole.
   *
   * @param signature the value of the {@code X-Hook0-Signature} header
   * @return the pieces it names
   * @throws ClientException for every way a header can fail to be one
   */
  public static Signature parse(String signature) {
    if (signature == null) {
      throw ClientException.refusedDelivery("there is no signature to read");
    }
    if (signature.length() > MAX_SIGNATURE_CHARS) {
      throw ClientException.refusedDelivery(
          "the signature is "
              + signature.length()
              + " characters long, above the "
              + MAX_SIGNATURE_CHARS
              + " accepted");
    }

    Map<String, String> read = partsOf(signature);
    if (read.size() < 2) {
      throw ClientException.refusedDelivery("the signature carries neither a timestamp nor a code");
    }

    byte[] bodyCode = codeOf(read, BODY_SCHEME_PART);
    byte[] headersCode = codeOf(read, HEADERS_SCHEME_PART);
    if (bodyCode == null && headersCode == null) {
      throw ClientException.refusedDelivery(
          "the signature carries neither a `" + BODY_SCHEME_PART + "` nor a `" + HEADERS_SCHEME_PART + "` code");
    }

    return new Signature(timestampOf(read), coveredHeadersOf(read), bodyCode, headersCode);
  }

  /** The {@code key=value} parts of a header, split on the first assignator of each and trimmed. */
  private static Map<String, String> partsOf(String signature) {
    String[] parts = signature.split(PART_SEPARATOR, -1);
    if (parts.length > MAX_SIGNATURE_PARTS) {
      throw ClientException.refusedDelivery(
          "the signature carries more than the " + MAX_SIGNATURE_PARTS + " parts accepted");
    }

    Map<String, String> read = new LinkedHashMap<>();
    for (String part : parts) {
      int at = part.indexOf(PART_ASSIGNATOR);
      if (at < 0) {
        continue;
      }
      read.put(part.substring(0, at).strip(), part.substring(at + 1).strip());
    }
    return read;
  }

  /** The moment the signature names, which it is not a signature without. */
  private static long timestampOf(Map<String, String> read) {
    String written = read.get(TIMESTAMP_PART);
    if (written == null) {
      throw ClientException.refusedDelivery("the signature carries no `" + TIMESTAMP_PART + "` part");
    }
    if (written.length() > MAX_TIMESTAMP_CHARS || !WHOLE_SECONDS.matcher(written).matches()) {
      throw ClientException.refusedDelivery("`" + written + "` is not a number of seconds");
    }

    long seconds = Long.parseLong(written);
    if (Math.abs(seconds) > MAX_TIMESTAMP) {
      throw ClientException.refusedDelivery(
          "the signature's moment is further than " + MAX_TIMESTAMP + " seconds from the epoch");
    }
    return seconds;
  }

  /** One of the codes a signature offers, decoded whole or not at all. */
  private static byte[] codeOf(Map<String, String> read, String part) {
    String written = read.get(part);
    if (written == null) {
      return null;
    }
    if (!WHOLE_HEXADECIMAL.matcher(written).matches()) {
      throw ClientException.refusedDelivery("the `" + part + "` code is not hexadecimal");
    }

    byte[] decoded = new byte[written.length() / 2];
    for (int index = 0; index < decoded.length; index++) {
      decoded[index] = (byte) Integer.parseInt(written.substring(index * 2, index * 2 + 2), 16);
    }
    return decoded;
  }

  /** The headers the stronger scheme covers, in the order it covers them. */
  private static List<String> coveredHeadersOf(Map<String, String> read) {
    String written = read.get(COVERED_HEADERS_PART);
    if (written == null || written.isEmpty()) {
      return List.of();
    }

    String[] names = written.split(HEADER_NAME_SEPARATOR, -1);
    if (names.length > MAX_COVERED_HEADERS) {
      throw ClientException.refusedDelivery(
          "the signature covers more than the " + MAX_COVERED_HEADERS + " headers accepted");
    }

    List<String> covered = new ArrayList<>(names.length);
    for (String name : names) {
      if (!HEADER_NAME.matcher(name).matches()) {
        throw ClientException.refusedDelivery("`" + name + "` is not a header name");
      }
      covered.add(name.toLowerCase(Locale.ROOT));
    }
    return List.copyOf(covered);
  }

  /**
   * Whether the code this signature carries is the one the secret produces.
   *
   * <p>The stronger scheme wins when both are offered, and the comparison is made in constant time: one that gave up at
   * the first differing byte would say, by how long it took, how much of a guess was right.
   *
   * @param payload the raw body of the webhook request
   * @param coveredValues the values of the covered headers, in the order the signature names them
   * @param subscriptionSecret the signing secret of the subscription the delivery was made for
   * @return whether the delivery is the one that was signed
   */
  public boolean matches(String payload, List<String> coveredValues, String subscriptionSecret) {
    ByteArrayOutputStream message = new ByteArrayOutputStream();
    append(message, Long.toString(timestamp));
    append(message, MESSAGE_SEPARATOR);

    if (headersCode != null) {
      append(message, String.join(HEADER_NAME_SEPARATOR, coveredHeaders));
      append(message, MESSAGE_SEPARATOR);
      append(message, String.join(MESSAGE_SEPARATOR, coveredValues));
      append(message, MESSAGE_SEPARATOR);
      append(message, payload);
      return MessageDigest.isEqual(code(subscriptionSecret, message.toByteArray()), headersCode);
    }

    // A signature carrying neither code is refused while it is being read, so what is left here is
    // the body-only scheme.
    append(message, payload);
    return MessageDigest.isEqual(code(subscriptionSecret, message.toByteArray()), bodyCode);
  }

  private static void append(ByteArrayOutputStream message, String text) {
    message.writeBytes(text.getBytes(StandardCharsets.UTF_8));
  }

  private static byte[] code(String subscriptionSecret, byte[] message) {
    byte[] key = (subscriptionSecret == null ? "" : subscriptionSecret).getBytes(StandardCharsets.UTF_8);
    if (key.length == 0) {
      // `SecretKeySpec` refuses an empty key outright, and a secret nobody set is a refusal rather
      // than a delivery this client has any way to accept.
      throw ClientException.refusedDelivery("the signature does not match: no subscription secret was given");
    }
    try {
      Mac mac = Mac.getInstance(DIGEST);
      mac.init(new SecretKeySpec(key, DIGEST));
      return mac.doFinal(message);
    } catch (NoSuchAlgorithmException | InvalidKeyException | IllegalArgumentException unusable) {
      throw ClientException.refusedDelivery("this runtime cannot compute " + DIGEST + ": " + unusable.getMessage());
    }
  }
}
