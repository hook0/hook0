# Statig FSM vs Match-Based State Machine: Comparison

**Date:** 2026-04-02
**Context:** Health monitor subscription state machine (`api/src/health_monitor/state_machine.rs`)

## Current State Machine

### States

| State | Meaning |
|-------|---------|
| `None` (no prior event) | Fresh subscription, never evaluated |
| `Warning` | Failure rate crossed `warning_failure_percent` |
| `Disabled` | Failure rate crossed `disable_failure_percent`; subscription is off |
| `Resolved` | Was in Warning, failure rate dropped back below threshold |

### Transitions (current match arms)

```
None/Resolved + failure >= disable%  --> Warning + Disabled (two events, immediate escalation)
None/Resolved + failure >= warning%  --> Warning
Warning + failure >= disable%        --> Disabled
Warning + failure still in range     --> (no-op, stay Warning)
Warning + failure < warning%         --> Resolved
Disabled                             --> (no-op, manual re-enable required)
Resolved within cooldown             --> (no-op, prevent email spam)
```

### Side-Effects per Transition

- **INSERT health event** (`subscription_health_event` row): Warning, Disabled, Resolved
- **UPDATE subscription** (`is_enabled = false`): Disabled
- **UPDATE subscription** (`failure_percent`): Always (every tick)
- **Push HealthAction**: Warning email, Disabled email + Hook0 event, Recovered email

### Code Shape (~120 lines)

```rust
pub async fn evaluate_health_transition(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    config: &HealthMonitorConfig,
    subscription: &SubscriptionHealth,
) -> Result<Vec<HealthAction>, HealthMonitorError> {
    // ...
    match last_status {
        Some(HealthStatus::Disabled) => {}
        Some(HealthStatus::Resolved) if within_cooldown => {}
        Some(HealthStatus::Warning) if still_in_range => {}
        Some(HealthStatus::Warning) if recovered => { /* insert resolved, push Recovered */ }
        Some(HealthStatus::Warning) => { /* disable, push Disabled */ }
        _ if severe => { /* insert warning, push Warning, disable, push Disabled */ }
        _ if warning => { /* insert warning, push Warning */ }
        _ => {}
    }
    Ok(actions)
}
```

Key characteristics:
- **Stateless function**: no struct, no `self`. State comes from DB (`last_health_status`), output goes to DB.
- **Transaction-bound**: every side-effect runs inside a `sqlx::Transaction`.
- **Returns actions**: side-effects (emails, webhooks) are collected as `Vec<HealthAction>` and dispatched *after* commit.
- **One call per subscription per tick**: not a long-lived object sitting in a loop.

---

## How It Would Look with Statig

### Statig Overview

