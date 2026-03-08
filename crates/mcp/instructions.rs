//! Dynamic MCP server instruction text generation.
//!
//! Builds the `instructions` field of `ServerInfo` so MCP clients receive
//! comprehensive documentation of all tools, prompts, resources, and
//! common parameters exposed by the NSIP server.

/// Build server instructions covering all tools, prompts, and resources.
///
/// Returns a single `String` suitable for the `ServerInfo.instructions` field.
#[must_use]
pub(crate) fn build_instructions() -> String {
    let mut out = String::with_capacity(4096);
    append_header(&mut out);
    append_search_tools(&mut out);
    append_analytics_tools(&mut out);
    append_flock_tools(&mut out);
    append_breed_tools(&mut out);
    append_resource_guide(&mut out);
    append_prompt_guide(&mut out);
    append_common_parameters(&mut out);
    out
}

fn append_header(out: &mut String) {
    out.push_str(
        "NSIP Livestock Intelligence Server \u{2014} sheep genetic evaluation, \
         breeding decision support, and flock analytics.\n\n\
         Covers the full nsipsearch.nsip.org/api surface with 13 tools, \
         guided prompts, and resource templates for sheep breeders.\n",
    );
}

fn append_search_tools(out: &mut String) {
    out.push_str(
        "\n## Search & Retrieval Tools\n\n\
         - **search**: Find animals by breed, status, gender, birth date range, flock, \
         and keyword. Supports pagination via page/page_size.\n\
         - **details**: Retrieve full EBV profile and metadata for a single animal by \
         LPN ID.\n\
         - **lineage**: Get sire/dam pedigree tree for an animal (parents, grandparents).\n\
         - **progeny**: Paginated list of offspring for a specific animal.\n\
         - **profile**: Combined view \u{2014} details + lineage + progeny in one call.\n",
    );
}

fn append_analytics_tools(out: &mut String) {
    out.push_str(
        "\n## Analytics Tools\n\n\
         - **compare**: Side-by-side EBV comparison of 2\u{2013}5 animals with trait-level \
         differences and optional trait filtering.\n\
         - **rank**: Rank animals within a breed by a target trait (e.g. YWT, YEMD, \
         NLB). Returns top-N with percentile positions.\n\
         - **inbreeding_check**: Compute inbreeding coefficient (COI) between two \
         animals by comparing pedigree overlap.\n\
         - **mating_recommendations**: Given a sire, find optimal mates from available \
         ewes based on trait complementarity and inbreeding avoidance.\n",
    );
}

fn append_flock_tools(out: &mut String) {
    out.push_str(
        "\n## Flock Tools\n\n\
         - **flock_summary**: Aggregate statistics for a flock \u{2014} animal counts by \
         gender/status, mean EBVs, and trait distributions.\n\
         - **database_status**: NSIP database last-updated date and available animal \
         statuses.\n",
    );
}

fn append_breed_tools(out: &mut String) {
    out.push_str(
        "\n## Breed Tools\n\n\
         - **breed_groups**: List all breed groups and individual breeds in the NSIP \
         database with their IDs.\n\
         - **trait_ranges**: Breed-level EBV percentile ranges (min/max/mean) for \
         benchmarking individual animals.\n",
    );
}

fn append_resource_guide(out: &mut String) {
    out.push_str(
        "\n## Resources (nsip:// URI scheme)\n\n\
         ### Static Resources\n\
         - `nsip://glossary` \u{2014} EBV trait glossary with descriptions and units\n\
         - `nsip://breeds` \u{2014} Complete breed listing\n\
         - `nsip://guide/selection` \u{2014} Selection strategy guide for breeding programs\n\
         - `nsip://guide/inbreeding` \u{2014} Inbreeding management reference\n\
         - `nsip://status` \u{2014} Database status and last-updated timestamp\n\n\
         ### Resource Templates\n\
         - `nsip://animal/{lpn_id}` \u{2014} Full animal profile by LPN ID\n\
         - `nsip://animal/{lpn_id}/pedigree` \u{2014} Pedigree/lineage tree\n\
         - `nsip://animal/{lpn_id}/progeny` \u{2014} Offspring listing\n\
         - `nsip://breed/{breed_id}/ranges` \u{2014} Breed EBV percentile ranges\n",
    );
}

