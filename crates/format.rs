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
    "BWT", "WWT", "PWWT", "YWT", "FAT", "EMD", "NLB", "NWT", "PWT", "DAG", "WGR", "WEC", "FEC",
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
