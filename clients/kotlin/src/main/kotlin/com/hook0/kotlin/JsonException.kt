package com.hook0.kotlin

/**
 * A document the reader this artefact carries could not read, or a value it could not write.
 *
 * Every ceiling [Json] applies is reported through this rather than by trimming what crossed it:
 * half a document read as a whole one is worse than no document at all.
 *
 * @param detail what to say about the failure
 */
class JsonException(detail: String) : Hook0Exception(detail)
