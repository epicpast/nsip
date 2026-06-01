# `api/error` — Upstream API returned an error

- **type**: `https://github.com/zircote/nsip/blob/main/docs/reference/errors/api/error.md`
- **status**: the upstream HTTP status (e.g. 400, 429, 500)
- **exit_code**: `1` for a 4xx client error; `75` (`EX_TEMPFAIL`) for `429` and `5xx`
- **class**: caller (4xx) or transient (429 / 5xx)
- **`suggested_fix` applicability**: `unspecified` for 429/5xx; none for other 4xx
- **`retry_after`**: present for `429` (and `5xx` when the upstream sends `Retry-After`)

## When it occurs

The NSIP API returned a non-success, non-404 HTTP status. The `status` member
carries the real upstream code so a consumer can branch:

- **4xx (not 429)** — a client-side problem (bad request the API rejected).
  Terminal; exit `1`.
- **429** — rate limited. Transient; exit `75`; `retry_after` carries the
  delay parsed from the `Retry-After` header.
- **5xx** — upstream server error. Transient; exit `75`. The client already
  retried retryable 5xx (`500/502/503/504`) with backoff before surfacing this.

## Recovery

For 429/5xx, wait `retry_after` seconds (or use exponential backoff) and retry.
For other 4xx, inspect `detail` (the upstream response body) — the request
itself needs to change.

## Example

```json
{
  "type": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/api/error.md",
  "title": "Upstream API returned an error",
  "status": 429,
  "detail": "API error (HTTP 429): rate limited",
  "instance": "urn:nsip:search:b8a9c0f3-f8fc-44a0-8c9c-f9dc78b1b7c2",
  "exit_code": 75,
  "suggested_fix": "wait for the retry_after interval before retrying",
  "retry_after": 30,
  "docs_url": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/api/error.md"
}
```
