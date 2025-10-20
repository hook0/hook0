# Filters Module Usage Guide

This document demonstrates how to use the reusable `filters` module in different handlers.

## Overview

The `filters` module provides `MetadataFilters` and `LabelsFilters` types that can be embedded in any query parameter structure to enable metadata and labels filtering via dot notation.

## Current Implementation

### subscriptions.rs ✅

```rust
use crate::filters::{MetadataFilters, LabelsFilters};

#[derive(Debug, Deserialize, Serialize, Apiv2Schema)]
pub struct Qs {
    application_id: Uuid,
    is_enabled: Option<bool>,
    event_types: Option<String>,
    #[serde(default, rename = "created_at[gte]")]
    created_at_gte: Option<String>,
    #[serde(default, rename = "created_at[lte]")]
    created_at_lte: Option<String>,
    #[serde(flatten)]
    metadata: MetadataFilters,
    #[serde(flatten)]
    labels: LabelsFilters,
}

// In the list function:
let metadata_filters = qs.metadata.extract();
let labels_filters = qs.labels.extract();
```

**Query Example:**
```
GET /subscriptions?application_id=xxx&metadata.env=prod&metadata.region=eu&labels.team=backend&is_enabled=true
```

## Future Usage Examples

### applications.rs (when metadata/labels are added)

```rust
use crate::filters::{MetadataFilters, LabelsFilters};

#[derive(Debug, Deserialize, Serialize, Apiv2Schema)]
pub struct Qs {
    organization_id: Uuid,
    #[serde(flatten)]
    metadata: MetadataFilters,
    #[serde(flatten)]
    labels: LabelsFilters,
}

pub async fn list(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    qs: Query<Qs>,
) -> Result<Json<Vec<Application>>, Hook0Problem> {
    // Extract filters
    let metadata_filters = qs.metadata.extract();
    let labels_filters = qs.labels.extract();

    // Build WHERE conditions
    if !metadata_filters.is_empty() {
        let metadata_json = serde_json::to_value(metadata_filters).unwrap();
        // Use in SQL: WHERE metadata @> $X
    }

    if !labels_filters.is_empty() {
        let labels_json = serde_json::to_value(labels_filters).unwrap();
        // Use in SQL: WHERE labels @> $X
    }

    // ... rest of implementation
}
```

**Query Example:**
```
GET /applications?organization_id=xxx&metadata.env=staging&labels.priority=high
```

### request_attempts.rs (when metadata/labels are added)

```rust
use crate::filters::{MetadataFilters, LabelsFilters};

#[derive(Debug, Deserialize, Serialize, Apiv2Schema)]
pub struct Qs {
    application_id: Uuid,
    event_id: Option<Uuid>,
    subscription_id: Option<Uuid>,
    #[serde(flatten)]
    metadata: MetadataFilters,  // Filter by event metadata
    #[serde(flatten)]
    labels: LabelsFilters,       // Filter by subscription labels
    // ... other filters
}

pub async fn list(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    qs: Query<Qs>,
) -> Result<Json<Paginated<RequestAttempt>>, Hook0Problem> {
    // Extract filters
    let metadata_filters = qs.metadata.extract();
    let labels_filters = qs.labels.extract();

    // Build complex JOIN query with filters
    // ... implementation
}
```

**Query Example:**
```
GET /request_attempts?application_id=xxx&metadata.user_id=123&labels.environment=production
```

## Key Features

### 1. Prefix Isolation
The extraction methods only extract keys with the correct prefix:
- `metadata.env=prod` → extracted as `env: prod`
- `azf=bb` → **ignored** (no prefix)
- `other_key=value` → **ignored** (no prefix)

### 2. Separate Field Storage
Metadata and labels are stored in separate fields in the `Qs` struct, not in a single `extra` field. This provides:
- Type safety
- Clear API contract
- Better OpenAPI documentation
- Easier testing

### 3. Comprehensive Testing
The module includes 7 unit tests covering:
- Extraction of only metadata.* keys
- Extraction of only labels.* keys
- Mixed keys (ensures non-prefixed keys are ignored)
- Empty filter handling
- Boundary conditions

## SQL Implementation Pattern

When implementing filtering in handlers, use this pattern:

```rust
// 1. Extract filters
let metadata_filters = qs.metadata.extract();
let labels_filters = qs.labels.extract();

// 2. Build dynamic WHERE conditions
let mut where_conditions = vec!["base_condition".to_string()];
let mut param_index = 2;

// 3. Add metadata filter
let metadata_filter_json = if !metadata_filters.is_empty() {
    where_conditions.push(format!("metadata @> ${}", param_index));
    param_index += 1;
    Some(serde_json::to_value(metadata_filters).unwrap())
} else {
    None
};

// 4. Add labels filter
let labels_filter_json = if !labels_filters.is_empty() {
    where_conditions.push(format!("labels @> ${}", param_index));
    param_index += 1;
    Some(serde_json::to_value(labels_filters).unwrap())
} else {
    None
};

// 5. Build SQL query
let where_clause = where_conditions.join(" AND ");
let sql = format!("SELECT ... WHERE {}", where_clause);

// 6. Bind parameters
let mut query = sqlx::query_as(&sql).bind(base_param);
if let Some(m) = metadata_filter_json {
    query = query.bind(m);
}
if let Some(l) = labels_filter_json {
    query = query.bind(l);
}
```

## Database Requirements

For optimal performance, ensure GIN indexes exist on JSONB columns:

```sql
CREATE INDEX idx_table_metadata ON schema.table USING GIN (metadata);
CREATE INDEX idx_table_labels ON schema.table USING GIN (labels);
```

## Benefits

1. **Reusability**: Write filtering logic once, use everywhere
2. **Consistency**: Same API across all endpoints
3. **Type Safety**: Compile-time guarantees via Rust's type system
4. **Testability**: Isolated, well-tested components
5. **Maintainability**: Changes to filtering logic happen in one place
6. **Performance**: Efficient JSONB containment queries with GIN indexes
