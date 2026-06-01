//! Data models for the NSIP Search API.
//!
//! These types map to the JSON responses from `nsipsearch.nsip.org/api`.
//! The API uses a mix of `camelCase` and `PascalCase` field names depending
//! on the endpoint; serde aliases handle both conventions transparently.

use std::collections::HashMap;

use regex::Regex;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Search criteria (request body for POST search)
// ---------------------------------------------------------------------------

/// Criteria for the animal search endpoint.
///
/// Uses a builder pattern so callers can set only the fields they care about.
///
/// # Examples
///
/// ```rust
/// use nsip::SearchCriteria;
///
/// let criteria = SearchCriteria::new()
///     .with_breed_id(486)
///     .with_status("CURRENT")
///     .with_gender("Female");
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchCriteria {
    /// Breed group identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breed_group_id: Option<i64>,

    /// Breed identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breed_id: Option<i64>,

    /// Only return animals born after this date (`YYYY-MM-DD`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub born_after: Option<String>,

    /// Only return animals born before this date (`YYYY-MM-DD`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub born_before: Option<String>,

    /// Gender filter: `"Male"`, `"Female"`, or `"Both"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,

    /// Only return proven animals.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proven_only: Option<bool>,

    /// Animal status: `"CURRENT"`, `"SOLD"`, `"DEAD"`, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Flock identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flock_id: Option<String>,

    /// Per-trait min/max ranges, e.g. `{"BWT": {"min": -1.0, "max": 1.0}}`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trait_ranges: Option<HashMap<String, TraitRangeFilter>>,
}

/// Min/max bounds for a single trait filter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitRangeFilter {
    /// Minimum value (inclusive).
    pub min: f64,
    /// Maximum value (inclusive).
    pub max: f64,
}

impl SearchCriteria {
    /// Creates an empty search criteria.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            breed_group_id: None,
            breed_id: None,
            born_after: None,
            born_before: None,
            gender: None,
            proven_only: None,
            status: None,
            flock_id: None,
            trait_ranges: None,
        }
    }

    /// Sets the breed group identifier.
    #[must_use]
    pub const fn with_breed_group_id(mut self, id: i64) -> Self {
        self.breed_group_id = Some(id);
        self
    }

    /// Sets the breed identifier.
    #[must_use]
    pub const fn with_breed_id(mut self, id: i64) -> Self {
        self.breed_id = Some(id);
        self
    }

    /// Sets the born-after date filter (`YYYY-MM-DD`).
    #[must_use]
    pub fn with_born_after(mut self, date: impl Into<String>) -> Self {
        self.born_after = Some(date.into());
        self
    }

    /// Sets the born-before date filter (`YYYY-MM-DD`).
    #[must_use]
    pub fn with_born_before(mut self, date: impl Into<String>) -> Self {
        self.born_before = Some(date.into());
        self
    }

    /// Sets the gender filter (`"Male"`, `"Female"`, or `"Both"`).
    #[must_use]
    pub fn with_gender(mut self, gender: impl Into<String>) -> Self {
        self.gender = Some(gender.into());
        self
    }

    /// Restricts results to proven animals only.
    #[must_use]
    pub const fn with_proven_only(mut self, proven: bool) -> Self {
        self.proven_only = Some(proven);
        self
    }

    /// Sets the animal status filter.
    #[must_use]
    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    /// Sets the flock identifier filter.
    #[must_use]
    pub fn with_flock_id(mut self, flock_id: impl Into<String>) -> Self {
        self.flock_id = Some(flock_id.into());
        self
    }

    /// Sets the trait range filters.
    #[must_use]
    pub fn with_trait_ranges(mut self, ranges: HashMap<String, TraitRangeFilter>) -> Self {
        self.trait_ranges = Some(ranges);
        self
    }
}

// ---------------------------------------------------------------------------
// Breed groups
// ---------------------------------------------------------------------------

/// A single breed within a breed group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breed {
    /// Breed identifier.
    pub id: i64,
    /// Human-readable breed name.
    pub name: String,
}

/// A breed group containing one or more breeds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreedGroup {
    /// Breed group identifier.
    pub id: i64,
    /// Human-readable group name.
    pub name: String,
    /// Breeds belonging to this group.
    pub breeds: Vec<Breed>,
}

// ---------------------------------------------------------------------------
// Trait information
// ---------------------------------------------------------------------------

/// A single EBV trait with value and optional accuracy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trait {
    /// Trait abbreviation (e.g. `"BWT"`, `"WWT"`).
    pub name: String,
    /// Estimated breeding value.
    pub value: f64,
    /// Accuracy percentage (0–100), if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accuracy: Option<i32>,
    /// Unit of measurement, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub units: Option<String>,
}

/// Min/max range for a trait within a breed (returned by `getTraitRangesByBreed`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitRange {
    /// Trait abbreviation.
    pub trait_name: String,
    /// Minimum observed value.
    pub min_value: f64,
    /// Maximum observed value.
    pub max_value: f64,
    /// Unit of measurement.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
}

// ---------------------------------------------------------------------------
// Contact info
// ---------------------------------------------------------------------------

