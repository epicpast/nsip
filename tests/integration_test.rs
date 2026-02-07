//! Integration tests for NSIP Search API client.

use nsip::{Error, NsipClient, SearchCriteria};

#[test]
fn test_client_creation() {
    let client = NsipClient::new();
    // Should create successfully
    assert!(std::any::type_name_of_val(&client).contains("NsipClient"));
}

#[test]
fn test_client_with_custom_url() {
    let client = NsipClient::with_base_url("https://example.com");
    // Should create successfully with custom URL
    assert!(std::any::type_name_of_val(&client).contains("NsipClient"));
}

#[test]
fn test_search_criteria_builder() {
    let criteria = SearchCriteria::new()
        .with_breed_group("Sheep")
        .with_status("Active")
        .with_query("test")
        .with_page(1)
        .with_per_page(20);

    assert_eq!(criteria.breed_group, Some("Sheep".to_string()));
    assert_eq!(criteria.status, Some("Active".to_string()));
    assert_eq!(criteria.query, Some("test".to_string()));
    assert_eq!(criteria.page, Some(1));
    assert_eq!(criteria.per_page, Some(20));
}

#[test]
fn test_search_criteria_default() {
    let criteria = SearchCriteria::default();
    assert!(criteria.breed_group.is_none());
    assert!(criteria.status.is_none());
    assert!(criteria.query.is_none());
    assert!(criteria.page.is_none());
    assert!(criteria.per_page.is_none());
}

#[test]
fn test_error_types() {
    // Test InvalidInput error
    let err = Error::InvalidInput("test message".to_string());
    let display = format!("{err}");
    assert!(display.contains("invalid input"));
    assert!(display.contains("test message"));

    // Test ApiError
    let err = Error::ApiError("API failed".to_string());
    let display = format!("{err}");
    assert!(display.contains("API error"));
    assert!(display.contains("API failed"));

    // Test ParseError
    let err = Error::ParseError("parse failed".to_string());
    let display = format!("{err}");
    assert!(display.contains("parse error"));
    assert!(display.contains("parse failed"));

    // Test OperationFailed error
    let err = Error::OperationFailed {
        operation: "fetch".to_string(),
        cause: "network error".to_string(),
    };
    let display = format!("{err}");
    assert!(display.contains("fetch"));
    assert!(display.contains("network error"));
}

mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn search_criteria_builder_preserves_values(
            breed_group in any::<Option<String>>(),
            status in any::<Option<String>>(),
            query in any::<Option<String>>(),
        ) {
            let mut criteria = SearchCriteria::new();

            if let Some(bg) = breed_group.clone() {
                criteria = criteria.with_breed_group(bg);
            }
            if let Some(s) = status.clone() {
                criteria = criteria.with_status(s);
            }
            if let Some(q) = query.clone() {
                criteria = criteria.with_query(q);
            }

            prop_assert_eq!(criteria.breed_group, breed_group);
            prop_assert_eq!(criteria.status, status);
            prop_assert_eq!(criteria.query, query);
        }
    }
}
