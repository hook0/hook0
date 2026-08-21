package com.hook0.client;

/**
 * One name and value of a query string, before either is escaped.
 *
 * <p>A list of these rather than a map: the document declares parameters a request may carry more than once, and a map
 * would keep whichever one was written last.
 *
 * @param name what the parameter travels under
 * @param value what it carries
 */
public record QueryParameter(String name, String value) {}