/// Contact information for an animal's owner / flock.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContactInfo {
    /// Farm or ranch name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub farm_name: Option<String>,
    /// Contact person's name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_name: Option<String>,
    /// Phone number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    /// Email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Street address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    /// City.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    /// State / province.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// ZIP / postal code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zip_code: Option<String>,
}

// ---------------------------------------------------------------------------
// Animal details
// ---------------------------------------------------------------------------

/// Full detail record for a single animal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimalDetails {
    /// LPN identifier.
    pub lpn_id: String,
    /// Breed name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breed: Option<String>,
    /// Breed group name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breed_group: Option<String>,
    /// Date of birth (string as returned by the API).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_of_birth: Option<String>,
    /// Gender: `"Male"` or `"Female"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    /// Status: `"CURRENT"`, `"SOLD"`, `"DEAD"`, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Sire LPN identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sire: Option<String>,
    /// Dam LPN identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dam: Option<String>,
    /// Registration number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration_number: Option<String>,
    /// Total number of progeny.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_progeny: Option<i64>,
    /// Number of flocks this animal appears in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flock_count: Option<i64>,
    /// Genotyped status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genotyped: Option<String>,
    /// EBV traits keyed by abbreviation.
    pub traits: HashMap<String, Trait>,
    /// Owner / flock contact information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_info: Option<ContactInfo>,
}

/// Mapping from API `searchResultViewModel` trait field names to canonical
/// abbreviations and their corresponding accuracy field names.
const TRAIT_MAPPING: &[(&str, &str, &str)] = &[
    ("bwt", "BWT", "accbwt"),
    ("wwt", "WWT", "accwwt"),
    ("pwwt", "PWWT", "accpwwt"),
    ("ywt", "YWT", "accywt"),
    ("fat", "FAT", "accfat"),
    ("emd", "EMD", "accemd"),
    ("nlb", "NLB", "accnlb"),
    ("nwt", "NWT", "accnwt"),
    ("pwt", "PWT", "accpwt"),
    ("dag", "DAG", "accdag"),
    ("wgr", "WGR", "accwgr"),
    ("wec", "WEC", "accwec"),
    ("fec", "FEC", "accfec"),
];

/// Convert a raw accuracy value to an integer percentage.
///
/// Values <= 1.0 are treated as fractions and multiplied by 100.
#[allow(clippy::cast_possible_truncation)]
fn convert_accuracy(acc: f64) -> i32 {
    if acc <= 1.0 {
        (acc * 100.0) as i32
    } else {
        acc as i32
    }
}

/// Extract EBV traits from the nested `searchResultViewModel` object.
fn extract_traits_nested(
    sr: &serde_json::Map<String, serde_json::Value>,
) -> HashMap<String, Trait> {
    let mut traits = HashMap::new();
    for &(trait_key, trait_name, acc_key) in TRAIT_MAPPING {
        let Some(val) = sr.get(trait_key).and_then(serde_json::Value::as_f64) else {
            continue;
        };
        let accuracy = sr
            .get(acc_key)
            .and_then(serde_json::Value::as_f64)
            .map(convert_accuracy);
        traits.insert(
            trait_name.to_string(),
            Trait {
                name: trait_name.to_string(),
                value: val,
                accuracy,
                units: None,
            },
        );
    }
    traits
}

/// Extract contact information from a JSON value, checking both camelCase and `PascalCase` keys.
fn extract_contact_info(c: &serde_json::Value) -> Option<ContactInfo> {
    if c.is_null() || !c.is_object() {
        return None;
    }
    Some(ContactInfo {
        farm_name: c
            .get("farmName")
            .or_else(|| c.get("FarmName"))
            .and_then(serde_json::Value::as_str)
            .map(String::from),
        contact_name: c
            .get("customerName")
            .or_else(|| c.get("ContactName"))
            .and_then(serde_json::Value::as_str)
            .map(String::from),
        phone: c
            .get("phone")
            .or_else(|| c.get("Phone"))
            .and_then(serde_json::Value::as_str)
            .map(String::from),
        email: c
            .get("email")
            .or_else(|| c.get("Email"))
            .and_then(serde_json::Value::as_str)
            .map(String::from),
        address: c
            .get("address")
            .or_else(|| c.get("Address"))
            .and_then(serde_json::Value::as_str)
            .map(String::from),
        city: c
            .get("city")
            .or_else(|| c.get("City"))
            .and_then(serde_json::Value::as_str)
            .map(String::from),
        state: c
            .get("state")
            .or_else(|| c.get("State"))
            .and_then(serde_json::Value::as_str)
            .map(String::from),
        zip_code: c
            .get("zipCode")
            .or_else(|| c.get("ZipCode"))
            .and_then(serde_json::Value::as_str)
            .map(String::from),
    })
}

