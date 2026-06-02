//! Human-readable formatting for CLI output.
//!
//! Every `fmt_*` function takes a model struct and returns a `String` suitable
//! for printing to the terminal. The [`Table`] helper renders ASCII tables with
//! `+-|` borders and auto-calculated column widths.

use std::fmt::Write as _;

use nsip::{
    AnimalDetails, AnimalProfile, BreedGroup, Lineage, LineageAnimal, Progeny, SearchResults, Trait,
};

// ---------------------------------------------------------------------------
// Table renderer
// ---------------------------------------------------------------------------

/// Simple ASCII table with `+-|` borders.
struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Table {
    const fn new(headers: Vec<String>) -> Self {
        Self {
            headers,
            rows: Vec::new(),
        }
    }

    fn add_row(&mut self, row: Vec<String>) {
        self.rows.push(row);
    }

    fn render(&self) -> String {
        let col_count = self.headers.len();
        let mut widths = vec![0usize; col_count];

        for (i, h) in self.headers.iter().enumerate() {
            widths[i] = widths[i].max(h.len());
        }
        for row in &self.rows {
            for (i, cell) in row.iter().take(col_count).enumerate() {
                widths[i] = widths[i].max(cell.len());
            }
        }

        let mut out = String::new();
        let sep = separator_line(&widths);

        out.push_str(&sep);
        out.push('\n');
        out.push_str(&format_row(&self.headers, &widths));
        out.push('\n');
        out.push_str(&sep);
        out.push('\n');

        for row in &self.rows {
            out.push_str(&format_row(row, &widths));
            out.push('\n');
        }

        out.push_str(&sep);
        out
    }
}

fn separator_line(widths: &[usize]) -> String {
    let mut s = String::from("+");
    for w in widths {
        s.push_str(&"-".repeat(w + 2));
        s.push('+');
    }
    s
}

fn format_row(cells: &[String], widths: &[usize]) -> String {
    let mut s = String::from("|");
    for (i, w) in widths.iter().enumerate() {
        let cell = cells.get(i).map_or("", String::as_str);
        let _ = write!(s, " {cell:<w$} |", w = *w);
    }
    s
}

// ---------------------------------------------------------------------------
// Trait display helpers
// ---------------------------------------------------------------------------

/// Canonical ordering for traits in tables.
const TRAIT_ORDER: &[&str] = &[
    "BWT", "WWT", "PWWT", "YWT", "MWWT", "NLB", "NLW", "PEMD", "PFAT", "YEMD", "YFAT", "WFEC",
    "PFEC", "YFD", "YGFW", "YSL",
];

/// Compare two trait names by canonical position.
fn trait_sort_key(name: &str) -> usize {
    TRAIT_ORDER
        .iter()
        .position(|&t| t == name)
        .unwrap_or(usize::MAX)
}

/// Return traits sorted in canonical order.
fn sorted_traits(traits: &std::collections::HashMap<String, Trait>) -> Vec<&Trait> {
    let mut ordered: Vec<&Trait> = Vec::new();
    for &name in TRAIT_ORDER {
        if let Some(t) = traits.get(name) {
            ordered.push(t);
        }
    }
    let mut extra: Vec<&Trait> = traits
        .values()
        .filter(|t| !TRAIT_ORDER.contains(&t.name.as_str()))
        .collect();
    extra.sort_by(|a, b| a.name.cmp(&b.name));
    ordered.extend(extra);
    ordered
}

/// Collect unique trait names from an iterator of key sets, sorted canonically.
fn collect_sorted_trait_names<'a>(
    keys_iter: impl Iterator<Item = impl Iterator<Item = &'a String>>,
) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for keys in keys_iter {
        for name in keys {
            if !names.contains(name) {
                names.push(name.clone());
            }
        }
    }
    names.sort_by_key(|a| trait_sort_key(a));
    names
}

// ---------------------------------------------------------------------------
// Lineage helpers
// ---------------------------------------------------------------------------

/// Format a lineage animal node as a one-line summary.
fn lineage_node_summary(animal: &LineageAnimal) -> String {
    let mut parts = vec![animal.lpn_id.clone()];
    if let Some(ref farm) = animal.farm_name {
        parts.push(format!("({farm})"));
    }
    if let Some(idx) = animal.us_index {
        parts.push(format!("US:{idx:.1}"));
    }
    if let Some(idx) = animal.src_index {
        parts.push(format!("SRC$:{idx:.1}"));
    }
    if let Some(ref dob) = animal.date_of_birth {
        parts.push(format!("DOB:{dob}"));
    }
    if let Some(ref status) = animal.status {
        parts.push(status.clone());
    }
    parts.join(" ")
}

/// Pick a tree connector based on whether a sibling follows.
const fn tree_connector(has_sibling: bool) -> &'static str {
    if has_sibling {
        "\u{251c}\u{2500}"
    } else {
        "\u{2514}\u{2500}"
    }
}

/// Format grandparent nodes from a generation slice.
fn fmt_grandparents(
    out: &mut String,
    gp: &[LineageAnimal],
    sire_idx: usize,
    dam_idx: usize,
    prefix: &str,
) {
    if let Some(gs) = gp.get(sire_idx) {
        let conn = tree_connector(gp.get(dam_idx).is_some());
        let _ = writeln!(out, "{prefix}{conn} Sire: {}", lineage_node_summary(gs));
    }
    if let Some(gd) = gp.get(dam_idx) {
        let _ = writeln!(
            out,
            "{prefix}\u{2514}\u{2500} Dam:  {}",
            lineage_node_summary(gd)
        );
    }
}

/// Build a comparison row for a single trait across multiple animals.
fn comparison_trait_row(t_name: &str, animals: &[AnimalDetails]) -> Vec<String> {
    let mut row = vec![t_name.to_string()];
    for a in animals {
        let (val, acc) = a.traits.get(t_name).map_or_else(
            || ("-".to_string(), "-".to_string()),
            |t| {
                let v = format!("{:.3}", t.value);
                let a = t
                    .accuracy
                    .map_or_else(|| "-".to_string(), |acc| format!("{acc}%"));
                (v, a)
            },
        );
        row.push(val);
        row.push(acc);
    }
    row
}

// ---------------------------------------------------------------------------
// Public formatters
// ---------------------------------------------------------------------------

