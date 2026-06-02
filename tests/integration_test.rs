//! Integration tests for NSIP Search API client.

use nsip::{Error, NsipClient, SearchCriteria};

#[test]
fn test_client_creation() {
    let client = NsipClient::new();
    assert_eq!(client.base_url(), "http://nsipsearch.nsip.org/api");
}

#[test]
fn test_client_with_custom_url() {
    let client = NsipClient::with_base_url("http://example.com");
    assert_eq!(client.base_url(), "http://example.com");
}

#[test]
fn test_client_builder() {
    let client = NsipClient::builder()
        .base_url("http://test.local/api")
        .timeout_secs(60)
        .max_retries(5)
        .build()
        .unwrap();

    assert_eq!(client.base_url(), "http://test.local/api");
}

#[test]
fn test_search_criteria_builder() {
    let criteria = SearchCriteria::new()
        .with_breed_id(486)
        .with_breed_group_id(61)
        .with_status("CURRENT")
        .with_gender("Female")
        .with_proven_only(true)
        .with_flock_id("FLOCK1");

    assert_eq!(criteria.breed_id, Some(486));
    assert_eq!(criteria.breed_group_id, Some(61));
    assert_eq!(criteria.status.as_deref(), Some("CURRENT"));
    assert_eq!(criteria.gender.as_deref(), Some("Female"));
    assert_eq!(criteria.proven_only, Some(true));
    assert_eq!(criteria.flock_id.as_deref(), Some("FLOCK1"));
}

#[test]
fn test_search_criteria_default() {
    let criteria = SearchCriteria::default();
    assert!(criteria.breed_group_id.is_none());
    assert!(criteria.breed_id.is_none());
    assert!(criteria.status.is_none());
    assert!(criteria.gender.is_none());
    assert!(criteria.proven_only.is_none());
    assert!(criteria.flock_id.is_none());
    assert!(criteria.trait_ranges.is_none());
}

#[test]
fn test_error_types() {
    // Test Validation error
    let err = Error::validation("bad input");
    let display = format!("{err}");
    assert!(display.contains("validation error"));
    assert!(display.contains("bad input"));

    // Test Api error
    let err = Error::api(500, "server error");
    let display = format!("{err}");
    assert!(display.contains("500"));
    assert!(display.contains("server error"));

    // Test NotFound error
    let err = Error::NotFound("animal not found".to_string());
    let display = format!("{err}");
    assert!(display.contains("not found"));

    // Test Timeout error
    let err = Error::timeout("30s exceeded");
    let display = format!("{err}");
    assert!(display.contains("timed out"));

    // Test Connection error
    let err = Error::connection("refused");
    let display = format!("{err}");
    assert!(display.contains("connection error"));

    // Test Parse error
    let err = Error::parse("invalid json");
    let display = format!("{err}");
    assert!(display.contains("parse error"));
}

#[test]
fn test_search_criteria_serialization() {
    let criteria = SearchCriteria::new()
        .with_breed_group_id(61)
        .with_breed_id(486)
        .with_status("CURRENT");

    let json = serde_json::to_value(&criteria).unwrap();
    assert_eq!(json["breedGroupId"], 61);
    assert_eq!(json["breedId"], 486);
    assert_eq!(json["status"], "CURRENT");
    // None fields should be absent
    assert!(json.get("gender").is_none());
    assert!(json.get("provenOnly").is_none());
}

#[test]
fn test_animal_details_nested_response() {
    let json = serde_json::json!({
        "success": true,
        "data": {
            "progenyCount": 6,
            "dateOfBirth": "01/15/2020",
            "gender": "Female",
            "genotyped": "Yes",
            "flockCount": "2",
            "breed": { "breedName": "Katahdin", "breedId": 640 },
            "searchResultViewModel": {
                "lpnId": "6####92020###249",
                "lpnSre": "SIRE123",
                "lpnDam": "DAM456",
                "status": "CURRENT",
                "regNumber": "REG789",
                "bwt": 0.246,
                "accbwt": 0.80
            },
            "contactInfo": {
                "farmName": "Test Farm",
                "customerName": "John Doe",
                "phone": "555-1234"
            }
        }
    });

    let details = nsip::AnimalDetails::from_api_response(&json).unwrap();
    assert_eq!(details.lpn_id, "6####92020###249");
    assert_eq!(details.breed.as_deref(), Some("Katahdin"));
    assert_eq!(details.gender.as_deref(), Some("Female"));
    assert_eq!(details.total_progeny, Some(6));
    assert_eq!(details.flock_count, Some(2));
    assert_eq!(details.sire.as_deref(), Some("SIRE123"));
    assert_eq!(details.dam.as_deref(), Some("DAM456"));
    assert_eq!(details.registration_number.as_deref(), Some("REG789"));

    let bwt = details.traits.get("BWT").unwrap();
    assert!((bwt.value - 0.246).abs() < f64::EPSILON);
    assert_eq!(bwt.accuracy, Some(80));

    let contact = details.contact_info.unwrap();
    assert_eq!(contact.farm_name.as_deref(), Some("Test Farm"));
    assert_eq!(contact.contact_name.as_deref(), Some("John Doe"));
    assert_eq!(contact.phone.as_deref(), Some("555-1234"));
}