/// Extract traits from the legacy `PascalCase` format.
#[allow(clippy::cast_possible_truncation)]
fn extract_traits_legacy(data: &serde_json::Value) -> HashMap<String, Trait> {
    let Some(obj) = data.get("Traits").and_then(serde_json::Value::as_object) else {
        return HashMap::new();
    };
    let mut traits = HashMap::new();
    for (name, td) in obj {
        let Some(td_obj) = td.as_object() else {
            continue;
        };
        let value = td_obj
            .get("Value")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        let accuracy = td_obj
            .get("Accuracy")
            .and_then(|a| a.as_f64().map(|v| v as i32));
        traits.insert(
            name.clone(),
            Trait {
                name: name.clone(),
                value,
                accuracy,
                units: None,
            },
        );
    }
    traits
}

/// Extract trait values from a progeny record.
fn extract_progeny_traits(item: &serde_json::Value) -> HashMap<String, f64> {
    let Some(obj) = item.get("Traits").and_then(serde_json::Value::as_object) else {
        return HashMap::new();
    };
    obj.iter()
        .filter_map(|(k, v)| v.as_f64().map(|f| (k.clone(), f)))
        .collect()
}

impl AnimalDetails {
    /// Parse an `AnimalDetails` from the raw JSON value returned by the API.
    ///
    /// Supports both the nested format (`{ "success": true, "data": { ... } }`)
    /// and a legacy flat `PascalCase` format.
    ///
    /// # Errors
    ///
    /// Returns `crate::Error::Parse` if the response cannot be interpreted.
    pub fn from_api_response(data: &serde_json::Value) -> crate::Result<Self> {
        let is_nested = data
            .get("data")
            .and_then(serde_json::Value::as_object)
            .is_some();

        if is_nested {
            Ok(Self::from_nested_format(data))
        } else if Self::has_string_identity(data, "lpnId") {
            // Search result row: camelCase fields with inline trait values
            Ok(Self::from_search_result(data))
        } else if Self::has_string_identity(data, "LpnId") {
            // Legacy flat PascalCase format.
            Ok(Self::from_legacy_format(data))
        } else {
            // No recognized identity field: a 200 body that is not an animal
            // record (including one where the identity key is present but
            // `null`, non-string, or empty). Fail loudly instead of returning a
            // record with an empty `lpn_id` and zeroed traits masquerading as
            // valid.
            Err(crate::Error::parse(
                "animal details response missing identity field \
                 (expected `data`, `lpnId`, or `LpnId`)",
            ))
        }
    }

    /// Whether `key` is present on `data` as a non-empty string. Used to detect
    /// the response shape: an explicit `null`, a non-string, or an empty string
    /// is not a usable identity and must not be treated as one.
    fn has_string_identity(data: &serde_json::Value, key: &str) -> bool {
        data.get(key)
            .and_then(serde_json::Value::as_str)
            .is_some_and(|s| !s.is_empty())
    }

    fn from_nested_format(data: &serde_json::Value) -> Self {
        let section = &data["data"];
        let sr = &section["searchResultViewModel"];
        let breed_obj = &section["breed"];
        let contact_obj = section
            .get("contactInfo")
            .or_else(|| section.get("ContactInfo"));

        let lpn_id = sr["lpnId"].as_str().unwrap_or_default().to_string();
        let breed = breed_obj
            .get("breedName")
            .and_then(serde_json::Value::as_str)
            .map(String::from);
        let date_of_birth = section
            .get("dateOfBirth")
            .and_then(serde_json::Value::as_str)
            .map(String::from);
        let gender = section
            .get("gender")
            .and_then(serde_json::Value::as_str)
            .map(String::from);
        let status = sr
            .get("status")
            .and_then(serde_json::Value::as_str)
            .map(String::from);
        let sire = sr
            .get("lpnSre")
            .and_then(serde_json::Value::as_str)
            .map(String::from);
        let dam = sr
            .get("lpnDam")
            .and_then(serde_json::Value::as_str)
            .map(String::from);
        let registration_number = sr
            .get("regNumber")
            .and_then(serde_json::Value::as_str)
            .map(String::from);
        let total_progeny = section
            .get("progenyCount")
            .and_then(serde_json::Value::as_i64);
        let genotyped = section
            .get("genotyped")
            .and_then(serde_json::Value::as_str)
            .map(String::from);

        let flock_count = section.get("flockCount").and_then(|v| {
            v.as_i64()
                .or_else(|| v.as_str().and_then(|s| s.parse::<i64>().ok()))
        });

        // Extract traits from searchResultViewModel
        let traits = sr
            .as_object()
            .map(extract_traits_nested)
            .unwrap_or_default();

        let contact_info = contact_obj.and_then(extract_contact_info);

        Self {
            lpn_id,
            breed,
            breed_group: None,
            date_of_birth,
            gender,
            status,
            sire,
            dam,
            registration_number,
            total_progeny,
            flock_count,
            genotyped,
            traits,
            contact_info,
        }
    }