/// Format animal details as a readable card.
pub fn fmt_details(details: &AnimalDetails) -> String {
    let mut out = String::new();

    let _ = writeln!(out, "Animal: {}", details.lpn_id);
    let _ = writeln!(
        out,
        "  Breed:        {}",
        details.breed.as_deref().unwrap_or("-")
    );
    if let Some(ref bg) = details.breed_group {
        let _ = writeln!(out, "  Breed Group:  {bg}");
    }
    let _ = writeln!(
        out,
        "  Gender:       {}",
        details.gender.as_deref().unwrap_or("-")
    );
    let _ = writeln!(
        out,
        "  DOB:          {}",
        details.date_of_birth.as_deref().unwrap_or("-")
    );
    let _ = writeln!(
        out,
        "  Status:       {}",
        details.status.as_deref().unwrap_or("-")
    );

    if let Some(ref sire) = details.sire {
        let _ = writeln!(out, "  Sire:         {sire}");
    }
    if let Some(ref dam) = details.dam {
        let _ = writeln!(out, "  Dam:          {dam}");
    }
    if let Some(ref reg) = details.registration_number {
        let _ = writeln!(out, "  Reg #:        {reg}");
    }
    if let Some(count) = details.total_progeny {
        let _ = writeln!(out, "  Progeny:      {count}");
    }
    if let Some(count) = details.flock_count {
        let _ = writeln!(out, "  Flocks:       {count}");
    }
    if let Some(ref g) = details.genotyped {
        let _ = writeln!(out, "  Genotyped:    {g}");
    }

    fmt_details_traits(&mut out, details);
    fmt_details_contact(&mut out, details);

    out
}

/// Append EBV trait table to details output.
fn fmt_details_traits(out: &mut String, details: &AnimalDetails) {
    if details.traits.is_empty() {
        return;
    }

    out.push_str("\n  EBV Traits:\n");
    let mut table = Table::new(vec!["Trait".into(), "Value".into(), "Accuracy".into()]);
    for t in sorted_traits(&details.traits) {
        let acc = t
            .accuracy
            .map_or_else(|| "-".to_string(), |a| format!("{a}%"));
        table.add_row(vec![t.name.clone(), format!("{:.3}", t.value), acc]);
    }
    for line in table.render().lines() {
        let _ = writeln!(out, "  {line}");
    }
}