[statig](https://github.com/mdeloof/statig) (767 stars, MIT, last pushed 2025-11) is a hierarchical state machine crate for Rust. Key features:

- `#[state_machine]` proc macro generates `State` and `Superstate` enums
- State handlers return `Outcome<State>`: `Transition(target)`, `Handled`, or `Super` (defer to superstate)
- Entry/exit actions via `#[state(entry_action = "...", exit_action = "...")]`
- Async handler support (auto-detected from `async fn`)
- `Context<'ctx>` type for passing external dependencies
- Shared storage on the struct implementing the state machine
- **No built-in guards** -- guard logic lives inside the handler match arms (same as today)

### Hypothetical Implementation

```rust
use statig::prelude::*;

/// Shared storage: the struct holds external deps needed by handlers.
pub struct HealthFsm {
    config: HealthMonitorConfig,
}

/// The event submitted each tick.
pub struct HealthEvent {
    failure_percent: f64,
    last_source: Option<HealthEventSource>,
}

/// Context passed mutably to each handler call.
pub struct FsmContext<'a> {
    transaction: &'a mut sqlx::Transaction<'static, sqlx::Postgres>,
    subscription: &'a SubscriptionHealth,
    actions: Vec<HealthAction>,
}

#[state_machine(
    initial = "State::healthy()",
    state(derive(Debug, Clone, PartialEq)),
)]
impl HealthFsm {
    // -- States --

    #[state]
    async fn healthy(
        &self,
        event: &HealthEvent,
        context: &mut FsmContext<'_>,
    ) -> Outcome<State> {
        if event.failure_percent >= self.config.disable_failure_percent as f64 {
            // Immediate escalation: warning + disable
            insert_health_event(/* ... */).await;
            context.actions.push(HealthAction::Warning(/* ... */));
            let disabled_at = disable_subscription(/* ... */).await;
            context.actions.push(HealthAction::Disabled(/* ... */));
            Transition(State::disabled())
        } else if event.failure_percent >= self.config.warning_failure_percent as f64 {
            insert_health_event(/* ... */).await;
            context.actions.push(HealthAction::Warning(/* ... */));
            Transition(State::warning())
        } else {
            Handled
        }
    }

    #[state]
    async fn warning(
        &self,
        event: &HealthEvent,
        context: &mut FsmContext<'_>,
    ) -> Outcome<State> {
        if event.failure_percent >= self.config.disable_failure_percent as f64 {
            let disabled_at = disable_subscription(/* ... */).await;
            context.actions.push(HealthAction::Disabled(/* ... */));
            Transition(State::disabled())
        } else if event.failure_percent < self.config.warning_failure_percent as f64 {
            insert_health_event(/* ... */).await;
            if event.last_source != Some(HealthEventSource::User) {
                context.actions.push(HealthAction::Recovered(/* ... */));
            }
            Transition(State::resolved())
        } else {
            Handled // still in warning range, no-op
        }
    }

    #[state]
    async fn disabled(
        &self,
        _event: &HealthEvent,
        _context: &mut FsmContext<'_>,
    ) -> Outcome<State> {
        Handled // absorbing state, manual re-enable required
    }

    #[state]
    async fn resolved(
        &self,
        event: &HealthEvent,
        context: &mut FsmContext<'_>,
    ) -> Outcome<State> {
        // Cooldown logic would need to be here or use state-local storage
        // for the timestamp. After cooldown, same as `healthy`.
        if event.failure_percent >= self.config.warning_failure_percent as f64 {
            insert_health_event(/* ... */).await;
            context.actions.push(HealthAction::Warning(/* ... */));
            Transition(State::warning())
        } else {
            Handled
        }
    }
}
```

### Usage (hypothetical)

```rust
// Problem: we'd need to create a StateMachine per subscription per tick,
// or persist one per subscription across ticks.
let fsm = HealthFsm { config: config.clone() };
let mut sm = fsm.state_machine();

// But the initial state depends on `subscription.last_health_status` from DB,
// not a fixed `initial = "..."`. We'd need to force-set the state.
// statig does not have a public `set_state()` -- initial state is fixed at compile time.

let event = HealthEvent { failure_percent, last_source };
let mut ctx = FsmContext { transaction, subscription, actions: vec![] };
sm.handle_with_context(&event, &mut ctx).await;
```

---

## Comparison

### Pros of Statig

| Benefit | Applicability Here |
|---------|--------------------|
| Enforced state enum -- compiler ensures all states handled | We already have `HealthStatus` enum + exhaustive match |
| Entry/exit actions | We have no entry/exit; side-effects are inline in transitions |
| Hierarchical superstates for shared behavior | We have 4 flat states, no hierarchy needed |
| State-local storage (data owned by variant) | Our state is a DB column, not in-memory |
| Visual state diagram from code structure | Marginal value for 4 states |
| Async handler support | Yes, but our side-effects are `sqlx` calls on a `&mut Transaction` |
| Prevents impossible transitions at compile time | Statig does not do this -- transitions are runtime via `Transition(target)` |

### Cons of Statig

| Drawback | Impact |
|----------|--------|
| **State lives in DB, not in memory.** Statig assumes a long-lived in-memory FSM. Our "state" is `subscription.last_health_status` loaded from Postgres each tick. We'd have to reconstruct the FSM from DB state on every call, or maintain a `HashMap<Uuid, StateMachine>` across ticks. | High -- fundamental mismatch |
| **Initial state is fixed at compile time.** `initial = "State::healthy()"` is baked in. Our initial state varies per subscription per tick (`None`, `Warning`, `Resolved`, `Disabled`). No `set_state()` API. | High -- would require workaround (dummy "init" event to jump to correct state) |
| **Context threading is awkward.** Passing `&mut Transaction` + `&SubscriptionHealth` + `Vec<HealthAction>` through `Context<'ctx>` works but adds a wrapper struct and lifetime gymnastics. | Medium |
| **No built-in guards.** Guard conditions (cooldown check, threshold comparisons) still live as `if` branches inside handlers -- same as today's `match` arms. No declarative transition table. | Neutral -- no improvement |
| **Side-effects in handlers.** Statig handlers run DB inserts and pushes. This is not "pure state + effect list" -- same as today but wrapped in more ceremony. | Neutral -- no improvement |
| **New dependency + proc macro.** Adds compile-time cost and a macro-generated layer to debug through. | Low-Medium |
| **4 states, 6 transitions.** This is a trivially small FSM. The overhead of the framework exceeds the complexity it manages. | High -- over-engineering |
| **Not a loop-based event system.** Statig is designed for `loop { sm.handle(queue.recv()) }`. Our usage is batch: iterate subscriptions, evaluate once, commit. Opposite usage pattern. | High -- architectural mismatch |

### What Statig Is Good For (vs. this use case)

Statig shines when you have:
- A **long-lived object** whose state evolves over many events (embedded device, protocol handler, UI component)
- **Hierarchical states** with shared behavior (e.g., `Blinking > LedOn/LedOff`)
- **State-local storage** (data that only exists in certain states)
- An **event queue** feeding into `handle()` in a loop

Our health monitor is none of these. It is a **batch evaluator** that reads state from a database, applies threshold logic, writes new state to the database, and collects side-effects. The "state machine" is implicit in the match arms and the DB column.

---

## Recommendation

**Do not adopt statig for this use case.**

The current match-based approach is the right tool:
1. **~40 lines of match logic** -- trivially readable, easy to add a new state
2. **State lives in the database** -- no in-memory FSM lifecycle to manage
3. **Zero dependencies** for the FSM itself
4. **Side-effects are explicit** -- `insert_health_event`, `disable_subscription`, `actions.push()` right where the transition happens
5. **Testable** -- pass a mock transaction, assert on returned actions

Statig would add a proc-macro dependency, a forced in-memory lifecycle that fights the DB-driven architecture, and roughly the same amount of code in each state handler (the `if` threshold checks don't go away). The only thing it would add is the `#[state]` annotation, which for 4 states provides negligible value over an exhaustive `match`.

If the state machine grows to 10+ states with deep hierarchy and state-local data, revisit. For now, the match block is the correct abstraction level.