    /// Parse from a search result row (camelCase fields with inline trait values).
    ///
    /// Search results from `getPageOfSearchResults` use camelCase keys like
    /// `lpnId`, `bwt`, `accbwt` — the same schema as `searchResultViewModel`
    /// but at the top level without the `data` wrapper.
    fn from_search_result(data: &serde_json::Value) -> Self {
        let lpn_id = data
            .get("lpnId")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string();

        let traits = data
            .as_object()
            .map(extract_traits_nested)
            .unwrap_or_default();

        Self {
            lpn_id,
            breed: None,
            breed_group: None,
            date_of_birth: data
                .get("dateOfBirth")
                .and_then(serde_json::Value::as_str)
                .map(String::from),
            gender: data
                .get("gender")
                .and_then(serde_json::Value::as_str)
                .map(String::from),
            status: data
                .get("status")
                .and_then(serde_json::Value::as_str)
                .map(String::from),
            sire: data
                .get("lpnSre")
                .and_then(serde_json::Value::as_str)
                .map(String::from),
            dam: data
                .get("lpnDam")
                .and_then(serde_json::Value::as_str)
                .map(String::from),
            registration_number: data
                .get("regNumber")
                .and_then(serde_json::Value::as_str)
                .map(String::from),
            total_progeny: None,
            flock_count: None,
            genotyped: None,
            traits,
            contact_info: None,
        }
    }

    fn from_legacy_format(data: &serde_json::Value) -> Self {
        let lpn_id = data
            .get("LpnId")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string();

        let traits = extract_traits_legacy(data);
        let contact_info = data.get("ContactInfo").and_then(extract_contact_info);

        Self {
            lpn_id,
            breed: data
                .get("Breed")
                .and_then(serde_json::Value::as_str)
                .map(String::from),
            breed_group: data
                .get("BreedGroup")
                .and_then(serde_json::Value::as_str)
                .map(String::from),
            date_of_birth: data
                .get("DateOfBirth")
                .and_then(serde_json::Value::as_str)
                .map(String::from),
            gender: data
                .get("Gender")
                .and_then(serde_json::Value::as_str)
                .map(String::from),
            status: data
                .get("Status")
                .and_then(serde_json::Value::as_str)
                .map(String::from),
            sire: data
                .get("Sire")
                .and_then(serde_json::Value::as_str)
                .map(String::from),
            dam: data
                .get("Dam")
                .and_then(serde_json::Value::as_str)
                .map(String::from),
            registration_number: data
                .get("RegistrationNumber")
                .and_then(serde_json::Value::as_str)
                .map(String::from),
            total_progeny: data.get("TotalProgeny").and_then(serde_json::Value::as_i64),
            flock_count: data.get("FlockCount").and_then(serde_json::Value::as_i64),
            genotyped: data
                .get("Genotyped")
                .and_then(serde_json::Value::as_str)
                .map(String::from),
            traits,
            contact_info,
        }
    }
}

// ---------------------------------------------------------------------------
// Progeny
// ---------------------------------------------------------------------------

/// A single offspring in a progeny response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgenyAnimal {
    /// LPN identifier.
    pub lpn_id: String,
    /// Sex of the animal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sex: Option<String>,
    /// Date of birth.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_of_birth: Option<String>,
    /// Trait values keyed by abbreviation.
    pub traits: HashMap<String, f64>,
}

/// Paginated progeny result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progeny {
    /// Total number of offspring.
    pub total_count: i64,
    /// Offspring on this page.
    pub animals: Vec<ProgenyAnimal>,
    /// Current page number (0-indexed).
    pub page: u32,
    /// Page size.
    pub page_size: u32,
}

impl Progeny {
    /// Parse a `Progeny` from the raw JSON value returned by the API.
    ///
    /// The progeny endpoint uses `records` / `recordCount` (lowercase) rather
    /// than the `Results` / `TotalCount` convention used elsewhere.
    ///
    /// # Errors
    ///
    /// Currently infallible — missing or malformed fields degrade to defaults
    /// and an empty result set is valid. Returns [`crate::Result`] for API
    /// consistency with the other model constructors and forward compatibility.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn from_api_response(
        data: &serde_json::Value,
        page: u32,
        page_size: u32,
    ) -> crate::Result<Self> {
        let records = data
            .get("records")
            .or_else(|| data.get("Results"))
            .and_then(serde_json::Value::as_array);

        let mut animals = Vec::new();
        if let Some(arr) = records {
            for item in arr {
                let lpn_id = item
                    .get("lpnId")
                    .or_else(|| item.get("LpnId"))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string();

                let sex = item
                    .get("sex")
                    .or_else(|| item.get("Sex"))
                    .and_then(serde_json::Value::as_str)
                    .map(String::from);

                let date_of_birth = item
                    .get("dob")
                    .or_else(|| item.get("DateOfBirth"))
                    .and_then(serde_json::Value::as_str)
                    .map(String::from);

                let traits = extract_progeny_traits(item);

                animals.push(ProgenyAnimal {
                    lpn_id,
                    sex,
                    date_of_birth,
                    traits,
                });
            }
        }

        let total_count = data
            .get("recordCount")
            .or_else(|| data.get("TotalCount"))
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0);

        Ok(Self {
            total_count,
            animals,
            page,
            page_size,
        })
    }
}

// ---------------------------------------------------------------------------
// Lineage
// ---------------------------------------------------------------------------

