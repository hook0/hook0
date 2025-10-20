# Generic Filtering Proposal for `/subscriptions` Endpoint

**Response to MR #155 - David's comment on multiple metadata filters**

---

Hi David,

I agree with your `?metadata.k1=v1&metadata.k2=v2` approach! It's intuitive and works well with the `@>` SQL operator.

Rather than limiting this to `metadata` only, I propose a consistent filtering syntax for all relevant `Subscription` fields:

## 1. JSONB Fields (metadata, labels) - Dot Notation

**URL Query Syntax:**
```
?metadata.env=prod&metadata.region=eu-west
?labels.team=platform&labels.priority=high
```

**SQL Implementation:**
```sql
WHERE s.metadata @> '{"env":"prod","region":"eu-west"}'::jsonb
WHERE s.labels @> '{"team":"platform","priority":"high"}'::jsonb
```

The `@>` (containment) operator is perfect here: it performs well with GIN indexes, and the "must have AT LEAST all these key/value pairs" semantics matches exactly what clients need.

## 2. Boolean Fields - Direct Equality

**URL Query Syntax:**
```
?is_enabled=true
?is_enabled=false
```

**SQL Implementation:**
```sql
WHERE s.is_enabled = $X
```

## 3. Array Fields (event_types) - Overlap Operator

**URL Query Syntax:**
```
?event_types=user.created,user.updated
```

**SQL Implementation (overlap - at least one matching type):**
```sql
WHERE s.event_types && ARRAY[$X, $Y]::text[]
```

Or if we want "contains ALL types":
```sql
WHERE s.event_types @> ARRAY[$X, $Y]::text[]
```

## 4. Date Fields (created_at) - Comparison Operators

**URL Query Syntax:**
```
?created_at[gte]=2024-01-01
?created_at[lte]=2024-12-31
```

**SQL Implementation:**
```sql
WHERE s.created_at >= $X AND s.created_at <= $Y
```

## Combined Example

**Request:**
```
GET /subscriptions?application_id=xxx&metadata.env=prod&metadata.tenant=acme&is_enabled=true&event_types=user.created
```

**Resulting SQL:**
```sql
WHERE s.application__id = $1
  AND s.metadata @> '{"env":"prod","tenant":"acme"}'::jsonb
  AND s.is_enabled = true
  AND s.event_types && ARRAY['user.created']::text[]
  AND s.deleted_at IS NULL
```

## Rust Implementation

We could parse query parameters with a structure like:

```rust
#[derive(Debug, Deserialize, Apiv2Schema)]
#[serde(rename_all = "lowercase")]
pub struct SubscriptionFilters {
    application_id: Uuid,

    // JSONB filters - flattened map with dot notation
    #[serde(flatten)]
    metadata: HashMap<String, String>, // keys starting with "metadata."

    #[serde(flatten)]
    labels: HashMap<String, String>, // keys starting with "labels."

    // Simple filters
    is_enabled: Option<bool>,
    event_types: Option<Vec<String>>, // comma-separated or repeated params

    // Date range filters
    #[serde(flatten)]
    created_at: Option<DateRangeFilter>,
}
```

With custom parsing to extract `metadata.*` and `labels.*` prefixes from query parameters.

## Considerations

### 1. Performance
The `@>` operator benefits from GIN indexes on `metadata` and `labels`. If we don't have them already, we should add:

```sql
CREATE INDEX idx_subscription_metadata ON webhook.subscription USING GIN (metadata);
CREATE INDEX idx_subscription_labels ON webhook.subscription USING GIN (labels);
```

### 2. IAM Compatibility
The current permission system (`authorize_for_application`) remains compatible since filters only restrict results within the already-authorized scope. No IAM modifications needed for now.

### 3. Extensibility
This syntax is easily extensible to other endpoints (`/events`, `/event_types`, etc.) that also have `metadata`/`labels` fields.

### 4. Backward Compatibility
Absence of filters = current behavior (returns everything).

---

**What do you think?** I can detail the Rust parsing implementation if needed.
