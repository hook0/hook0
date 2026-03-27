use paperclip::v2::models::DefaultApiRaw;

/// Post-process the OpenAPI spec.
///
/// Note: Validation constraints and descriptions are now documented directly
/// in the code using doc comments (`///`) on struct fields. Paperclip extracts
/// these automatically. This function is kept as a placeholder for any future
/// post-processing needs.
pub fn enrich_openapi_spec(_spec: &mut DefaultApiRaw) {
    // No-op: descriptions are now in doc comments on struct fields.
    // This function is kept as a hook for any future post-processing needs.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enrich_openapi_spec_is_no_op() {
        let mut spec = DefaultApiRaw::default();
        enrich_openapi_spec(&mut spec);
        // Just verify it doesn't panic
        assert!(spec.definitions.is_empty());
    }
}