/// A single animal node in the pedigree tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageAnimal {
    /// LPN identifier.
    pub lpn_id: String,
    /// Farm name (parsed from HTML content).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub farm_name: Option<String>,
    /// US (Hair) Index value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub us_index: Option<f64>,
    /// SRC$ Index value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_index: Option<f64>,
    /// Date of birth.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_of_birth: Option<String>,
    /// Sex.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sex: Option<String>,
    /// Status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// Pedigree / lineage information for an animal.
///
/// The API returns a recursive tree where each node has `lpnId`, `content`
/// (HTML), and `children: [sire, dam]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lineage {
    /// The subject animal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<LineageAnimal>,
    /// Sire (father).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sire: Option<LineageAnimal>,
    /// Dam (mother).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dam: Option<LineageAnimal>,
    /// Ancestor generations beyond parents; index 0 = grandparents.
    pub generations: Vec<Vec<LineageAnimal>>,
}

/// Parse structured fields out of the HTML `content` string returned by the
/// lineage endpoint.
///
/// Example content:
/// ```text
/// <div>Farm Name</div><div>US Hair Index: 102.03</div><div>DOB: 2/13/2024</div>
/// ```
fn parse_lineage_content(content: &str) -> ParsedLineageContent {
    // These are cheap to construct; `Regex::new` is the expensive part, but
    // the patterns are simple enough that the cost is negligible here.
    // For hot-path usage consider `std::sync::LazyLock`.
    let re_farm = Regex::new(r"<div>([^<]+)</div>").ok();
    let re_us = Regex::new(r"US (?:Hair )?Index: ([\d.]+)").ok();
    let re_src = Regex::new(r"SRC\$ Index: ([\d.]+)").ok();
    let re_dob = Regex::new(r"DOB: ([^<]+)").ok();
    let re_sex = Regex::new(r"Sex: ([^<]+)").ok();
    let re_status = Regex::new(r"Status: ([^<]+)").ok();

    ParsedLineageContent {
        farm_name: re_farm
            .and_then(|r| r.captures(content))
            .map(|c| c[1].to_string()),
        us_index: re_us
            .and_then(|r| r.captures(content))
            .and_then(|c| c[1].parse().ok()),
        src_index: re_src
            .and_then(|r| r.captures(content))
            .and_then(|c| c[1].parse().ok()),
        date_of_birth: re_dob
            .and_then(|r| r.captures(content))
            .map(|c| c[1].trim().to_string()),
        sex: re_sex
            .and_then(|r| r.captures(content))
            .map(|c| c[1].trim().to_string()),
        status: re_status
            .and_then(|r| r.captures(content))
            .map(|c| c[1].trim().to_string()),
    }
}

struct ParsedLineageContent {
    farm_name: Option<String>,
    us_index: Option<f64>,
    src_index: Option<f64>,
    date_of_birth: Option<String>,
    sex: Option<String>,
    status: Option<String>,
}

/// Parse a single lineage tree node into a [`LineageAnimal`].
fn parse_lineage_node(node: &serde_json::Value) -> Option<LineageAnimal> {
    let lpn_id = node.get("lpnId")?.as_str()?;
    let content = node
        .get("content")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let parsed = parse_lineage_content(content);

    Some(LineageAnimal {
        lpn_id: lpn_id.to_string(),
        farm_name: parsed.farm_name,
        us_index: parsed.us_index,
        src_index: parsed.src_index,
        date_of_birth: parsed.date_of_birth,
        sex: parsed.sex,
        status: parsed.status,
    })
}

/// Recursively collect ancestor generations from the lineage tree.
fn collect_generations(
    node: &serde_json::Value,
    generations: &mut Vec<Vec<LineageAnimal>>,
    depth: usize,
) {
    let Some(children) = node.get("children").and_then(serde_json::Value::as_array) else {
        return;
    };
    if children.is_empty() {
        return;
    }

    while generations.len() <= depth {
        generations.push(Vec::new());
    }

    for child in children {
        if let Some(animal) = parse_lineage_node(child) {
            generations[depth].push(animal);
        }
        collect_generations(child, generations, depth + 1);
    }
}

impl Lineage {
    /// Parse a `Lineage` from the raw JSON value returned by the API.
    ///
    /// # Errors
    ///
    /// Currently infallible — a tree with no parents is valid. Returns
    /// [`crate::Result`] for API consistency with the other model constructors.
    pub fn from_api_response(data: &serde_json::Value) -> crate::Result<Self> {
        let node = if data
            .get("data")
            .and_then(serde_json::Value::as_object)
            .is_some()
        {
            &data["data"]
        } else {
            data
        };

        let subject = parse_lineage_node(node);

        let children = node.get("children").and_then(serde_json::Value::as_array);

        let sire = children
            .and_then(|c| c.first())
            .and_then(parse_lineage_node);

        let dam = children.and_then(|c| c.get(1)).and_then(parse_lineage_node);

        let mut generations = Vec::new();
        collect_generations(node, &mut generations, 0);

        Ok(Self {
            subject,
            sire,
            dam,
            generations,
        })
    }
}

// ---------------------------------------------------------------------------
// Search results
// ---------------------------------------------------------------------------

