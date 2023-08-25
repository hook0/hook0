# 3. VueJS component style

Date: 2023-07-17

## Status

Accepted

## Context

We need to unify the way we write VueJS components.

## Decision

Decided to use VueJS's composition API because it enables TypeScript type checking for components.

Components' scripts must be enclosed in:

```html
<script setup lang="ts">
// ...
</script>
```

Furthermore, source files that are not components must be written in TypeScript and not in JavaScript.

See refactoring MR: https://gitlab.com/hook0/hook0/-/merge_requests/42
