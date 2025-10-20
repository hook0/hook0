use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::HashMap;

/// Builder for dynamic SQL WHERE clauses with parameterized queries.
///
/// This module provides a safe, SQL-injection-proof way to build dynamic WHERE clauses.
/// All user input is bound as parameters ($1, $2, etc.) and never interpolated into SQL strings.
///
/// # Safety
/// - Column names are always hardcoded literals
/// - User values are stored in QueryParam enum and bound separately
/// - SQL strings contain only placeholders, never user input
/// - PostgreSQL driver handles all escaping automatically
///
/// # Example
/// ```rust
/// let mut builder = QueryBuilder::new("organization_id = $1".to_string(), 2);
/// builder.add_string_filter("name", Some("test".to_string()));
/// let where_clause = builder.build_where_clause(); // "organization_id = $1 AND name = $2"
/// let query = query_as(&format!("SELECT * FROM table WHERE {}", where_clause))
///     .bind(org_id);
/// let query = builder.bind_params(query);
/// ```
pub struct QueryBuilder {
    conditions: Vec<String>,
    pub param_index: usize,
    params: Vec<QueryParam>,
}

#[derive(Debug)]
#[allow(dead_code)] // All variants reserved for current and future handler use
pub enum QueryParam {
    Uuid(uuid::Uuid),
    String(String),
    Bool(bool),
    DateTime(DateTime<Utc>),
    Json(Value),
    StringArray(Vec<String>),
}

impl QueryBuilder {
    /// Create a new QueryBuilder starting with a base condition
    pub fn new(base_condition: String, starting_param_index: usize) -> Self {
        Self {
            conditions: vec![base_condition],
            param_index: starting_param_index,
            params: Vec::new(),
        }
    }

    /// Add a JSONB containment filter (metadata @> $X or labels @> $X)
    ///
    /// Reserved for future subscription filtering by metadata/labels.
    #[allow(dead_code)]
    pub fn add_jsonb_filter(
        &mut self,
        column_name: &str,
        filters: HashMap<String, String>,
    ) -> &mut Self {
        if !filters.is_empty() {
            self.conditions
                .push(format!("{} @> ${}", column_name, self.param_index));
            self.param_index += 1;
            self.params
                .push(QueryParam::Json(serde_json::to_value(filters).unwrap()));
        }
        self
    }

    /// Add a boolean equality filter
    ///
    /// Reserved for future subscription filtering by boolean fields (e.g., is_enabled).
    #[allow(dead_code)]
    pub fn add_bool_filter(&mut self, column_name: &str, value: Option<bool>) -> &mut Self {
        if let Some(v) = value {
            self.conditions
                .push(format!("{} = ${}", column_name, self.param_index));
            self.param_index += 1;
            self.params.push(QueryParam::Bool(v));
        }
        self
    }

    /// Add a string equality filter
    pub fn add_string_filter(&mut self, column_name: &str, value: Option<String>) -> &mut Self {
        if let Some(v) = value {
            self.conditions
                .push(format!("{} = ${}", column_name, self.param_index));
            self.param_index += 1;
            self.params.push(QueryParam::String(v));
        }
        self
    }

    /// Add a UUID equality filter
    pub fn add_uuid_filter(&mut self, column_name: &str, value: Option<uuid::Uuid>) -> &mut Self {
        if let Some(v) = value {
            self.conditions
                .push(format!("{} = ${}", column_name, self.param_index));
            self.param_index += 1;
            self.params.push(QueryParam::Uuid(v));
        }
        self
    }

    /// Add a date range filter (>=)
    ///
    /// Reserved for future subscription filtering by date ranges (e.g., created_at >= date).
    #[allow(dead_code)]
    pub fn add_date_gte_filter(
        &mut self,
        column_name: &str,
        value: Option<DateTime<Utc>>,
    ) -> &mut Self {
        if let Some(v) = value {
            self.conditions
                .push(format!("{} >= ${}", column_name, self.param_index));
            self.param_index += 1;
            self.params.push(QueryParam::DateTime(v));
        }
        self
    }

    /// Add a date range filter (<=)
    ///
    /// Reserved for future subscription filtering by date ranges (e.g., created_at <= date).
    #[allow(dead_code)]
    pub fn add_date_lte_filter(
        &mut self,
        column_name: &str,
        value: Option<DateTime<Utc>>,
    ) -> &mut Self {
        if let Some(v) = value {
            self.conditions
                .push(format!("{} <= ${}", column_name, self.param_index));
            self.param_index += 1;
            self.params.push(QueryParam::DateTime(v));
        }
        self
    }

