# 4. REST API Collection Filtering Specification

Date: 2025-01-20

## Status

Accepted

## Context

Hook0's REST API provides collection endpoints (e.g., `/applications`, `/subscriptions`, `/event_types`, `/request_attempts`) that require flexible filtering capabilities. Prior to this decision, filtering was implemented inconsistently across endpoints, with some handlers constructing SQL queries through string concatenation and manual parameter binding. This approach had several issues:

1. **Security Risk**: String interpolation in SQL queries creates potential SQL injection vulnerabilities if not carefully implemented
2. **Inconsistency**: Each handler implemented filtering differently, making the codebase harder to maintain
3. **Scalability**: Adding new filter types required duplicating logic across multiple handlers
4. **Type Safety**: No centralized validation of filter parameters or SQL construction patterns
5. **Testing Complexity**: SQL injection protection needed to be verified independently for each endpoint

### Current Data Model

Hook0's domain entities commonly include these field types that require filtering:

- **Scalar fields**: UUIDs, strings, booleans (e.g., `application_id`, `name`, `is_enabled`)
- **JSONB fields**: Flexible metadata and labels (e.g., `metadata`, `labels`)
- **Array fields**: Event types, tags (e.g., `event_types[]`)
- **Timestamp fields**: Created/updated dates with range queries (e.g., `created_at`)

## Decision

We will implement a consistent, type-safe filtering system across all REST API collection endpoints using:

### 1. QueryBuilder Pattern

A centralized `QueryBuilder` struct that:
- Constructs SQL WHERE clauses using parameterized queries exclusively
- Never interpolates user input directly into SQL strings
- Maintains a parameter index counter for PostgreSQL's `$N` syntax
- Provides typed methods for each filter type (string, UUID, JSONB, boolean, date range, array)
- Binds parameters to `sqlx` queries in a type-safe manner

**Implementation**: `api/src/query_builder.rs`

```rust
pub struct QueryBuilder {
    conditions: Vec<String>,    // SQL condition strings with placeholders
    pub param_index: usize,      // Next parameter index ($N)
    params: Vec<QueryParam>,     // Typed parameter values
}
```

### 2. Filter Type Taxonomy

#### Scalar Filters (Equality)
- **URL Syntax**: `?field=value`
- **SQL Pattern**: `field = $N`
- **Types**: String, UUID, Boolean
- **Examples**:
  - `?name=my-app` → `name = $2`
  - `?application_id=550e8400-e29b-41d4-a716-446655440000` → `application__id = $2`
  - `?is_enabled=true` → `is_enabled = $2`

#### JSONB Filters (Containment)
- **URL Syntax**: `?metadata.key=value&metadata.key2=value2`
- **SQL Pattern**: `metadata @> $N::jsonb`
- **Operator**: `@>` (PostgreSQL containment operator)
- **Semantics**: "Must contain at least these key-value pairs"
- **Index Support**: GIN indexes on JSONB columns
- **Examples**:
  - `?metadata.env=prod&metadata.region=eu-west` → `metadata @> '{"env":"prod","region":"eu-west"}'::jsonb`
  - `?labels.team=platform&labels.priority=high` → `labels @> '{"team":"platform","priority":"high"}'::jsonb`

#### Array Filters (Overlap)
- **URL Syntax**: `?field=value1,value2` (comma-separated)
- **SQL Pattern**: `field && ARRAY[$N]::text[]` (overlap) or `field @> ARRAY[$N]::text[]` (containment)
- **Examples**:
  - `?event_types=user.created,user.updated` → `event_types && ARRAY['user.created', 'user.updated']::text[]`

#### Date Range Filters
- **URL Syntax**: `?field[gte]=date1&field[lte]=date2`
- **SQL Pattern**: `field >= $N AND field <= $M`
- **Format Support**: RFC3339 (`2024-01-01T00:00:00Z`) or simple date (`2024-01-01`)
- **Examples**:
  - `?created_at[gte]=2024-01-01&created_at[lte]=2024-12-31` → `created_at >= $2 AND created_at <= $3`

### 3. SQL Injection Prevention

**Defense Mechanisms**:
1. **Column names are always hardcoded literals** - never derived from user input
2. **User values are stored in `QueryParam` enum** - typed and validated before binding
3. **SQL strings contain only placeholders** (`$1`, `$2`, etc.) - no interpolation
4. **PostgreSQL driver handles all escaping** - via `sqlx::bind()`
5. **Comprehensive test coverage** - validates injection attack vectors

**Example Attack Prevention**:
```rust
// User input: "test'; DROP TABLE users; --"
builder.add_string_filter("name", Some("test'; DROP TABLE users; --".to_string()));

// Generated SQL (safe):
// WHERE name = $2
// Parameter $2 = "test'; DROP TABLE users; --" (treated as literal string)
```

### 4. Handler Integration Pattern

All collection endpoints follow this consistent pattern:

```rust
pub async fn list(qs: Query<Qs>) -> Result<Json<Vec<Entity>>> {
    // 1. Initialize builder with base condition
    let mut query_builder = QueryBuilder::new(
        "organization_id = $1 AND deleted_at IS NULL".to_string(),
        2  // Next parameter index
    );

    // 2. Add optional filters
    query_builder.add_string_filter("name", qs.name.clone());
    query_builder.add_bool_filter("is_enabled", qs.is_enabled);

    // 3. Build WHERE clause
    let where_clause = query_builder.build_where_clause();
    let sql = format!("SELECT ... WHERE {}", where_clause);

    // 4. Bind parameters in order
    let query = query_as(&sql).bind(qs.organization_id);
    let query = query_builder.bind_params(query);

    // 5. Execute
    query.fetch_all(&state.db_pool).await
}
```

