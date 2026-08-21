package com.hook0.kotlin

/**
 * An event type, read out of the `service.resource_type.verb` it is written as.
 *
 * @property service the leading segment
 * @property resourceType the middle segment
 * @property verb the trailing segment
 */
data class EventType(val service: String, val resourceType: String, val verb: String) {

  /**
   * The event type as the API reads one.
   *
   * @return the three parts, joined
   */
  override fun toString(): String = "$service.$resourceType.$verb"

  companion object {
    /** What an event type reads as. */
    private val PATTERN = Regex("\\A([A-Za-z0-9_]+)\\.([A-Za-z0-9_]+)\\.([A-Za-z0-9_]+)\\z")

    /**
     * Reads an event type, refusing one that does not name all three of its parts.
     *
     * @param written the event type as an application spells it
     * @return the three parts it names
     * @throws ClientException when it names anything else
     */
    fun parse(written: String): EventType {
      val read = PATTERN.matchEntire(written) ?: throw ClientException.invalidEventType(written)
      return EventType(read.groupValues[1], read.groupValues[2], read.groupValues[3])
    }
  }
}
