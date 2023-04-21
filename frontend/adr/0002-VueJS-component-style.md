# 20. Record architecture decisions

Date: 2023-04-21

## Status

Accepted

## Context

We need to unify the way we write VueJS components.

## Decision

Decided to use `defineComponent` because it enables type inference for components defined in plain JavaScript.
