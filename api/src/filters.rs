use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Extracts filters with a specific prefix from a HashMap.
/// For example, with prefix "metadata.", extracts "env" from "metadata.env=prod"
fn extract_prefixed_filters(
    map: &HashMap<String, String>,
    prefix: &str,
) -> HashMap<String, String> {
    map.iter()
        .filter_map(|(k, v)| {
            k.strip_prefix(prefix)
                .map(|key| (key.to_string(), v.clone()))
        })
        .collect()
}

/// Query parameters structure for metadata filtering
#[derive(Debug, Default, Deserialize, Serialize, Apiv2Schema)]
pub struct MetadataFilters {
    #[serde(flatten)]
    inner: HashMap<String, String>,
}

impl MetadataFilters {
    /// Extracts only the metadata.* keys from the flattened map
    pub fn extract(&self) -> HashMap<String, String> {
        extract_prefixed_filters(&self.inner, "metadata.")
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.extract().is_empty()
    }
}

/// Query parameters structure for labels filtering
#[derive(Debug, Default, Deserialize, Serialize, Apiv2Schema)]
pub struct LabelsFilters {
    #[serde(flatten)]
    inner: HashMap<String, String>,
}

impl LabelsFilters {
    /// Extracts only the labels.* keys from the flattened map
    pub fn extract(&self) -> HashMap<String, String> {
        extract_prefixed_filters(&self.inner, "labels.")
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.extract().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_metadata_filters_only_metadata() {
        let mut map = HashMap::new();
        map.insert("metadata.env".to_string(), "prod".to_string());
        map.insert("metadata.region".to_string(), "eu".to_string());

        let filters = MetadataFilters { inner: map };
        let extracted = filters.extract();

        assert_eq!(extracted.len(), 2);
        assert_eq!(extracted.get("env"), Some(&"prod".to_string()));
        assert_eq!(extracted.get("region"), Some(&"eu".to_string()));
    }

    #[test]
    fn test_extract_metadata_filters_mixed_keys() {
        let mut map = HashMap::new();
        map.insert("metadata.env".to_string(), "prod".to_string());
        map.insert("azf".to_string(), "bb".to_string());
        map.insert("other_key".to_string(), "value".to_string());

        let filters = MetadataFilters { inner: map };
        let extracted = filters.extract();

        // Only metadata.* keys should be extracted
        assert_eq!(extracted.len(), 1);
        assert_eq!(extracted.get("env"), Some(&"prod".to_string()));
        assert_eq!(extracted.get("azf"), None);
        assert_eq!(extracted.get("other_key"), None);
    }

    #[test]
    fn test_extract_metadata_filters_no_metadata() {
        let mut map = HashMap::new();
        map.insert("azf".to_string(), "bb".to_string());
        map.insert("other_key".to_string(), "value".to_string());

        let filters = MetadataFilters { inner: map };
        let extracted = filters.extract();

        assert_eq!(extracted.len(), 0);
        assert!(filters.is_empty());
    }

    #[test]
    fn test_extract_labels_filters_only_labels() {
        let mut map = HashMap::new();
        map.insert("labels.team".to_string(), "backend".to_string());
        map.insert("labels.priority".to_string(), "high".to_string());

        let filters = LabelsFilters { inner: map };
        let extracted = filters.extract();

        assert_eq!(extracted.len(), 2);
        assert_eq!(extracted.get("team"), Some(&"backend".to_string()));
        assert_eq!(extracted.get("priority"), Some(&"high".to_string()));
    }

    #[test]
    fn test_extract_labels_filters_mixed_keys() {
        let mut map = HashMap::new();
        map.insert("labels.plop".to_string(), "aa".to_string());
        map.insert("azf".to_string(), "bb".to_string());
        map.insert("metadata.env".to_string(), "prod".to_string());

        let filters = LabelsFilters { inner: map };
        let extracted = filters.extract();

        // Only labels.* keys should be extracted
        assert_eq!(extracted.len(), 1);
        assert_eq!(extracted.get("plop"), Some(&"aa".to_string()));
        assert_eq!(extracted.get("azf"), None);
        assert_eq!(extracted.get("env"), None);
    }

    #[test]
    fn test_extract_labels_filters_no_labels() {
        let mut map = HashMap::new();
        map.insert("azf".to_string(), "bb".to_string());
        map.insert("metadata.env".to_string(), "prod".to_string());

        let filters = LabelsFilters { inner: map };
        let extracted = filters.extract();

        assert_eq!(extracted.len(), 0);
        assert!(filters.is_empty());
    }

    #[test]
    fn test_empty_maps() {
        let metadata_filters = MetadataFilters {
            inner: HashMap::new(),
        };
        let labels_filters = LabelsFilters {
            inner: HashMap::new(),
        };

        assert!(metadata_filters.is_empty());
        assert!(labels_filters.is_empty());
        assert_eq!(metadata_filters.extract().len(), 0);
        assert_eq!(labels_filters.extract().len(), 0);
    }
}
