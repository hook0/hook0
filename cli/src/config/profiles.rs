use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A configuration profile for a specific environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// API URL for this profile
    pub api_url: String,

    /// Application ID associated with this profile
    pub application_id: Uuid,

    /// Optional organization ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<Uuid>,

    /// Optional description for this profile
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Profile {
    /// Create a new profile
    pub fn new(api_url: String, application_id: Uuid) -> Self {
        Self {
            api_url,
            application_id,
            organization_id: None,
            description: None,
        }
    }

    /// Create a new profile with all fields
    pub fn with_details(
        api_url: String,
        application_id: Uuid,
        organization_id: Option<Uuid>,
        description: Option<String>,
    ) -> Self {
        Self {
            api_url,
            application_id,
            organization_id,
            description,
        }
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            api_url: "https://app.hook0.com/api/v1".to_string(),
            application_id: Uuid::nil(),
            organization_id: None,
            description: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_new() {
        let app_id = Uuid::new_v4();
        let profile = Profile::new("https://api.example.com".to_string(), app_id);

        assert_eq!(profile.api_url, "https://api.example.com");
        assert_eq!(profile.application_id, app_id);
        assert!(profile.organization_id.is_none());
        assert!(profile.description.is_none());
    }

    #[test]
    fn test_profile_with_details() {
        let app_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let profile = Profile::with_details(
            "https://api.example.com".to_string(),
            app_id,
            Some(org_id),
            Some("Production environment".to_string()),
        );

        assert_eq!(profile.api_url, "https://api.example.com");
        assert_eq!(profile.application_id, app_id);
        assert_eq!(profile.organization_id, Some(org_id));
        assert_eq!(profile.description, Some("Production environment".to_string()));
    }

    #[test]
    fn test_profile_default() {
        let profile = Profile::default();

        assert_eq!(profile.api_url, "https://app.hook0.com/api/v1");
        assert_eq!(profile.application_id, Uuid::nil());
        assert!(profile.organization_id.is_none());
        assert!(profile.description.is_none());
    }

    #[test]
    fn test_profile_serialization() {
        let app_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").expect("valid uuid");
        let profile = Profile::new("https://api.example.com".to_string(), app_id);

        let toml_str = toml::to_string(&profile).expect("serialization should work");
        assert!(toml_str.contains("api_url"));
        assert!(toml_str.contains("application_id"));
    }

    #[test]
    fn test_profile_deserialization() {
        let toml_str = r#"
            api_url = "https://api.example.com"
            application_id = "550e8400-e29b-41d4-a716-446655440000"
        "#;

        let profile: Profile = toml::from_str(toml_str).expect("deserialization should work");
        assert_eq!(profile.api_url, "https://api.example.com");
    }
}