/// Paginated search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    /// Total number of matching animals.
    pub total_count: i64,
    /// Result rows for the current page (raw JSON objects).
    pub results: Vec<serde_json::Value>,
    /// Current page number (0-indexed).
    pub page: u32,
    /// Page size.
    pub page_size: u32,
}

impl SearchResults {
    /// Parse `SearchResults` from the raw JSON value returned by the API.
    ///
    /// Supports both `PascalCase` (`TotalCount`, `Results`) and `camelCase`
    /// (`recordCount`, `records`) field names.
    ///
    /// # Errors
    ///
    /// Currently infallible — missing or malformed fields degrade to defaults
    /// and an empty result set is valid. Returns [`crate::Result`] for API
    /// consistency with the other model constructors and forward compatibility.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn from_api_response(
        data: &serde_json::Value,
        page: u32,
        page_size: u32,
    ) -> crate::Result<Self> {
        let total_count = data
            .get("TotalCount")
            .or_else(|| data.get("recordCount"))
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0);

        let results = data
            .get("Results")
            .or_else(|| data.get("records"))
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();

        Ok(Self {
            total_count,
            results,
            page,
            page_size,
        })
    }
}

// ---------------------------------------------------------------------------
// Date last updated
// ---------------------------------------------------------------------------

/// Response from the `getDateLastUpdated` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateLastUpdated {
    /// The raw response value.
    pub data: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Breed group API response (raw JSON)
// ---------------------------------------------------------------------------