fn append_prompt_guide(out: &mut String) {
    out.push_str(
        "\n## Guided Prompts\n\n\
         - **evaluate-ram**: Evaluate a ram's breeding value \u{2014} fetches EBVs, breed \
         ranges, and constructs a comprehensive assessment.\n\
         - **evaluate-ewe**: Evaluate a ewe's breeding value with emphasis on maternal \
         traits (NLB, NWT, PWT).\n\
         - **compare-breeding-stock**: Compare multiple animals side-by-side with trait \
         analysis and breeding recommendations.\n\
         - **plan-mating**: Plan a specific mating \u{2014} COI check, trait complementarity, \
         and offspring prediction.\n\
         - **flock-improvement**: Analyze a breed or flock for trait gaps and improvement \
         opportunities.\n\
         - **select-replacement**: Find top replacement candidates within a breed by \
         gender and target trait.\n\
         - **interpret-ebvs**: Explain an animal's EBV profile in plain language with \
         breed-relative context.\n",
    );
}

fn append_common_parameters(out: &mut String) {
    out.push_str(
        "\n## Common Parameters\n\n\
         - **page / page_size**: Pagination cursors for search and progeny results. \
         First page is 1; default page_size varies by tool.\n\
         - **output format**: All tool results return JSON. Use the details tool for \
         full EBV breakdowns or profile for a combined view.\n",
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_is_nonempty() {
        let instructions = build_instructions();
        assert!(!instructions.is_empty());
    }

    #[test]
    fn contains_section_headers() {
        let instructions = build_instructions();
        let headers = [
            "NSIP Livestock Intelligence Server",
            "## Search & Retrieval Tools",
            "## Analytics Tools",
            "## Flock Tools",
            "## Breed Tools",
            "## Resources",
            "## Guided Prompts",
            "## Common Parameters",
        ];
        for header in headers {
            assert!(
                instructions.contains(header),
                "Missing section header: {header}"
            );
        }
    }

    #[test]
    fn contains_all_thirteen_tool_names() {
        let instructions = build_instructions();
        let tools = [
            "search",
            "details",
            "lineage",
            "progeny",
            "profile",
            "compare",
            "rank",
            "inbreeding_check",
            "mating_recommendations",
            "flock_summary",
            "database_status",
            "breed_groups",
            "trait_ranges",
        ];
        for tool in tools {
            assert!(instructions.contains(tool), "Missing tool name: {tool}");
        }
    }

    #[test]
    fn contains_all_seven_prompt_names() {
        let instructions = build_instructions();
        let prompts = [
            "evaluate-ram",
            "evaluate-ewe",
            "compare-breeding-stock",
            "plan-mating",
            "flock-improvement",
            "select-replacement",
            "interpret-ebvs",
        ];
        for prompt in prompts {
            assert!(
                instructions.contains(prompt),
                "Missing prompt name: {prompt}"
            );
        }
    }

    #[test]
    fn contains_nsip_uri_references() {
        let instructions = build_instructions();
        assert!(
            instructions.contains("nsip://"),
            "Missing nsip:// URI references"
        );
        let uris = [
            "nsip://glossary",
            "nsip://breeds",
            "nsip://guide/selection",
            "nsip://guide/inbreeding",
            "nsip://status",
            "nsip://animal/{lpn_id}",
            "nsip://animal/{lpn_id}/pedigree",
            "nsip://animal/{lpn_id}/progeny",
            "nsip://breed/{breed_id}/ranges",
        ];
        for uri in uris {
            assert!(instructions.contains(uri), "Missing URI reference: {uri}");
        }
    }

    #[test]
    fn uses_preallocated_buffer() {
        // Verify that the output fits within a reasonable capacity range
        // to confirm we're not wildly over- or under-allocating.
        let instructions = build_instructions();
        assert!(
            instructions.len() < 8192,
            "Instructions unexpectedly large: {} bytes",
            instructions.len()
        );
        assert!(
            instructions.len() > 512,
            "Instructions unexpectedly small: {} bytes",
            instructions.len()
        );
    }
}