    /// Add an array overlap filter (for comma-separated values)
    ///
    /// Reserved for future subscription filtering by event type arrays.
    #[allow(dead_code)]
    pub fn add_array_overlap_filter(
        &mut self,
        subquery: &str,
        values: Option<Vec<String>>,
    ) -> &mut Self {
        if let Some(v) = values
            && !v.is_empty()
        {
            self.conditions.push(format!(
                "subscription__id IN ({} WHERE event_type__name = ANY(${}::text[]))",
                subquery, self.param_index
            ));
            self.param_index += 1;
            self.params.push(QueryParam::StringArray(v));
        }
        self
    }

    /// Build the WHERE clause
    pub fn build_where_clause(&self) -> String {
        self.conditions.join(" AND ")
    }

    /// Get the collected parameters for binding
    ///
    /// Primarily used in tests for validation. May be used in future for debugging/logging.
    #[allow(dead_code)]
    pub fn get_params(&self) -> &[QueryParam] {
        &self.params
    }

    /// Bind parameters to a sqlx query
    pub fn bind_params<'q, DB, O>(
        &'q self,
        mut query: sqlx::query::QueryAs<'q, DB, O, <DB as sqlx::Database>::Arguments<'q>>,
    ) -> sqlx::query::QueryAs<'q, DB, O, <DB as sqlx::Database>::Arguments<'q>>
    where
        DB: sqlx::Database,
        O: for<'r> sqlx::FromRow<'r, DB::Row>,
        uuid::Uuid: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
        String: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
        bool: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
        DateTime<Utc>: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
        Value: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
        Vec<String>: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    {
        for param in &self.params {
            query = match param {
                QueryParam::Uuid(v) => query.bind(v),
                QueryParam::String(v) => query.bind(v),
                QueryParam::Bool(v) => query.bind(v),
                QueryParam::DateTime(v) => query.bind(v),
                QueryParam::Json(v) => query.bind(v),
                QueryParam::StringArray(v) => query.bind(v),
            };
        }
        query
    }
}