/// Append contact info to details output.
fn fmt_details_contact(out: &mut String, details: &AnimalDetails) {
    let Some(ref ci) = details.contact_info else {
        return;
    };

    out.push_str("\n  Contact:\n");
    if let Some(ref v) = ci.farm_name {
        let _ = writeln!(out, "    Farm:    {v}");
    }
    if let Some(ref v) = ci.contact_name {
        let _ = writeln!(out, "    Name:    {v}");
    }
    if let Some(ref v) = ci.phone {
        let _ = writeln!(out, "    Phone:   {v}");
    }
    if let Some(ref v) = ci.email {
        let _ = writeln!(out, "    Email:   {v}");
    }
    if let Some(ref v) = ci.address {
        let _ = writeln!(out, "    Address: {v}");
    }
    let city_state_zip: Vec<&str> = [
        ci.city.as_deref(),
        ci.state.as_deref(),
        ci.zip_code.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect();
    if !city_state_zip.is_empty() {
        let _ = writeln!(out, "             {}", city_state_zip.join(", "));
    }
}

/// Format lineage as an ASCII pedigree tree.
pub fn fmt_lineage(lineage: &Lineage) -> String {
    let mut out = String::new();

    out.push_str("Pedigree:\n");

    if let Some(ref subject) = lineage.subject {
        let _ = writeln!(out, "  {}", lineage_node_summary(subject));
    }

    if let Some(ref sire) = lineage.sire {
        let _ = writeln!(
            out,
            "  \u{251c}\u{2500} Sire: {}",
            lineage_node_summary(sire)
        );
        if !lineage.generations.is_empty() {
            fmt_grandparents(&mut out, &lineage.generations[0], 0, 1, "  \u{2502}  ");
        }
    }

    if let Some(ref dam) = lineage.dam {
        let _ = writeln!(
            out,
            "  \u{2514}\u{2500} Dam:  {}",
            lineage_node_summary(dam)
        );
        if !lineage.generations.is_empty() {
            fmt_grandparents(&mut out, &lineage.generations[0], 2, 3, "     ");
        }
    }

    out
}

/// Format progeny as a table.
pub fn fmt_progeny(progeny: &Progeny, lpn_id: &str) -> String {
    let mut out = String::new();

    let _ = writeln!(out, "Progeny for {lpn_id} ({} total):", progeny.total_count);

    if progeny.animals.is_empty() {
        out.push_str("  No progeny found.\n");
        return out;
    }

    let mut all_traits =
        collect_sorted_trait_names(progeny.animals.iter().map(|a| a.traits.keys()));
    all_traits.truncate(5);

    let mut headers = vec!["LPN ID".into(), "Sex".into(), "DOB".into()];
    headers.extend(all_traits.iter().cloned());

    let mut table = Table::new(headers);

    for animal in &progeny.animals {
        let mut row = vec![
            animal.lpn_id.clone(),
            animal.sex.as_deref().unwrap_or("-").to_string(),
            animal.date_of_birth.as_deref().unwrap_or("-").to_string(),
        ];
        for t_name in &all_traits {
            let val = animal
                .traits
                .get(t_name)
                .map_or_else(|| "-".to_string(), |v| format!("{v:.3}"));
            row.push(val);
        }
        table.add_row(row);
    }

    out.push_str(&table.render());
    out.push('\n');

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    if progeny.animals.len() < progeny.total_count as usize {
        let _ = writeln!(
            out,
            "  Showing page {} ({} per page)",
            progeny.page, progeny.page_size
        );
    }

    out
}

/// Format search results as a table.
pub fn fmt_search_results(results: &SearchResults) -> String {
    let mut out = String::new();

    let _ = writeln!(
        out,
        "Search Results (page {}, {} per page, {} total):",
        results.page, results.page_size, results.total_count
    );

    if results.results.is_empty() {
        out.push_str("  No results found.\n");
        return out;
    }

    let mut table = Table::new(vec![
        "LPN ID".into(),
        "Breed".into(),
        "Gender".into(),
        "Status".into(),
        "DOB".into(),
    ]);

    for row in &results.results {
        table.add_row(extract_search_row(row));
    }

    out.push_str(&table.render());
    out
}

/// Extract a table row from a raw search result JSON value.
fn extract_search_row(row: &serde_json::Value) -> Vec<String> {
    let lpn_id = json_str(row, &["lpnId", "LpnId"]);
    let breed = json_str(row, &["breed", "Breed"]);
    let gender = json_str(row, &["gender", "Gender"]);
    let status = json_str(row, &["status", "Status"]);
    let dob = json_str(row, &["dateOfBirth", "DateOfBirth", "dob"]);
    vec![lpn_id, breed, gender, status, dob]
}

/// Try multiple keys on a JSON value, returning the first match or `"-"`.
fn json_str(val: &serde_json::Value, keys: &[&str]) -> String {
    for key in keys {
        if let Some(s) = val.get(*key).and_then(serde_json::Value::as_str) {
            return s.to_string();
        }
    }
    "-".to_string()
}

/// Format a side-by-side comparison of multiple animals.
pub fn fmt_comparison(animals: &[AnimalDetails], trait_filter: Option<&[String]>) -> String {
    let mut out = String::new();

    let _ = writeln!(out, "Comparison ({} animals):\n", animals.len());

    // Basic info table
    let mut headers = vec!["Field".to_string()];
    headers.extend(animals.iter().map(|a| a.lpn_id.clone()));

    let mut table = Table::new(headers);
    table.add_row(info_row("Breed", animals, |a| a.breed.as_deref()));
    table.add_row(info_row("Gender", animals, |a| a.gender.as_deref()));
    table.add_row(info_row("DOB", animals, |a| a.date_of_birth.as_deref()));
    table.add_row(info_row("Status", animals, |a| a.status.as_deref()));

    out.push_str(&table.render());
    out.push('\n');

    // Trait comparison
    let mut all_trait_names = collect_sorted_trait_names(animals.iter().map(|a| a.traits.keys()));

    if let Some(filter) = trait_filter {
        all_trait_names.retain(|n| filter.iter().any(|f| f.eq_ignore_ascii_case(n)));
    }

    if !all_trait_names.is_empty() {
        fmt_comparison_traits(&mut out, animals, &all_trait_names);
    }

    out
}

/// Build a comparison info row from an accessor function.
fn info_row(
    label: &str,
    animals: &[AnimalDetails],
    accessor: fn(&AnimalDetails) -> Option<&str>,
) -> Vec<String> {
    let mut row = vec![label.to_string()];
    row.extend(
        animals
            .iter()
            .map(|a| accessor(a).unwrap_or("-").to_string()),
    );
    row
}

/// Append trait comparison table to output.
fn fmt_comparison_traits(out: &mut String, animals: &[AnimalDetails], trait_names: &[String]) {
    out.push_str("EBV Traits:\n");

    let mut headers = vec!["Trait".to_string()];
    for a in animals {
        headers.push(format!("{} (val)", a.lpn_id));
        headers.push(format!("{} (acc)", a.lpn_id));
    }

    let mut table = Table::new(headers);
    for t_name in trait_names {
        table.add_row(comparison_trait_row(t_name, animals));
    }

    out.push_str(&table.render());
}

/// Format breed groups as a tree listing.
pub fn fmt_breed_groups(groups: &[BreedGroup]) -> String {
    let mut out = String::new();

    out.push_str("Breed Groups:\n");

    for (i, bg) in groups.iter().enumerate() {
        let is_last = i == groups.len() - 1;
        let connector = tree_connector(!is_last);
        let pipe = if is_last { "  " } else { "\u{2502} " };

        let _ = writeln!(out, "  {connector} {} (ID: {})", bg.name, bg.id);
        for (j, breed) in bg.breeds.iter().enumerate() {
            let b_conn = tree_connector(j < bg.breeds.len() - 1);
            let _ = writeln!(out, "  {pipe} {b_conn} {} (ID: {})", breed.name, breed.id);
        }
    }

    out
}

/// Format trait ranges as a table.
pub fn fmt_trait_ranges(data: &serde_json::Value) -> String {
    let mut out = String::new();

    out.push_str("Trait Ranges:\n");

    if let Some(arr) = data.as_array() {
        out.push_str(&fmt_trait_ranges_array(arr));
    } else if let Some(obj) = data.as_object() {
        out.push_str(&fmt_trait_ranges_object(obj));
    } else {
        out.push_str("  No trait range data available.\n");
    }

    out
}

/// Format trait ranges from an array of objects.
fn fmt_trait_ranges_array(arr: &[serde_json::Value]) -> String {
    let mut table = Table::new(vec!["Trait".into(), "Min".into(), "Max".into()]);

    for item in arr {
        let name = json_str(item, &["traitName", "TraitName", "name"]);
        let min = json_f64(item, &["minValue", "MinValue", "min"]);
        let max = json_f64(item, &["maxValue", "MaxValue", "max"]);
        table.add_row(vec![name, min, max]);
    }

    table.render()
}

/// Format trait ranges from an object (trait name -> {min, max}).
fn fmt_trait_ranges_object(obj: &serde_json::Map<String, serde_json::Value>) -> String {
    let mut table = Table::new(vec!["Trait".into(), "Min".into(), "Max".into()]);

    for (name, val) in obj {
        let min = json_f64(val, &["min", "Min"]);
        let max = json_f64(val, &["max", "Max"]);
        table.add_row(vec![name.clone(), min, max]);
    }

    table.render()
}

/// Try multiple keys for an f64 value, formatting as `{:.3}` or `"-"`.
fn json_f64(val: &serde_json::Value, keys: &[&str]) -> String {
    for key in keys {
        if let Some(v) = val.get(*key).and_then(serde_json::Value::as_f64) {
            return format!("{v:.3}");
        }
    }
    "-".to_string()
}

/// Format a full animal profile (details + lineage + progeny).
pub fn fmt_profile(profile: &AnimalProfile) -> String {
    let mut out = String::new();

    let _ = writeln!(out, "=== Profile: {} ===\n", profile.details.lpn_id);

    out.push_str(&fmt_details(&profile.details));
    out.push('\n');
    out.push_str(&fmt_lineage(&profile.lineage));
    out.push('\n');
    out.push_str(&fmt_progeny(&profile.progeny, &profile.details.lpn_id));

    out
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use nsip::{
        AnimalDetails, AnimalProfile, Breed, BreedGroup, ContactInfo, Lineage, LineageAnimal,
        Progeny, ProgenyAnimal, SearchResults, Trait,
    };

    use super::*;

    // -----------------------------------------------------------------------
    // Test data builders
    // -----------------------------------------------------------------------

    fn make_trait(name: &str, value: f64, accuracy: Option<i32>) -> Trait {
        Trait {
            name: name.to_string(),
            value,
            accuracy,
            units: None,
        }
    }

    fn make_traits(entries: &[(&str, f64, Option<i32>)]) -> HashMap<String, Trait> {
        entries
            .iter()
            .map(|(n, v, a)| ((*n).to_string(), make_trait(n, *v, *a)))
            .collect()
    }

    fn make_details_minimal() -> AnimalDetails {
        AnimalDetails {
            lpn_id: "LPN001".to_string(),
            breed: None,
            breed_group: None,
            date_of_birth: None,
            gender: None,
            status: None,
            sire: None,
            dam: None,
            registration_number: None,
            total_progeny: None,
            flock_count: None,
            genotyped: None,
            traits: HashMap::new(),
            contact_info: None,
        }
    }

    fn make_details_full() -> AnimalDetails {
        AnimalDetails {
            lpn_id: "6400012020ABC123".to_string(),
            breed: Some("Katahdin".to_string()),
            breed_group: Some("Hair Sheep".to_string()),
            date_of_birth: Some("01/15/2020".to_string()),
            gender: Some("Male".to_string()),
            status: Some("CURRENT".to_string()),
            sire: Some("SIRE001".to_string()),
            dam: Some("DAM001".to_string()),
            registration_number: Some("REG789".to_string()),
            total_progeny: Some(42),
            flock_count: Some(3),
            genotyped: Some("Yes".to_string()),
            traits: make_traits(&[
                ("BWT", 0.246, Some(80)),
                ("WWT", 2.5, Some(68)),
                ("NLB", 0.15, None),
            ]),
            contact_info: Some(ContactInfo {
                farm_name: Some("Happy Acres".to_string()),
                contact_name: Some("John Doe".to_string()),
                phone: Some("555-1234".to_string()),
                email: Some("john@farm.com".to_string()),
                address: Some("123 Farm Rd".to_string()),
                city: Some("Farmville".to_string()),
                state: Some("VA".to_string()),
                zip_code: Some("23901".to_string()),
            }),
        }
    }

    fn make_lineage_animal(
        lpn_id: &str,
        farm: Option<&str>,
        us_idx: Option<f64>,
        src_idx: Option<f64>,
        dob: Option<&str>,
        status: Option<&str>,
    ) -> LineageAnimal {
        LineageAnimal {
            lpn_id: lpn_id.to_string(),
            farm_name: farm.map(String::from),
            us_index: us_idx,
            src_index: src_idx,
            date_of_birth: dob.map(String::from),
            sex: None,
            status: status.map(String::from),
        }
    }

    // -----------------------------------------------------------------------
    // Table renderer tests
    // -----------------------------------------------------------------------

    #[test]
    fn table_empty_rows() {
        let table = Table::new(vec!["A".into(), "B".into()]);
        let rendered = table.render();
        assert!(rendered.contains("| A"));
        assert!(rendered.contains("| B"));
        // top sep, header row, middle sep, bottom sep = 4 lines
        assert_eq!(rendered.lines().count(), 4);
    }

    #[test]
    fn table_with_rows() {
        let mut table = Table::new(vec!["Name".into(), "Val".into()]);
        table.add_row(vec!["BWT".into(), "0.246".into()]);
        table.add_row(vec!["WWT".into(), "2.500".into()]);
        let rendered = table.render();
        assert!(rendered.contains("| BWT"));
        assert!(rendered.contains("| 0.246"));
        assert!(rendered.contains("| WWT"));
        // 3 separator lines + 1 header + 2 data rows = 6 lines
        assert_eq!(rendered.lines().count(), 6);
    }

    #[test]
    fn table_column_width_adapts_to_content() {
        let mut table = Table::new(vec!["X".into()]);
        table.add_row(vec!["LongValue".into()]);
        let rendered = table.render();
        // Column should be wide enough for "LongValue" (9 chars)
        assert!(rendered.contains("| LongValue |"));
    }

    #[test]
    fn table_row_shorter_than_headers() {
        let mut table = Table::new(vec!["A".into(), "B".into(), "C".into()]);
        table.add_row(vec!["1".into()]);
        let rendered = table.render();
        // Missing cells should render as empty
        assert!(rendered.contains("| 1"));
    }

    // -----------------------------------------------------------------------
    // separator_line tests
    // -----------------------------------------------------------------------

    #[test]
    fn separator_line_basic() {
        let sep = separator_line(&[3, 5]);
        assert_eq!(sep, "+-----+-------+");
    }

    #[test]
    fn separator_line_empty() {
        let sep = separator_line(&[]);
        assert_eq!(sep, "+");
    }

    #[test]
    fn separator_line_single() {
        let sep = separator_line(&[0]);
        assert_eq!(sep, "+--+");
    }

    // -----------------------------------------------------------------------
    // format_row tests
    // -----------------------------------------------------------------------

    #[test]
    fn format_row_basic() {
        let row = format_row(&["Hi".into(), "There".into()], &[5, 6]);
        assert_eq!(row, "| Hi    | There  |");
    }

    #[test]
    fn format_row_missing_cells() {
        let row = format_row(&["Only".into()], &[5, 5]);
        assert_eq!(row, "| Only  |       |");
    }

    // -----------------------------------------------------------------------
    // trait_sort_key tests
    // -----------------------------------------------------------------------

    #[test]
    fn trait_sort_key_known() {
        assert_eq!(trait_sort_key("BWT"), 0);
        assert_eq!(trait_sort_key("WWT"), 1);
        assert_eq!(trait_sort_key("PFEC"), 12);
    }

    #[test]
    fn trait_sort_key_unknown() {
        assert_eq!(trait_sort_key("UNKNOWN"), usize::MAX);
    }

    // -----------------------------------------------------------------------
    // sorted_traits tests
    // -----------------------------------------------------------------------

    #[test]
    fn sorted_traits_canonical_order() {
        let traits = make_traits(&[
            ("NLB", 0.1, None),
            ("BWT", 0.2, Some(80)),
            ("WWT", 2.5, Some(68)),
        ]);
        let sorted = sorted_traits(&traits);
        assert_eq!(sorted[0].name, "BWT");
        assert_eq!(sorted[1].name, "WWT");
        assert_eq!(sorted[2].name, "NLB");
    }

    #[test]
    fn sorted_traits_with_extras() {
        let traits = make_traits(&[("BWT", 0.2, None), ("ZZZ", 1.0, None), ("AAA", 2.0, None)]);
        let sorted = sorted_traits(&traits);
        assert_eq!(sorted[0].name, "BWT");
        // Non-canonical traits sorted alphabetically after canonical
        assert_eq!(sorted[1].name, "AAA");
        assert_eq!(sorted[2].name, "ZZZ");
    }

    #[test]
    fn sorted_traits_empty() {
        let traits: HashMap<String, Trait> = HashMap::new();
        let sorted = sorted_traits(&traits);
        assert!(sorted.is_empty());
    }

    // -----------------------------------------------------------------------
    // collect_sorted_trait_names tests
    // -----------------------------------------------------------------------

    #[test]
    fn collect_sorted_trait_names_dedup() {
        let maps = [
            HashMap::from([("BWT".to_string(), 0.1), ("WWT".to_string(), 2.0)]),
            HashMap::from([("WWT".to_string(), 3.0), ("NLB".to_string(), 0.5)]),
        ];
        let names = collect_sorted_trait_names(maps.iter().map(|m| m.keys()));
        assert_eq!(names, vec!["BWT", "WWT", "NLB"]);
    }

    #[test]
    fn collect_sorted_trait_names_empty() {
        let maps: Vec<HashMap<String, f64>> = vec![];
        let names = collect_sorted_trait_names(maps.iter().map(|m| m.keys()));
        assert!(names.is_empty());
    }

    // -----------------------------------------------------------------------
    // lineage_node_summary tests
    // -----------------------------------------------------------------------

    #[test]
    fn lineage_node_summary_full() {
        let animal = make_lineage_animal(
            "LPN001",
            Some("TestFarm"),
            Some(105.2),
            Some(98.3),
            Some("1/1/2020"),
            Some("CURRENT"),
        );
        let summary = lineage_node_summary(&animal);
        assert!(summary.contains("LPN001"));
        assert!(summary.contains("(TestFarm)"));
        assert!(summary.contains("US:105.2"));
        assert!(summary.contains("SRC$:98.3"));
        assert!(summary.contains("DOB:1/1/2020"));
        assert!(summary.contains("CURRENT"));
    }

    #[test]
    fn lineage_node_summary_minimal() {
        let animal = make_lineage_animal("LPN002", None, None, None, None, None);
        let summary = lineage_node_summary(&animal);
        assert_eq!(summary, "LPN002");
    }

    // -----------------------------------------------------------------------
    // tree_connector tests
    // -----------------------------------------------------------------------

    #[test]
    fn tree_connector_with_sibling() {
        assert_eq!(tree_connector(true), "\u{251c}\u{2500}");
    }

    #[test]
    fn tree_connector_last() {
        assert_eq!(tree_connector(false), "\u{2514}\u{2500}");
    }

    // -----------------------------------------------------------------------
    // fmt_grandparents tests
    // -----------------------------------------------------------------------

    #[test]
    fn fmt_grandparents_both_present() {
        let gp = vec![
            make_lineage_animal("GS1", Some("Farm1"), None, None, None, None),
            make_lineage_animal("GD1", Some("Farm2"), None, None, None, None),
        ];
        let mut out = String::new();
        fmt_grandparents(&mut out, &gp, 0, 1, "  ");
        assert!(out.contains("Sire:"));
        assert!(out.contains("GS1"));
        assert!(out.contains("Dam:"));
        assert!(out.contains("GD1"));
    }

    #[test]
    fn fmt_grandparents_sire_only() {
        let gp = vec![make_lineage_animal("GS1", None, None, None, None, None)];
        let mut out = String::new();
        fmt_grandparents(&mut out, &gp, 0, 1, "  ");
        assert!(out.contains("GS1"));
        // Uses last-child connector since dam is absent
        assert!(out.contains("\u{2514}\u{2500}"));
    }

    #[test]
    fn fmt_grandparents_empty() {
        let gp: Vec<LineageAnimal> = vec![];
        let mut out = String::new();
        fmt_grandparents(&mut out, &gp, 0, 1, "  ");
        assert!(out.is_empty());
    }

    // -----------------------------------------------------------------------
    // comparison_trait_row tests
    // -----------------------------------------------------------------------

    #[test]
    fn comparison_trait_row_with_data() {
        let animals = vec![
            AnimalDetails {
                traits: make_traits(&[("BWT", 0.246, Some(80))]),
                ..make_details_minimal()
            },
            AnimalDetails {
                traits: make_traits(&[("BWT", 0.512, None)]),
                ..make_details_minimal()
            },
        ];
        let row = comparison_trait_row("BWT", &animals);
        assert_eq!(row[0], "BWT");
        assert_eq!(row[1], "0.246");
        assert_eq!(row[2], "80%");
        assert_eq!(row[3], "0.512");
        assert_eq!(row[4], "-"); // no accuracy
    }

    #[test]
    fn comparison_trait_row_missing_trait() {
        let animals = vec![make_details_minimal()];
        let row = comparison_trait_row("BWT", &animals);
        assert_eq!(row[0], "BWT");
        assert_eq!(row[1], "-");
        assert_eq!(row[2], "-");
    }

    // -----------------------------------------------------------------------
    // json_str tests
    // -----------------------------------------------------------------------

    #[test]
    fn json_str_first_key_match() {
        let val = serde_json::json!({"lpnId": "ABC123"});
        assert_eq!(json_str(&val, &["lpnId", "LpnId"]), "ABC123");
    }

    #[test]
    fn json_str_second_key_match() {
        let val = serde_json::json!({"LpnId": "XYZ789"});
        assert_eq!(json_str(&val, &["lpnId", "LpnId"]), "XYZ789");
    }

    #[test]
    fn json_str_no_match() {
        let val = serde_json::json!({"other": "value"});
        assert_eq!(json_str(&val, &["lpnId", "LpnId"]), "-");
    }

    #[test]
    fn json_str_non_string_value() {
        let val = serde_json::json!({"lpnId": 12345});
        assert_eq!(json_str(&val, &["lpnId"]), "-");
    }

    // -----------------------------------------------------------------------
    // json_f64 tests
    // -----------------------------------------------------------------------

    #[test]
    fn json_f64_first_key() {
        let val = serde_json::json!({"minValue": 1.234});
        assert_eq!(json_f64(&val, &["minValue", "min"]), "1.234");
    }

    #[test]
    fn json_f64_second_key() {
        let val = serde_json::json!({"min": 5.678});
        assert_eq!(json_f64(&val, &["minValue", "min"]), "5.678");
    }

    #[test]
    fn json_f64_no_match() {
        let val = serde_json::json!({"other": 99});
        // 99 is integer but should still match as f64
        assert_eq!(json_f64(&val, &["min"]), "-");
    }

    #[test]
    fn json_f64_integer_coercion() {
        let val = serde_json::json!({"min": 5});
        assert_eq!(json_f64(&val, &["min"]), "5.000");
    }

    // -----------------------------------------------------------------------
    // extract_search_row tests
    // -----------------------------------------------------------------------

    #[test]
    fn extract_search_row_camel_case() {
        let row = serde_json::json!({
            "lpnId": "LPN1",
            "breed": "Katahdin",
            "gender": "Male",
            "status": "CURRENT",
            "dateOfBirth": "01/15/2020"
        });
        let result = extract_search_row(&row);
        assert_eq!(
            result,
            vec!["LPN1", "Katahdin", "Male", "CURRENT", "01/15/2020"]
        );
    }

    #[test]
    fn extract_search_row_pascal_case() {
        let row = serde_json::json!({
            "LpnId": "LPN2",
            "Breed": "Targhee",
            "Gender": "Female",
            "Status": "SOLD",
            "DateOfBirth": "03/20/2019"
        });
        let result = extract_search_row(&row);
        assert_eq!(
            result,
            vec!["LPN2", "Targhee", "Female", "SOLD", "03/20/2019"]
        );
    }

    #[test]
    fn extract_search_row_missing_fields() {
        let row = serde_json::json!({});
        let result = extract_search_row(&row);
        assert_eq!(result, vec!["-", "-", "-", "-", "-"]);
    }

    // -----------------------------------------------------------------------
    // info_row tests
    // -----------------------------------------------------------------------

    #[test]
    fn info_row_basic() {
        let animals = vec![
            AnimalDetails {
                breed: Some("Katahdin".to_string()),
                ..make_details_minimal()
            },
            AnimalDetails {
                breed: None,
                ..make_details_minimal()
            },
        ];
        let row = info_row("Breed", &animals, |a| a.breed.as_deref());
        assert_eq!(row, vec!["Breed", "Katahdin", "-"]);
    }

    // -----------------------------------------------------------------------
    // fmt_details tests
    // -----------------------------------------------------------------------

    #[test]
    fn fmt_details_full_output() {
        let details = make_details_full();
        let output = fmt_details(&details);

        assert!(output.contains("Animal: 6400012020ABC123"));
        assert!(output.contains("Breed:        Katahdin"));
        assert!(output.contains("Breed Group:  Hair Sheep"));
        assert!(output.contains("Gender:       Male"));
        assert!(output.contains("DOB:          01/15/2020"));
        assert!(output.contains("Status:       CURRENT"));
        assert!(output.contains("Sire:         SIRE001"));
        assert!(output.contains("Dam:          DAM001"));
        assert!(output.contains("Reg #:        REG789"));
        assert!(output.contains("Progeny:      42"));
        assert!(output.contains("Flocks:       3"));
        assert!(output.contains("Genotyped:    Yes"));
    }

    #[test]
    fn fmt_details_minimal_output() {
        let details = make_details_minimal();
        let output = fmt_details(&details);

        assert!(output.contains("Animal: LPN001"));
        assert!(output.contains("Breed:        -"));
        assert!(output.contains("Gender:       -"));
        assert!(output.contains("DOB:          -"));
        assert!(output.contains("Status:       -"));
        // Optional fields should not appear
        assert!(!output.contains("Sire:"));
        assert!(!output.contains("Dam:"));
        assert!(!output.contains("Reg #:"));
        assert!(!output.contains("Progeny:"));
        assert!(!output.contains("Flocks:"));
        assert!(!output.contains("Genotyped:"));
        assert!(!output.contains("EBV Traits:"));
        assert!(!output.contains("Contact:"));
    }

    #[test]
    fn fmt_details_traits_section() {
        let details = make_details_full();
        let output = fmt_details(&details);

        assert!(output.contains("EBV Traits:"));
        assert!(output.contains("BWT"));
        assert!(output.contains("0.246"));
        assert!(output.contains("80%"));
        assert!(output.contains("WWT"));
        assert!(output.contains("NLB"));
    }

    #[test]
    fn fmt_details_contact_section() {
        let details = make_details_full();
        let output = fmt_details(&details);

        assert!(output.contains("Contact:"));
        assert!(output.contains("Farm:    Happy Acres"));
        assert!(output.contains("Name:    John Doe"));
        assert!(output.contains("Phone:   555-1234"));
        assert!(output.contains("Email:   john@farm.com"));
        assert!(output.contains("Address: 123 Farm Rd"));
        assert!(output.contains("Farmville, VA, 23901"));
    }

    #[test]
    fn fmt_details_contact_partial() {
        let details = AnimalDetails {
            contact_info: Some(ContactInfo {
                farm_name: Some("Partial Farm".to_string()),
                contact_name: None,
                phone: None,
                email: None,
                address: None,
                city: Some("Town".to_string()),
                state: None,
                zip_code: None,
            }),
            ..make_details_minimal()
        };
        let output = fmt_details(&details);
        assert!(output.contains("Farm:    Partial Farm"));
        assert!(!output.contains("Name:"));
        assert!(output.contains("Town"));
    }

    #[test]
    fn fmt_details_contact_no_city_state_zip() {
        let details = AnimalDetails {
            contact_info: Some(ContactInfo {
                farm_name: Some("No CSZ Farm".to_string()),
                contact_name: None,
                phone: None,
                email: None,
                address: None,
                city: None,
                state: None,
                zip_code: None,
            }),
            ..make_details_minimal()
        };
        let output = fmt_details(&details);
        assert!(output.contains("Farm:    No CSZ Farm"));
        // No city/state/zip line when all are None
        let lines: Vec<&str> = output.lines().collect();
        let last_content_line = lines.iter().rev().find(|l| !l.trim().is_empty()).unwrap();
        assert!(!last_content_line.starts_with("             "));
    }

    #[test]
    fn fmt_details_trait_accuracy_dash() {
        let details = AnimalDetails {
            traits: make_traits(&[("BWT", 0.5, None)]),
            ..make_details_minimal()
        };
        let output = fmt_details(&details);
        // Accuracy should show "-" when None
        assert!(output.contains("EBV Traits:"));
        // Check the rendered table has a "-" in accuracy column
        assert!(output.contains("| - "));
    }

    // -----------------------------------------------------------------------
    // fmt_lineage tests
    // -----------------------------------------------------------------------

    #[test]
    fn fmt_lineage_full() {
        let lineage = Lineage {
            subject: Some(make_lineage_animal(
                "SUBJECT",
                Some("MyFarm"),
                Some(102.0),
                None,
                Some("1/1/2020"),
                None,
            )),
            sire: Some(make_lineage_animal(
                "SIRE1",
                Some("SireFarm"),
                None,
                None,
                None,
                None,
            )),
            dam: Some(make_lineage_animal(
                "DAM1",
                Some("DamFarm"),
                None,
                None,
                None,
                None,
            )),
            generations: vec![vec![
                make_lineage_animal("GS1", None, None, None, None, None),
                make_lineage_animal("GD1", None, None, None, None, None),
                make_lineage_animal("GS2", None, None, None, None, None),
                make_lineage_animal("GD2", None, None, None, None, None),
            ]],
        };
        let output = fmt_lineage(&lineage);

        assert!(output.starts_with("Pedigree:\n"));
        assert!(output.contains("SUBJECT"));
        assert!(output.contains("(MyFarm)"));
        assert!(output.contains("US:102.0"));
        assert!(output.contains("Sire:"));
        assert!(output.contains("SIRE1"));
        assert!(output.contains("Dam:"));
        assert!(output.contains("DAM1"));
        // Grandparents
        assert!(output.contains("GS1"));
        assert!(output.contains("GD1"));
        assert!(output.contains("GS2"));
        assert!(output.contains("GD2"));
    }

    #[test]
    fn fmt_lineage_no_generations() {
        let lineage = Lineage {
            subject: Some(make_lineage_animal("SOLO", None, None, None, None, None)),
            sire: Some(make_lineage_animal("S1", None, None, None, None, None)),
            dam: Some(make_lineage_animal("D1", None, None, None, None, None)),
            generations: vec![],
        };
        let output = fmt_lineage(&lineage);
        assert!(output.contains("SOLO"));
        assert!(output.contains("Sire:"));
        assert!(output.contains("Dam:"));
    }

    #[test]
    fn fmt_lineage_empty() {
        let lineage = Lineage {
            subject: None,
            sire: None,
            dam: None,
            generations: vec![],
        };
        let output = fmt_lineage(&lineage);
        assert_eq!(output, "Pedigree:\n");
    }

    #[test]
    fn fmt_lineage_sire_only() {
        let lineage = Lineage {
            subject: None,
            sire: Some(make_lineage_animal("S_ONLY", None, None, None, None, None)),
            dam: None,
            generations: vec![],
        };
        let output = fmt_lineage(&lineage);
        assert!(output.contains("Sire:"));
        assert!(output.contains("S_ONLY"));
        assert!(!output.contains("Dam:"));
    }

    // -----------------------------------------------------------------------
    // fmt_progeny tests
    // -----------------------------------------------------------------------

    #[test]
    fn fmt_progeny_with_animals() {
        let progeny = Progeny {
            total_count: 2,
            animals: vec![
                ProgenyAnimal {
                    lpn_id: "P1".to_string(),
                    sex: Some("Male".to_string()),
                    date_of_birth: Some("03/10/2022".to_string()),
                    traits: HashMap::from([("BWT".to_string(), 0.3)]),
                },
                ProgenyAnimal {
                    lpn_id: "P2".to_string(),
                    sex: Some("Female".to_string()),
                    date_of_birth: None,
                    traits: HashMap::from([("BWT".to_string(), 0.5), ("WWT".to_string(), 2.1)]),
                },
            ],
            page: 0,
            page_size: 10,
        };
        let output = fmt_progeny(&progeny, "PARENT1");

        assert!(output.contains("Progeny for PARENT1 (2 total):"));
        assert!(output.contains("P1"));
        assert!(output.contains("Male"));
        assert!(output.contains("03/10/2022"));
        assert!(output.contains("P2"));
        assert!(output.contains("Female"));
        // P2 has no DOB, should show "-"
        assert!(output.contains('-'));
    }

    #[test]
    fn fmt_progeny_empty() {
        let progeny = Progeny {
            total_count: 0,
            animals: vec![],
            page: 0,
            page_size: 10,
        };
        let output = fmt_progeny(&progeny, "PARENT1");
        assert!(output.contains("Progeny for PARENT1 (0 total):"));
        assert!(output.contains("No progeny found."));
    }

    #[test]
    fn fmt_progeny_pagination_message() {
        let progeny = Progeny {
            total_count: 25,
            animals: vec![ProgenyAnimal {
                lpn_id: "P1".to_string(),
                sex: None,
                date_of_birth: None,
                traits: HashMap::new(),
            }],
            page: 1,
            page_size: 10,
        };
        let output = fmt_progeny(&progeny, "PARENT");
        assert!(output.contains("Showing page 1 (10 per page)"));
    }

    #[test]
    fn fmt_progeny_no_pagination_when_all_shown() {
        let progeny = Progeny {
            total_count: 1,
            animals: vec![ProgenyAnimal {
                lpn_id: "P1".to_string(),
                sex: None,
                date_of_birth: None,
                traits: HashMap::new(),
            }],
            page: 0,
            page_size: 10,
        };
        let output = fmt_progeny(&progeny, "PARENT");
        assert!(!output.contains("Showing page"));
    }

    #[test]
    fn fmt_progeny_trait_truncation() {
        // More than 5 traits should be truncated
        let mut trait_map = HashMap::new();
        for name in &["BWT", "WWT", "PWWT", "YWT", "MWWT", "NLB", "NLW"] {
            trait_map.insert((*name).to_string(), 1.0);
        }
        let progeny = Progeny {
            total_count: 1,
            animals: vec![ProgenyAnimal {
                lpn_id: "P1".to_string(),
                sex: None,
                date_of_birth: None,
                traits: trait_map,
            }],
            page: 0,
            page_size: 10,
        };
        let output = fmt_progeny(&progeny, "PARENT");
        // Should show at most 5 trait columns
        // Line 0 = "Progeny for ...", line 1 = separator, line 2 = header row
        let header_line = output.lines().nth(2).unwrap();
        // Top 5 by TRAIT_ORDER (BWT, WWT, PWWT, YWT, MWWT) shown; NLB/NLW cut.
        assert!(header_line.contains("BWT"));
        assert!(header_line.contains("MWWT"));
        assert!(!header_line.contains("NLB"));
        assert!(!header_line.contains("NLW"));
    }

    // -----------------------------------------------------------------------
    // fmt_search_results tests
    // -----------------------------------------------------------------------

    #[test]
    fn fmt_search_results_with_data() {
        let results = SearchResults {
            total_count: 42,
            results: vec![
                serde_json::json!({"lpnId": "A1", "breed": "Katahdin", "gender": "Male", "status": "CURRENT", "dateOfBirth": "01/01/2020"}),
                serde_json::json!({"lpnId": "A2", "breed": "Targhee", "gender": "Female", "status": "SOLD", "dateOfBirth": "06/15/2019"}),
            ],
            page: 0,
            page_size: 15,
        };
        let output = fmt_search_results(&results);

        assert!(output.contains("Search Results (page 0, 15 per page, 42 total):"));
        assert!(output.contains("A1"));
        assert!(output.contains("Katahdin"));
        assert!(output.contains("A2"));
        assert!(output.contains("Targhee"));
    }

    #[test]
    fn fmt_search_results_empty() {
        let results = SearchResults {
            total_count: 0,
            results: vec![],
            page: 0,
            page_size: 15,
        };
        let output = fmt_search_results(&results);
        assert!(output.contains("No results found."));
    }

    // -----------------------------------------------------------------------
    // fmt_comparison tests
    // -----------------------------------------------------------------------

    #[test]
    fn fmt_comparison_basic() {
        let animals = vec![
            AnimalDetails {
                lpn_id: "A1".to_string(),
                breed: Some("Katahdin".to_string()),
                gender: Some("Male".to_string()),
                date_of_birth: Some("01/01/2020".to_string()),
                status: Some("CURRENT".to_string()),
                traits: make_traits(&[("BWT", 0.2, Some(80))]),
                ..make_details_minimal()
            },
            AnimalDetails {
                lpn_id: "A2".to_string(),
                breed: Some("Targhee".to_string()),
                gender: Some("Female".to_string()),
                date_of_birth: Some("03/15/2019".to_string()),
                status: Some("SOLD".to_string()),
                traits: make_traits(&[("BWT", 0.5, Some(90)), ("WWT", 3.0, Some(75))]),
                ..make_details_minimal()
            },
        ];
        let output = fmt_comparison(&animals, None);

        assert!(output.contains("Comparison (2 animals):"));
        assert!(output.contains("A1"));
        assert!(output.contains("A2"));
        assert!(output.contains("Katahdin"));
        assert!(output.contains("Targhee"));
        assert!(output.contains("EBV Traits:"));
        assert!(output.contains("BWT"));
        assert!(output.contains("WWT"));
    }

    #[test]
    fn fmt_comparison_with_trait_filter() {
        let animals = vec![
            AnimalDetails {
                lpn_id: "A1".to_string(),
                traits: make_traits(&[("BWT", 0.2, Some(80)), ("WWT", 2.5, Some(68))]),
                ..make_details_minimal()
            },
            AnimalDetails {
                lpn_id: "A2".to_string(),
                traits: make_traits(&[("BWT", 0.5, Some(90)), ("WWT", 3.0, Some(75))]),
                ..make_details_minimal()
            },
        ];
        let filter = vec!["BWT".to_string()];
        let output = fmt_comparison(&animals, Some(&filter));

        assert!(output.contains("BWT"));
        assert!(!output.contains("WWT"));
    }

    #[test]
    fn fmt_comparison_filter_case_insensitive() {
        let animals = vec![AnimalDetails {
            lpn_id: "A1".to_string(),
            traits: make_traits(&[("BWT", 0.2, None), ("WWT", 2.5, None)]),
            ..make_details_minimal()
        }];
        let filter = vec!["bwt".to_string()];
        let output = fmt_comparison(&animals, Some(&filter));
        assert!(output.contains("BWT"));
        assert!(!output.contains("WWT"));
    }

    #[test]
    fn fmt_comparison_no_traits() {
        let animals = vec![make_details_minimal(), make_details_minimal()];
        let output = fmt_comparison(&animals, None);
        assert!(!output.contains("EBV Traits:"));
    }

    // -----------------------------------------------------------------------
    // fmt_breed_groups tests
    // -----------------------------------------------------------------------

    #[test]
    fn fmt_breed_groups_multiple() {
        let groups = vec![
            BreedGroup {
                id: 1,
                name: "Hair Sheep".to_string(),
                breeds: vec![
                    Breed {
                        id: 10,
                        name: "Katahdin".to_string(),
                    },
                    Breed {
                        id: 11,
                        name: "Dorper".to_string(),
                    },
                ],
            },
            BreedGroup {
                id: 2,
                name: "Wool Sheep".to_string(),
                breeds: vec![Breed {
                    id: 20,
                    name: "Targhee".to_string(),
                }],
            },
        ];
        let output = fmt_breed_groups(&groups);

        assert!(output.starts_with("Breed Groups:\n"));
        assert!(output.contains("Hair Sheep (ID: 1)"));
        assert!(output.contains("Katahdin (ID: 10)"));
        assert!(output.contains("Dorper (ID: 11)"));
        assert!(output.contains("Wool Sheep (ID: 2)"));
        assert!(output.contains("Targhee (ID: 20)"));
    }

    #[test]
    fn fmt_breed_groups_single() {
        let groups = vec![BreedGroup {
            id: 1,
            name: "Solo Group".to_string(),
            breeds: vec![Breed {
                id: 10,
                name: "Solo Breed".to_string(),
            }],
        }];
        let output = fmt_breed_groups(&groups);
        assert!(output.contains("Solo Group (ID: 1)"));
        assert!(output.contains("Solo Breed (ID: 10)"));
        // Last group should use end connector
        assert!(output.contains("\u{2514}\u{2500}"));
    }

    #[test]
    fn fmt_breed_groups_empty() {
        let output = fmt_breed_groups(&[]);
        assert_eq!(output, "Breed Groups:\n");
    }

    // -----------------------------------------------------------------------
    // fmt_trait_ranges tests
    // -----------------------------------------------------------------------

    #[test]
    fn fmt_trait_ranges_array_format() {
        let data = serde_json::json!([
            {"traitName": "BWT", "minValue": -1.5, "maxValue": 2.0},
            {"traitName": "WWT", "minValue": -5.0, "maxValue": 10.0}
        ]);
        let output = fmt_trait_ranges(&data);

        assert!(output.starts_with("Trait Ranges:\n"));
        assert!(output.contains("BWT"));
        assert!(output.contains("-1.500"));
        assert!(output.contains("2.000"));
        assert!(output.contains("WWT"));
        assert!(output.contains("-5.000"));
        assert!(output.contains("10.000"));
    }

    #[test]
    fn fmt_trait_ranges_object_format() {
        let data = serde_json::json!({
            "BWT": {"min": -1.5, "max": 2.0},
            "WWT": {"min": -5.0, "max": 10.0}
        });
        let output = fmt_trait_ranges(&data);

        assert!(output.starts_with("Trait Ranges:\n"));
        assert!(output.contains("BWT"));
        assert!(output.contains("WWT"));
    }

    #[test]
    fn fmt_trait_ranges_neither_array_nor_object() {
        let data = serde_json::json!("just a string");
        let output = fmt_trait_ranges(&data);
        assert!(output.contains("No trait range data available."));
    }

    #[test]
    fn fmt_trait_ranges_null() {
        let data = serde_json::json!(null);
        let output = fmt_trait_ranges(&data);
        assert!(output.contains("No trait range data available."));
    }

    #[test]
    fn fmt_trait_ranges_array_missing_keys() {
        let data = serde_json::json!([
            {"name": "BWT", "min": -1.0, "max": 1.0}
        ]);
        let output = fmt_trait_ranges(&data);
        assert!(output.contains("BWT"));
        assert!(output.contains("-1.000"));
        assert!(output.contains("1.000"));
    }

    #[test]
    fn fmt_trait_ranges_array_pascal_case_keys() {
        let data = serde_json::json!([
            {"TraitName": "PEMD", "MinValue": 0.5, "MaxValue": 3.0}
        ]);
        let output = fmt_trait_ranges(&data);
        assert!(output.contains("PEMD"));
        assert!(output.contains("0.500"));
        assert!(output.contains("3.000"));
    }

    // -----------------------------------------------------------------------
    // fmt_profile tests
    // -----------------------------------------------------------------------

    #[test]
    fn fmt_profile_output() {
        let profile = AnimalProfile {
            details: make_details_full(),
            lineage: Lineage {
                subject: Some(make_lineage_animal(
                    "6400012020ABC123",
                    Some("Happy Acres"),
                    None,
                    None,
                    None,
                    None,
                )),
                sire: None,
                dam: None,
                generations: vec![],
            },
            progeny: Progeny {
                total_count: 0,
                animals: vec![],
                page: 0,
                page_size: 10,
            },
        };
        let output = fmt_profile(&profile);

        assert!(output.contains("=== Profile: 6400012020ABC123 ==="));
        assert!(output.contains("Animal: 6400012020ABC123"));
        assert!(output.contains("Pedigree:"));
        assert!(output.contains("Progeny for 6400012020ABC123"));
    }

    // -----------------------------------------------------------------------
    // fmt_comparison_traits tests (via fmt_comparison)
    // -----------------------------------------------------------------------

    #[test]
    fn fmt_comparison_traits_headers_include_val_acc() {
        let animals = vec![
            AnimalDetails {
                lpn_id: "X1".to_string(),
                traits: make_traits(&[("BWT", 0.1, Some(50))]),
                ..make_details_minimal()
            },
            AnimalDetails {
                lpn_id: "X2".to_string(),
                traits: make_traits(&[("BWT", 0.2, Some(60))]),
                ..make_details_minimal()
            },
        ];
        let output = fmt_comparison(&animals, None);
        assert!(output.contains("X1 (val)"));
        assert!(output.contains("X1 (acc)"));
        assert!(output.contains("X2 (val)"));
        assert!(output.contains("X2 (acc)"));
    }

    // -----------------------------------------------------------------------
    // Edge case: TRAIT_ORDER exhaustiveness
    // -----------------------------------------------------------------------

    #[test]
    fn trait_order_has_sixteen_entries() {
        assert_eq!(TRAIT_ORDER.len(), 16);
        assert_eq!(TRAIT_ORDER[0], "BWT");
        assert_eq!(TRAIT_ORDER[15], "YSL");
    }
}