/// Intermediate struct for deserializing the raw breed-groups API response.
#[derive(Deserialize)]
pub(crate) struct RawBreedGroupResponse {
    #[serde(default)]
    pub data: Option<Vec<RawBreedGroup>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RawBreedGroup {
    #[serde(alias = "Id", alias = "id")]
    pub breed_group_id: Option<i64>,
    #[serde(alias = "Name", alias = "name")]
    pub breed_group_name: Option<String>,
    #[serde(default)]
    pub breeds: Vec<RawBreed>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RawBreed {
    #[serde(alias = "id")]
    pub breed_id: Option<i64>,
    #[serde(alias = "name")]
    pub breed_name: Option<String>,
}

// ---------------------------------------------------------------------------
// Animal profile (combined details + lineage + progeny)
// ---------------------------------------------------------------------------

/// Combined profile returned by [`crate::NsipClient::search_by_lpn`].
#[derive(Debug, Clone, Serialize)]
pub struct AnimalProfile {
    /// Detailed information about the animal.
    pub details: AnimalDetails,
    /// Pedigree / lineage tree.
    pub lineage: Lineage,
    /// Progeny (offspring) list.
    pub progeny: Progeny,
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_criteria_builder_round_trip() {
        let criteria = SearchCriteria::new()
            .with_breed_id(486)
            .with_status("CURRENT")
            .with_gender("Female")
            .with_proven_only(true);

        assert_eq!(criteria.breed_id, Some(486));
        assert_eq!(criteria.status.as_deref(), Some("CURRENT"));
        assert_eq!(criteria.gender.as_deref(), Some("Female"));
        assert_eq!(criteria.proven_only, Some(true));
        assert!(criteria.breed_group_id.is_none());
    }

    #[test]
    fn search_criteria_serializes_to_camel_case() {
        let criteria = SearchCriteria::new()
            .with_breed_group_id(61)
            .with_breed_id(486);

        let json = serde_json::to_value(&criteria).unwrap();
        assert_eq!(json["breedGroupId"], 61);
        assert_eq!(json["breedId"], 486);
        // None fields should be absent
        assert!(json.get("gender").is_none());
    }

    #[test]
    fn animal_details_from_nested_response() {
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
                    "customerName": "John Doe"
                }
            }
        });

        let details = AnimalDetails::from_api_response(&json).unwrap();
        assert_eq!(details.lpn_id, "6####92020###249");
        assert_eq!(details.breed.as_deref(), Some("Katahdin"));
        assert_eq!(details.gender.as_deref(), Some("Female"));
        assert_eq!(details.total_progeny, Some(6));
        assert_eq!(details.flock_count, Some(2));
        assert_eq!(details.sire.as_deref(), Some("SIRE123"));

        let bwt = details.traits.get("BWT").unwrap();
        assert!((bwt.value - 0.246).abs() < f64::EPSILON);
        assert_eq!(bwt.accuracy, Some(80));

        let contact = details.contact_info.unwrap();
        assert_eq!(contact.farm_name.as_deref(), Some("Test Farm"));
        assert_eq!(contact.contact_name.as_deref(), Some("John Doe"));
    }

    #[test]
    fn animal_details_from_legacy_response() {
        let json = serde_json::json!({
            "LpnId": "LEGACY123",
            "Breed": "Targhee",
            "Gender": "Male",
            "Status": "SOLD",
            "Traits": {
                "BWT": { "Value": 1.5, "Accuracy": 90 }
            }
        });

        let details = AnimalDetails::from_api_response(&json).unwrap();
        assert_eq!(details.lpn_id, "LEGACY123");
        assert_eq!(details.breed.as_deref(), Some("Targhee"));
        let bwt = details.traits.get("BWT").unwrap();
        assert!((bwt.value - 1.5).abs() < f64::EPSILON);
        assert_eq!(bwt.accuracy, Some(90));
    }

    #[test]
    fn animal_details_from_search_result() {
        let json = serde_json::json!({
            "lpnId": "6400012006BWR107",
            "lpnSre": "SIRE_SR",
            "lpnDam": "DAM_SR",
            "gender": "Male",
            "status": "CURRENT",
            "dateOfBirth": "3/15/2022",
            "regNumber": "SR_REG",
            "bwt": 0.35,
            "accbwt": 72,
            "wwt": 2.5,
            "accwwt": 68,
            "pwwt": 4.1,
            "accpwwt": 65,
            "ywt": 3.8,
            "accywt": 55,
            "nlb": 0.15,
            "accnlb": 40
        });

        let details = AnimalDetails::from_api_response(&json).unwrap();
        assert_eq!(details.lpn_id, "6400012006BWR107");
        assert_eq!(details.sire.as_deref(), Some("SIRE_SR"));
        assert_eq!(details.dam.as_deref(), Some("DAM_SR"));
        assert_eq!(details.gender.as_deref(), Some("Male"));
        assert_eq!(details.status.as_deref(), Some("CURRENT"));
        assert_eq!(details.date_of_birth.as_deref(), Some("3/15/2022"));
        assert_eq!(details.registration_number.as_deref(), Some("SR_REG"));

        // Verify traits parsed from camelCase inline fields
        let bwt = details.traits.get("BWT").unwrap();
        assert!((bwt.value - 0.35).abs() < f64::EPSILON);
        assert_eq!(bwt.accuracy, Some(72));

        let wwt = details.traits.get("WWT").unwrap();
        assert!((wwt.value - 2.5).abs() < f64::EPSILON);
        assert_eq!(wwt.accuracy, Some(68));

        let pwwt = details.traits.get("PWWT").unwrap();
        assert!((pwwt.value - 4.1).abs() < f64::EPSILON);

        let nlb = details.traits.get("NLB").unwrap();
        assert!((nlb.value - 0.15).abs() < f64::EPSILON);
        assert_eq!(nlb.accuracy, Some(40));

        // Should have 5 traits total
        assert_eq!(details.traits.len(), 5);
    }

    #[test]
    fn animal_details_rejects_body_without_identity() {
        // A 200 body with no `data`, `lpnId`, or `LpnId` is not an animal
        // record; it must fail rather than masquerade as a valid empty record.
        let garbage = serde_json::json!({ "unexpected": "payload" });
        let err = AnimalDetails::from_api_response(&garbage).unwrap_err();
        assert!(matches!(err, crate::Error::Parse { .. }), "got {err:?}");
    }

    #[test]
    fn animal_details_rejects_present_but_unusable_identity() {
        // The identity key may be present yet unusable: an explicit `null`, an
        // empty string, or a non-string. `.is_some()` would treat the key as
        // present and emit an empty-`lpn_id` record — the exact masquerade the
        // identity guard exists to prevent. Each must fail with `Parse`.
        for body in [
            serde_json::json!({ "lpnId": null }),
            serde_json::json!({ "lpnId": "" }),
            serde_json::json!({ "lpnId": 12345 }),
            serde_json::json!({ "LpnId": null }),
        ] {
            let err = AnimalDetails::from_api_response(&body).unwrap_err();
            assert!(
                matches!(err, crate::Error::Parse { .. }),
                "body {body} should be rejected, got {err:?}"
            );
        }

        // A genuine non-empty identity still parses.
        let ok = serde_json::json!({ "lpnId": "REAL1" });
        assert_eq!(
            AnimalDetails::from_api_response(&ok).unwrap().lpn_id,
            "REAL1"
        );
    }

    #[test]
    fn animal_details_accepts_legacy_pascalcase() {
        // A valid legacy record (PascalCase `LpnId`) still parses.
        let legacy = serde_json::json!({ "LpnId": "LEGACY1" });
        let details = AnimalDetails::from_api_response(&legacy).unwrap();
        assert_eq!(details.lpn_id, "LEGACY1");
    }

    #[test]
    fn progeny_from_api_response() {
        let json = serde_json::json!({
            "recordCount": 3,
            "records": [
                { "lpnId": "P1", "sex": "Male", "dob": "03/10/2022" },
                { "lpnId": "P2", "sex": "Female", "dob": "04/01/2022" }
            ]
        });

        let progeny = Progeny::from_api_response(&json, 0, 10).unwrap();
        assert_eq!(progeny.total_count, 3);
        assert_eq!(progeny.animals.len(), 2);
        assert_eq!(progeny.animals[0].lpn_id, "P1");
        assert_eq!(progeny.animals[1].sex.as_deref(), Some("Female"));
    }

    #[test]
    fn lineage_from_api_response() {
        let json = serde_json::json!({
            "data": {
                "lpnId": "SUBJECT1",
                "content": "<div>My Farm</div><div>US Hair Index: 105.2</div><div>DOB: 1/1/2020</div><div>Sex: Female</div>",
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

        let lineage = Lineage::from_api_response(&json).unwrap();
        let subject = lineage.subject.unwrap();
        assert_eq!(subject.lpn_id, "SUBJECT1");
        assert_eq!(subject.farm_name.as_deref(), Some("My Farm"));
        assert!((subject.us_index.unwrap() - 105.2).abs() < f64::EPSILON);
        assert_eq!(subject.sex.as_deref(), Some("Female"));

        let sire = lineage.sire.unwrap();
        assert_eq!(sire.lpn_id, "SIRE1");
        assert_eq!(sire.sex.as_deref(), Some("Male"));

        let dam = lineage.dam.unwrap();
        assert_eq!(dam.lpn_id, "DAM1");
    }

    #[test]
    fn search_results_from_api_response() {
        let json = serde_json::json!({
            "TotalCount": 42,
            "Results": [
                { "LpnId": "A1" },
                { "LpnId": "A2" }
            ]
        });

        let results = SearchResults::from_api_response(&json, 0, 15).unwrap();
        assert_eq!(results.total_count, 42);
        assert_eq!(results.results.len(), 2);
        assert_eq!(results.page, 0);
        assert_eq!(results.page_size, 15);
    }

    #[test]
    fn parse_lineage_html_content() {
        let content = "<div>Happy Acres</div><div>US Hair Index: 98.5</div><div>SRC$ Index: 102.3</div><div>DOB: 2/13/2024</div><div>Sex: Male</div><div>Status: CURRENT</div>";
        let parsed = parse_lineage_content(content);
        assert_eq!(parsed.farm_name.as_deref(), Some("Happy Acres"));
        assert!((parsed.us_index.unwrap() - 98.5).abs() < f64::EPSILON);
        assert!((parsed.src_index.unwrap() - 102.3).abs() < f64::EPSILON);
        assert_eq!(parsed.date_of_birth.as_deref(), Some("2/13/2024"));
        assert_eq!(parsed.sex.as_deref(), Some("Male"));
        assert_eq!(parsed.status.as_deref(), Some("CURRENT"));
    }

    #[test]
    fn trait_range_filter_serializes() {
        let filter = TraitRangeFilter {
            min: -1.0,
            max: 1.0,
        };
        let json = serde_json::to_value(&filter).unwrap();
        assert_eq!(json["min"], -1.0);
        assert_eq!(json["max"], 1.0);
    }

    #[test]
    fn search_criteria_with_trait_ranges() {
        let mut ranges = std::collections::HashMap::new();
        ranges.insert(
            "BWT".to_string(),
            TraitRangeFilter {
                min: -1.0,
                max: 1.0,
            },
        );
        let criteria = SearchCriteria::new().with_trait_ranges(ranges);
        let json = serde_json::to_value(&criteria).unwrap();
        let tr = &json["traitRanges"]["BWT"];
        assert_eq!(tr["min"], -1.0);
        assert_eq!(tr["max"], 1.0);
    }

    #[test]
    fn extract_contact_info_null_returns_none() {
        let json = serde_json::json!({
            "data": {
                "gender": "Male",
                "searchResultViewModel": { "lpnId": "X1" },
                "contactInfo": null
            }
        });
        let details = AnimalDetails::from_api_response(&json).unwrap();
        assert!(details.contact_info.is_none());
    }

    #[test]
    fn legacy_format_missing_traits_key() {
        let json = serde_json::json!({
            "LpnId": "LEG_NO_TRAITS",
            "Breed": "Suffolk",
            "Gender": "Female"
        });
        let details = AnimalDetails::from_api_response(&json).unwrap();
        assert_eq!(details.lpn_id, "LEG_NO_TRAITS");
        assert!(details.traits.is_empty());
    }

    #[test]
    fn legacy_format_non_object_trait_value() {
        let json = serde_json::json!({
            "LpnId": "LEG_BAD_TRAIT",
            "Traits": {
                "BWT": "not an object",
                "WWT": { "Value": 2.0, "Accuracy": 70 }
            }
        });
        let details = AnimalDetails::from_api_response(&json).unwrap();
        // BWT should be skipped, WWT should parse
        assert!(!details.traits.contains_key("BWT"));
        assert!(details.traits.contains_key("WWT"));
    }

    #[test]
    fn lineage_node_without_children() {
        // Lineage response where a node has no "children" key at all
        let json = serde_json::json!({
            "data": {
                "lpnId": "NOCHILDREN",
                "content": "<div>Farm</div>"
            }
        });
        let lineage = Lineage::from_api_response(&json).unwrap();
        assert!(lineage.sire.is_none());
        assert!(lineage.dam.is_none());
    }

    #[test]
    fn breed_group_serializes() {
        let bg = BreedGroup {
            id: 61,
            name: "Range".to_string(),
            breeds: vec![Breed {
                id: 486,
                name: "South African Meat Merino".to_string(),
            }],
        };

        let json = serde_json::to_value(&bg).unwrap();
        assert_eq!(json["id"], 61);
        assert_eq!(json["breeds"][0]["name"], "South African Meat Merino");
    }
}
