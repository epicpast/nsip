---
title: "Adopt Diataxis Documentation Framework"
description: "Organize user documentation using the Diataxis framework"
type: adr
category: documentation
tags:
  - documentation
  - diataxis
  - structure
status: accepted
created: "2026-02-16"
updated: "2026-02-16"
author: nsip maintainers
project: nsip
technologies:
  - markdown
audience:
  - developers
related:
  - 0002-documentation-directory-structure.md
---

# Adopt Diataxis Documentation Framework

## Context

The repository had comprehensive technical reference documentation (11,562 lines across 39 markdown files) but lacked structured learning paths for new users. Existing documentation focused on:

- Reference material (API docs, MCP server reference)
- Operational runbooks (releasing, CI troubleshooting)
- Template guides (for repository setup)

However, there were no:
- **Tutorials** - Learning-oriented, hands-on lessons for newcomers
- **How-To Guides** - Problem-oriented solutions for specific tasks
- **Explanations** - Understanding-oriented conceptual documentation

This created a documentation gap where users had to jump directly from the README examples to comprehensive API reference, with no intermediate learning materials.

---

## Decision

Adopt the [Diátaxis framework](https://diataxis.fr/) for organizing all user-facing documentation.

### Framework Structure

Diátaxis organizes documentation into four quadrants:

| Type | Orientation | Focus | Example |
|------|------------|-------|---------|
| **Tutorials** | Learning | Practical steps | "Getting Started with NSIP" |
| **How-To Guides** | Problem-solving | Specific goals | "How to Compare Animals" |
| **Explanation** | Understanding | Concepts | "Understanding EBVs" |
| **Reference** | Information | Technical facts | "Error Handling Reference" |

### Directory Structure

````
docs/
├── README.md              # Documentation index with Diátaxis organization
├── tutorials/             # Learning-oriented guides
├── how-to/                # Problem-oriented guides
├── explanation/           # Understanding-oriented discussions
├── reference/             # Information-oriented technical docs
├── runbooks/              # Operational procedures (existing)
├── template/              # Template adoption guides (existing)
└── workflows/             # CI/CD documentation (existing)
````

### Initial Documentation Set

**Tutorials:**
- `tutorials/GETTING-STARTED.md` - 15-minute introduction to NSIP API

**How-To Guides:**
- `how-to/CONFIGURE-CLIENT.md` - Customize timeout and retry settings
- `how-to/COMPARE-ANIMALS.md` - Side-by-side genetic trait comparisons

**Explanation:**
- `explanation/EBV-EXPLAINED.md` - What EBVs are and how to use them

**Reference:**
- `reference/ERROR-HANDLING.md` - Complete error type reference
- `MCP.md` - MCP server API reference (existing, moved to reference category)

---

## Consequences

### Positive

✅ **Clear learning path** - New users can start with tutorials and progress naturally  
✅ **Findability** - Users know where to look for different types of information  
✅ **Progressive disclosure** - Information complexity increases with user expertise  
✅ **Reduced cognitive load** - Each document has a single, clear purpose  
✅ **Maintainability** - Clear categorization makes updates easier  
✅ **Industry standard** - Diátaxis is used by Django, NumPy, and other major projects  

### Negative

⚠️ **Migration effort** - Some existing docs may need recategorization  
⚠️ **Link updates** - Internal references need updating to new paths  
⚠️ **Duplication risk** - Clear guidelines needed to avoid content overlap  

### Mitigations

- Existing documentation remains in place (runbooks, template guides, workflows)
- New categories augment rather than replace current structure
- Documentation index (`docs/README.md`) provides unified navigation
- Cross-references between quadrants help users discover related content

---

## Alternatives Considered

### 1. Keep Current Ad-Hoc Structure

**Pros:** No changes needed  
**Cons:** Continues documentation gaps, no clear user journey

### 2. Use README-Only Approach

**Pros:** Simple, single file  
**Cons:** Doesn't scale, hard to navigate, poor SEO

### 3. Auto-Generated API Docs Only

**Pros:** Always in sync with code  
**Cons:** No conceptual explanations, steep learning curve

---

## References

- [Diátaxis Framework](https://diataxis.fr/)
- [Django Documentation](https://docs.djangoproject.com/) - Diátaxis example
- [Write the Docs: Documentation Systems](https://www.writethedocs.org/guide/docs-as-code/)

---

## Implementation Notes

The `update-docs` workflow detected the documentation framework gap and created initial documentation in each quadrant. Future documentation should follow this structure:

**When writing tutorials:**
- Use step-by-step instructions
- Build something tangible
- Assume minimal prior knowledge
- Provide complete working examples

**When writing how-to guides:**
- Start with a clear problem statement
- Provide solution steps
- Focus on one specific task
- Include time estimates

**When writing explanations:**
- Clarify concepts and theory
- Connect ideas
- Provide context and background
- Use diagrams where helpful

**When writing reference:**
- Be accurate and complete
- Use consistent structure
- Include all parameters/options
- Provide examples
