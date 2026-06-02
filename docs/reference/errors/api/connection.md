# `api/connection` — Could not connect to the NSIP API

- **type**: `https://github.com/zircote/nsip/blob/main/docs/reference/errors/api/connection.md`
- **status**: 503 · **exit_code**: 75 (`EX_TEMPFAIL`) · **class**: transient
- **`suggested_fix` applicability**: `unspecified`
- **`retry_after`**: present when a delay is known

## When it occurs

The HTTP request failed to connect (DNS failure, refused connection, network
unreachable), or the retry budget was exhausted. The MCP server transport also
surfaces start/bind failures through this type.

## Recovery

Check network connectivity to `nsipsearch.nsip.org` and retry; the failure is
transient. The originating `reqwest` error is preserved on the cause chain
(`Error::source()`).

## Example

```json
{
  "type": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/api/connection.md",
  "title": "Could not connect to the NSIP API",
  "status": 503,
  "detail": "connection error: failed to connect to API: ...",
  "instance": "urn:nsip:search:3c4d5e6f-7081-92a3-b4c5-d6e7f8091011",
  "exit_code": 75,
  "suggested_fix": "check network connectivity to nsipsearch.nsip.org and retry",
  "docs_url": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/api/connection.md"
}
```
