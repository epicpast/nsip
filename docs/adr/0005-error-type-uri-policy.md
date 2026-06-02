---
title: "Error type URI Policy"
description: "Stable, unversioned repo-docs URIs for RFC 9457 error problem types"
type: adr
category: architecture
tags:
  - error-handling
  - rfc9457
  - versioning
status: accepted
created: "2026-06-01"
updated: "2026-06-01"
author: nsip maintainers
project: nsip
technologies:
  - rust
audience:
  - developers
related:
  - 0004-dual-consumer-error-envelope.md
---

# Error `type` URI Policy

## Context

RFC 9457 requires each problem `type` to be a stable URI and obliges the author
to commit to a versioning policy so agents can rely on the `type` as a dispatch
key. We needed to pick the URI shape and the stability contract.

## Decision

Error `type` URIs are repository documentation URLs of the form:

```
https://github.com/zircote/nsip/blob/main/docs/reference/errors/<domain>/<slug>.md
```

- **Stable forever, no version path segment.** A `type` URI's meaning never
  changes in place. We do not embed `/v1/`; instead the URI is permanent and
  any semantic evolution is tracked in the documentation and `CHANGELOG.md`.
- **Additive only.** New problem types may be introduced (a non-breaking
  change); existing slugs are never repurposed or removed.
- **`docs_url` equals `type`.** Each URI resolves to a real catalog page under
  [`docs/reference/errors/`](../reference/errors/) documenting the status, exit
  code, applicability marker, and recovery guidance.
- **One slug per error variant.** The `Api` variant uses a single `api/error`
  slug and conveys the specific HTTP status in the envelope's `status` member
  rather than splitting into per-status slugs.

## Consequences

- Agents can treat the `type` URI as a durable dispatch key and cache
  per-type handling (including the applicability marker from the catalog).
- Adding a new error requires adding a catalog page; CI doc-link checks and the
  envelope unit tests (which assert the `type` ends in `.md`) guard against
  drift.
- Choosing repo-docs URIs avoids a separate hosting/versioning commitment while
  keeping the documentation co-located with the code and versioned by git.
