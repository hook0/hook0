package com.hook0.client;

import java.util.regex.Pattern;

/**
 * An event type, read out of the {@code service.resource_type.verb} it is written as.
 *
 * @param service the leading segment
 * @param resourceType the middle segment
 * @param verb the trailing segment
 */
public record EventType(String service, String resourceType, String verb) {

  /** What an event type reads as. */
  private static final Pattern PATTERN =
      Pattern.compile("\\A([A-Za-z0-9_]+)\\.([A-Za-z0-9_]+)\\.([A-Za-z0-9_]+)\\z");

  /**
   * Reads an event type, refusing one that does not name all three of its parts.
   *
   * @param written the event type as an application spells it
   * @return the three parts it names
   * @throws ClientException when it names anything else
   */
  public static EventType parse(String written) {
    if (written == null) {
      throw ClientException.invalidEventType("");
    }
    var read = PATTERN.matcher(written);
    if (!read.matches()) {
      throw ClientException.invalidEventType(written);
    }
    return new EventType(read.group(1), read.group(2), read.group(3));
  }

  /**
   * The event type as the API reads one.
   *
   * @return the three parts, joined
   */
  @Override
  public String toString() {
    return service + "." + resourceType + "." + verb;
  }
}
