package com.hook0.kotlin

/**
 * One name and value of a query string, before either of them is escaped.
 *
 * A list of these rather than a map: the API declares parameters a request may carry more than
 * once, and a map would keep the last one and drop the rest without saying so.
 *
 * @property name what the parameter is called on the wire
 * @property value what it carries
 */
data class QueryParameter(val name: String, val value: String)
