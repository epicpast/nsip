# `api/upstream-parse` — Could not parse the NSIP API response

- **type**: `https://github.com/zircote/nsip/blob/main/docs/reference/errors/api/upstream-parse.md`
- **status**: 502 · **exit_code**: 3 (environment) · **class**: environment
- **`suggested_fix` applicability**: none (no deterministic local fix)
- **`retry_after`**: never

## When it occurs

The API returned a 200 response whose body could not be deserialized into the
expected model — malformed JSON, an unexpected shape, or a body missing the
identity field for a single-animal lookup. Treated as a bad-gateway-class
(`502`) environment error: the upstream sent something the client cannot use.

## Recovery

No deterministic local fix. The originating `serde_json` error is preserved on
the cause chain (`Error::source()`). If reproducible, the upstream response
shape has likely changed and the model parsing needs updating — file an issue.

## Example

```json
{
  "type": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/api/upstream-parse.md",
  "title": "Could not parse the NSIP API response",
  "status": 502,
  "detail": "parse error: failed to parse response: ...",
  "instance": "urn:nsip:date-updated:4d5e6f70-8192-a3b4-c5d6-e7f809101112",
  "exit_code": 3,
  "docs_url": "https://github.com/zircote/nsip/blob/main/docs/reference/errors/api/upstream-parse.md"
}
```