/// Parse a date string (RFC3339 or YYYY-MM-DD format)
///
/// Reserved for future subscription filtering by date ranges with flexible date format parsing.
#[allow(dead_code)]
pub fn parse_date_filter(
    date_str: &str,
    is_end_of_day: bool,
) -> Result<DateTime<Utc>, validator::ValidationErrors> {
    DateTime::parse_from_rfc3339(date_str)
        .map(|d| d.with_timezone(&Utc))
        .or_else(|_| {
            let time = if is_end_of_day {
                (23, 59, 59)
            } else {
                (0, 0, 0)
            };
            chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .map(|d| d.and_hms_opt(time.0, time.1, time.2).unwrap().and_utc())
                .map_err(|_| validator::ValidationErrors::new())
        })
        .map_err(|_| validator::ValidationErrors::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn test_query_builder_basic() {
        let mut builder = QueryBuilder::new("application_id = $1".to_string(), 2);

        builder.add_bool_filter("is_enabled", Some(true));
        builder.add_string_filter("name", Some("test".to_string()));

        let where_clause = builder.build_where_clause();
        assert_eq!(
            where_clause,
            "application_id = $1 AND is_enabled = $2 AND name = $3"
        );
        assert_eq!(builder.get_params().len(), 2);
    }

    #[test]
    fn test_query_builder_jsonb() {
        let mut builder = QueryBuilder::new("base = $1".to_string(), 2);

        let mut metadata = HashMap::new();
        metadata.insert("env".to_string(), "prod".to_string());

        builder.add_jsonb_filter("metadata", metadata);

        let where_clause = builder.build_where_clause();
        assert_eq!(where_clause, "base = $1 AND metadata @> $2");
        assert_eq!(builder.get_params().len(), 1);
    }

    #[test]
    fn test_query_builder_empty_filters() {
        let mut builder = QueryBuilder::new("base = $1".to_string(), 2);

        builder.add_bool_filter("is_enabled", None);
        builder.add_string_filter("name", None);
        builder.add_jsonb_filter("metadata", HashMap::new());

        let where_clause = builder.build_where_clause();
        assert_eq!(where_clause, "base = $1");
        assert_eq!(builder.get_params().len(), 0);
    }

    #[test]
    fn test_parse_date_filter_rfc3339() {
        let result = parse_date_filter("2024-01-01T00:00:00Z", false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_date_filter_simple_date_start() {
        let result = parse_date_filter("2024-01-01", false);
        assert!(result.is_ok());
        let date = result.unwrap();
        assert_eq!(date.hour(), 0);
        assert_eq!(date.minute(), 0);
        assert_eq!(date.second(), 0);
    }

    #[test]
    fn test_parse_date_filter_simple_date_end() {
        let result = parse_date_filter("2024-01-01", true);
        assert!(result.is_ok());
        let date = result.unwrap();
        assert_eq!(date.hour(), 23);
        assert_eq!(date.minute(), 59);
        assert_eq!(date.second(), 59);
    }

    #[test]
    fn test_parse_date_filter_invalid() {
        let result = parse_date_filter("invalid-date", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_sql_injection_protection_string_filter() {
        let mut builder = QueryBuilder::new("base = $1".to_string(), 2);

        // Try SQL injection payload
        let malicious_input = "test' OR '1'='1".to_string();
        builder.add_string_filter("name", Some(malicious_input.clone()));

        let where_clause = builder.build_where_clause();

        // Verify the WHERE clause contains only placeholders, not the malicious string
        assert_eq!(where_clause, "base = $1 AND name = $2");
        assert!(!where_clause.contains("OR '1'='1'"));

        // Verify the malicious string is stored as a parameter value
        let params = builder.get_params();
        assert_eq!(params.len(), 1);
        match &params[0] {
            QueryParam::String(s) => assert_eq!(s, &malicious_input),
            _ => panic!("Expected String parameter"),
        }
    }

    #[test]
    fn test_sql_injection_protection_multiple_filters() {
        let mut builder = QueryBuilder::new("org_id = $1".to_string(), 2);

        // Multiple injection attempts
        builder.add_string_filter("name", Some("test'; DROP TABLE users; --".to_string()));
        builder.add_string_filter("email", Some("admin@test.com' OR '1'='1".to_string()));
        builder.add_bool_filter("is_active", Some(true));

        let where_clause = builder.build_where_clause();

        // Verify only placeholders in SQL
        assert_eq!(
            where_clause,
            "org_id = $1 AND name = $2 AND email = $3 AND is_active = $4"
        );
        assert!(!where_clause.contains("DROP TABLE"));
        assert!(!where_clause.contains("--"));
        assert!(!where_clause.contains("OR '1'='1'"));

        // Verify all malicious strings are stored as parameters
        assert_eq!(builder.get_params().len(), 3);
    }

    #[test]
    fn test_sql_injection_protection_uuid_filter() {
        use uuid::Uuid;

        let mut builder = QueryBuilder::new("base = $1".to_string(), 2);
        let test_uuid = Uuid::new_v4();

        builder.add_uuid_filter("user_id", Some(test_uuid));

        let where_clause = builder.build_where_clause();

        // Verify placeholder is used
        assert_eq!(where_clause, "base = $1 AND user_id = $2");

        // Verify UUID is stored as parameter
        let params = builder.get_params();
        assert_eq!(params.len(), 1);
        match &params[0] {
            QueryParam::Uuid(u) => assert_eq!(u, &test_uuid),
            _ => panic!("Expected Uuid parameter"),
        }
    }

    #[test]
    fn test_sql_injection_protection_jsonb_filter() {
        let mut builder = QueryBuilder::new("base = $1".to_string(), 2);

        let mut malicious_metadata = HashMap::new();
        malicious_metadata.insert("key".to_string(), "value' OR '1'='1".to_string());

        builder.add_jsonb_filter("metadata", malicious_metadata.clone());

        let where_clause = builder.build_where_clause();

        // Verify placeholder is used for JSONB
        assert_eq!(where_clause, "base = $1 AND metadata @> $2");
        assert!(!where_clause.contains("OR '1'='1'"));

        // Verify malicious metadata is stored as JSON parameter
        let params = builder.get_params();
        assert_eq!(params.len(), 1);
        match &params[0] {
            QueryParam::Json(json) => {
                let expected_json = serde_json::to_value(malicious_metadata).unwrap();
                assert_eq!(json, &expected_json);
            }
            _ => panic!("Expected Json parameter"),
        }
    }
}
