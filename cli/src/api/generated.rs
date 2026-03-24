//! Auto-generated API information from the Hook0 OpenAPI specification.
//!
//! This module includes code generated at build time from the swagger.json.
//! It provides compile-time validation that our API client methods match the
//! actual API specification.

// Include the generated code from build.rs
include!(concat!(env!("OUT_DIR"), "/openapi_info.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openapi_endpoints_loaded() {
        // Verify that endpoints were loaded from the spec
        assert!(
            !API_ENDPOINTS.is_empty(),
            "OpenAPI endpoints should be loaded from spec"
        );
    }

    #[test]
    fn test_key_endpoints_exist() {
        // Verify critical endpoints exist in the spec
        // These are the actual operationId values from the OpenAPI spec, converted to snake_case
        let critical_endpoints = [
            "applications_list",
            "applications_create",
            "events_list",
            "events_ingest", // "create" is called "ingest" in the API
            "event_types_list",
            "subscriptions_list",
            "subscriptions_create",
        ];

        for endpoint in critical_endpoints {
            let found = API_ENDPOINTS.iter().any(|e| e.name == endpoint);
            assert!(
                found,
                "Critical endpoint '{}' should exist in OpenAPI spec. Available: {:?}",
                endpoint,
                API_ENDPOINTS.iter().map(|e| &e.name).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn test_get_endpoint() {
        // If we have endpoints, test the lookup function
        if let Some(first) = API_ENDPOINTS.first() {
            let found = get_endpoint(&first.name);
            assert!(found.is_some(), "Should find endpoint by name");
            assert_eq!(found.unwrap().name, first.name);
        }
    }

    #[test]
    fn test_endpoint_info_structure() {
        for endpoint in API_ENDPOINTS {
            // Verify each endpoint has valid data
            assert!(
                !endpoint.name.is_empty(),
                "Endpoint name should not be empty"
            );
            assert!(
                ["GET", "POST", "PUT", "DELETE", "PATCH"].contains(&endpoint.method),
                "Invalid HTTP method: {}",
                endpoint.method
            );
            assert!(
                endpoint.path.starts_with("/api/"),
                "Path should start with /api/: {}",
                endpoint.path
            );
        }
    }
}