#[test]
fn test_lineage_response() {
    let json = serde_json::json!({
        "data": {
            "lpnId": "SUBJECT1",
            "content": "<div>My Farm</div><div>US Hair Index: 105.2</div><div>DOB: 1/1/2020</div><div>Sex: Female</div><div>Status: CURRENT</div>",
            "children": [
                {
                    "lpnId": "SIRE1",
                    "content": "<div>Sire Farm</div><div>DOB: 3/15/2018</div><div>Sex: Male</div>",
                    "children": []
                },
                {
                    "lpnId": "DAM1",
                    "content": "<div>Dam Farm</div><div>DOB: 6/20/2017</div><div>Sex: Female</div>",
                    "children": []
                }
            ]
        }
    });

    let lineage = nsip::Lineage::from_api_response(&json).unwrap();
    let subject = lineage.subject.unwrap();
    assert_eq!(subject.lpn_id, "SUBJECT1");
    assert_eq!(subject.farm_name.as_deref(), Some("My Farm"));
    assert!((subject.us_index.unwrap() - 105.2).abs() < f64::EPSILON);

    assert_eq!(lineage.sire.unwrap().lpn_id, "SIRE1");
    assert_eq!(lineage.dam.unwrap().lpn_id, "DAM1");
    assert_eq!(lineage.generations.len(), 1);
    assert_eq!(lineage.generations[0].len(), 2);
}

#[test]
fn test_progeny_response() {
    let json = serde_json::json!({
        "recordCount": 3,
        "records": [
            { "lpnId": "P1", "sex": "Male", "dob": "03/10/2022" },
            { "lpnId": "P2", "sex": "Female", "dob": "04/01/2022" }
        ]
    });

    let progeny = nsip::Progeny::from_api_response(&json, 0, 10).unwrap();
    assert_eq!(progeny.total_count, 3);
    assert_eq!(progeny.animals.len(), 2);
    assert_eq!(progeny.animals[0].lpn_id, "P1");
    assert_eq!(progeny.animals[0].sex.as_deref(), Some("Male"));
    assert_eq!(progeny.animals[1].lpn_id, "P2");
}

#[test]
fn test_search_results_response() {
    let json = serde_json::json!({
        "TotalCount": 42,
        "Results": [
            { "LpnId": "A1" },
            { "LpnId": "A2" }
        ]
    });

    let results = nsip::SearchResults::from_api_response(&json, 0, 15).unwrap();
    assert_eq!(results.total_count, 42);
    assert_eq!(results.results.len(), 2);
    assert_eq!(results.page, 0);
    assert_eq!(results.page_size, 15);
}

mod property_tests {
    use nsip::{Error, SearchCriteria};
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn search_criteria_builder_preserves_values(
            breed_id in any::<Option<i64>>(),
            status in any::<Option<String>>(),
            gender in any::<Option<String>>(),
        ) {
            let mut criteria = SearchCriteria::new();

            if let Some(bid) = breed_id {
                criteria = criteria.with_breed_id(bid);
            }
            if let Some(s) = status.clone() {
                criteria = criteria.with_status(s);
            }
            if let Some(g) = gender.clone() {
                criteria = criteria.with_gender(g);
            }

            prop_assert_eq!(criteria.breed_id, breed_id);
            prop_assert_eq!(criteria.status, status);
            prop_assert_eq!(criteria.gender, gender);
        }

        /// For any HTTP status, an `Api` error produces a complete RFC 9457
        /// envelope: mandatory members present, status mirrored, exit code in
        /// the committed set, and a compact (<1 KB) JSON payload.
        #[test]
        fn api_envelope_complete_for_any_status(status in 400u16..600, msg in ".{0,64}") {
            let pd = Error::api(status, msg).to_problem_details("prop");
            prop_assert_eq!(pd.status, status);
            prop_assert!(!pd.title.is_empty());
            prop_assert!(!pd.detail.is_empty());
            prop_assert!(pd.instance.starts_with("urn:nsip:prop:"));
            prop_assert_eq!(&pd.type_uri[pd.type_uri.len() - 3..], ".md");
            prop_assert_eq!(&pd.docs_url, &pd.type_uri);
            prop_assert!(matches!(pd.exit_code, 1 | 75));
            let json = serde_json::to_string(&pd).expect("serialize");
            prop_assert!(json.len() <= 1024, "payload {} bytes", json.len());
        }
    }
}