### 5. Performance Optimization

**Database Indexes**:
- **GIN indexes** for JSONB fields: `CREATE INDEX idx_table_metadata ON schema.table USING GIN (metadata);`
- **B-tree indexes** for scalar fields: `CREATE INDEX idx_table_name ON schema.table (name);`
- **Composite indexes** for common filter combinations

**Query Optimization**:
- The `@>` containment operator performs efficiently with GIN indexes
- Optional filters only add SQL conditions when values are present
- Parameter binding avoids query plan cache pollution

## Alternatives Considered

### Alternative 1: ORM Query Builder (e.g., Diesel)
**Pros**: Type-safe schema mapping, automatic query generation
**Cons**:
- Adds significant dependency weight
- Steeper learning curve for team
- Less control over SQL generation
- Migration effort for existing codebase
**Rejected**: Too heavyweight for our needs; custom solution provides sufficient safety with better control

### Alternative 2: Raw SQL with Manual Parameter Binding
**Pros**: Maximum flexibility, no abstractions
**Cons**:
- Error-prone for complex dynamic queries
- Duplicated logic across handlers
- Higher risk of SQL injection bugs
- Difficult to enforce consistency
**Rejected**: The inconsistency and maintenance burden outweigh the flexibility benefits

### Alternative 3: GraphQL with Dynamic Filtering
**Pros**: Extremely flexible client-side filtering, standardized query language
**Cons**:
- Requires complete API redesign
- Adds complexity (GraphQL server, schema management)
- Higher learning curve for clients
- Potential for complex/expensive queries
**Rejected**: Too disruptive for incremental improvement; REST API is working well

### Alternative 4: Filter DSL (Domain-Specific Language)
**Pros**: Very expressive, could support complex boolean logic (AND/OR/NOT)
**Cons**:
- Requires parser implementation and maintenance
- Complex to document and debug
- Overkill for current filtering needs
- Higher cognitive load for API consumers
**Rejected**: Current use cases don't require boolean logic complexity

## Consequences

### Positive

1. **Security**: SQL injection vulnerabilities are eliminated by design through parameterized queries
2. **Consistency**: All collection endpoints use identical filtering patterns
3. **Maintainability**: Filter logic is centralized in `query_builder.rs` (400 lines including tests)
4. **Type Safety**: Rust's type system enforces correct parameter types at compile time
5. **Testability**: Comprehensive unit tests validate SQL injection protection (7 dedicated tests)
6. **Performance**: Database indexes can be optimized for known filter patterns
7. **Extensibility**: Adding new filter types requires implementing one method in `QueryBuilder`
8. **Documentation**: Filter behavior is self-documenting through typed query parameter structs

### Negative

1. **Verbosity**: Handler code is more verbose than simple string interpolation (trade-off for safety)
2. **Learning Curve**: New developers must understand the `QueryBuilder` pattern
3. **Abstraction Cost**: Debug queries require understanding the builder's SQL generation logic
4. **Limited Expressiveness**: No support for OR conditions or complex boolean logic (acceptable for current needs)

### Neutral

1. **Migration Effort**: Existing handlers need refactoring to use `QueryBuilder` (completed for 4 handlers: `applications`, `application_secrets`, `event_types`, `request_attempts`)
2. **Test Coverage**: Requires maintaining tests for both `QueryBuilder` and handler integration
3. **Future JSONB Filtering**: Reserved methods exist for metadata/labels filtering (e.g., `add_jsonb_filter`) but are not yet used in handlers

## Implementation Status

### Completed
- ✅ `api/src/query_builder.rs` with comprehensive tests
- ✅ Refactored handlers:
  - `api/src/handlers/applications.rs` (name filter)
  - `api/src/handlers/application_secrets.rs` (application name filter with JOIN)
  - `api/src/handlers/event_types.rs` (service, resource, verb, application, event type name filters)
  - `api/src/handlers/request_attempts.rs` (event ID, subscription ID, event type name filters)

### Reserved for Future Use
- `add_jsonb_filter()` - for metadata/labels filtering
- `add_bool_filter()` - for boolean fields (e.g., `is_enabled`)
- `add_date_gte_filter()` / `add_date_lte_filter()` - for date range filtering
- `add_array_overlap_filter()` - for array field filtering
- `parse_date_filter()` - flexible date parsing helper

## References

- [OWASP SQL Injection Prevention Cheat Sheet](https://cheats.owasp.org/cheatsheets/SQL_Injection_Prevention_Cheat_Sheet.html)
- [PostgreSQL JSONB Operators](https://www.postgresql.org/docs/current/functions-json.html)
- [sqlx Documentation - Bind Parameters](https://docs.rs/sqlx/latest/sqlx/)
- Implementation: `api/src/query_builder.rs`
- Test Suite: `api/src/query_builder.rs::tests` (7 tests including SQL injection protection)

## Future Considerations

1. **Complex Boolean Logic**: If use cases emerge requiring AND/OR combinations, consider implementing a filter DSL
2. **Full-Text Search**: For text-heavy fields, evaluate PostgreSQL's `tsvector` and trigram indexes
3. **Pagination Optimization**: Add cursor-based pagination support to `QueryBuilder` for large result sets
4. **Filter Validation**: Add compile-time verification that column names exist in schema (via macros or code generation)
5. **Telemetry**: Add query performance metrics to identify slow filters requiring index optimization
