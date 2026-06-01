# `api/timeout` — Request to the NSIP API timed out

- **type**: `https://github.com/zircote/nsip/blob/main/docs/reference/errors/api/timeout.md`
- **status**: 504 · **exit_code**: 75 (`EX_TEMPFAIL`) · **class**: transient
- **`suggested_fix` applicability**: `unspecified`
- **`retry_after`**: present when a delay is known

## When it occurs

The request exceeded the configured client timeout (default 30s). The client
already retried with exponential backoff before surfacing this.

## Recovery

Retry; the failure is transient. Increase the client timeout
(`NsipClient::builder().timeout_secs(...)`) if timeouts persist. The originating
`reqwest` error is preserved on the cause chain (`Error::source()`).

## Example

```json
{
  "type": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/api/timeout.md",
  "title": "Request to the NSIP API timed out",
  "status": 504,
  "detail": "request timed out: request timed out after 30s: ...",
  "instance": "urn:nsip:search:2b3c4d5e-6f70-8192-a3b4-c5d6e7f80910",
  "exit_code": 75,
  "suggested_fix": "retry the request; increase the client timeout if this persists",
  "docs_url": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/api/timeout.md"
}
```
