---
diataxis_type: how-to
---
# How to Enable and Use Telemetry

> **Problem:** You want structured, machine-readable logs from the NSIP MCP server with W3C trace context so you can correlate log lines across a single request/response cycle.

**Prerequisites:**
- Rust toolchain at MSRV 1.92 or later
- `cargo install` or a local clone of the `nsip` repository

---

## Install nsip with Telemetry Support

Telemetry is behind a compile-time feature flag. It is not included in the default build.

### From crates.io

```bash
cargo install nsip --features telemetry
```

### From a Local Clone

```bash
cargo build --release --features telemetry
```

The `telemetry` feature pulls in `opentelemetry`, `opentelemetry_sdk`, `opentelemetry-otlp`, and `tracing-opentelemetry` as additional dependencies.

---

## Verify Telemetry Is Enabled

Start the MCP server and look at the log output on stderr:

```bash
nsip mcp 2>mcp.log &
head -1 mcp.log
```

If telemetry is enabled, each log line is a JSON object containing `trace_id` and `span_id` fields:

```json
{"timestamp":"2026-03-08T12:00:00.000Z","level":"INFO","trace_id":"4bf92f3577b34da6a3ce929d0e0e4736","span_id":"00f067aa0ba902b7","fields":{"message":"MCP server started"},"target":"nsip::mcp","span":{"name":"serve"},"spans":[{"name":"serve"}]}
```

If telemetry is **not** enabled, the output is plain-text tracing directed to stderr with no JSON structure.

---

## Read the JSON Log Format

Every log line is a single JSON object with the following fields:

| Field | Type | Description |
|---|---|---|
| `timestamp` | string | ISO 8601 timestamp from the system clock |
| `level` | string | Tracing level: `TRACE`, `DEBUG`, `INFO`, `WARN`, or `ERROR` |
| `trace_id` | string or null | W3C trace ID (32 hex characters), or `null` when no trace context exists |
| `span_id` | string or null | W3C span ID (16 hex characters), or `null` when no span is active |
| `fields` | object | Event key-value pairs (e.g., `{"message": "request received", "tool": "search_animals"}`) |
| `target` | string | The Rust module path that emitted the event (e.g., `nsip::mcp::server`) |
| `span` | object | The innermost (current) span, with at minimum a `name` key |
| `spans` | array | Full scope from root span to leaf span, each with a `name` key and any recorded fields |

### Parse a Log Line with jq

```bash
# Pretty-print the first log line
head -1 mcp.log | jq .

# Extract only trace IDs
cat mcp.log | jq -r '.trace_id // empty'
```

---

## Correlate Logs with Traces

All log lines emitted during a single MCP request/response cycle share the same `trace_id`. Use this to group related events.

### Step 1: Identify the Trace ID

Pick the `trace_id` from any log line associated with the request you want to investigate:

```bash
cat mcp.log | jq -r 'select(.fields.message == "request received") | .trace_id'
```

### Step 2: Filter All Lines for That Trace

```bash
TRACE="4bf92f3577b34da6a3ce929d0e0e4736"
cat mcp.log | jq "select(.trace_id == \"$TRACE\")"
```

This shows every event from the request being accepted through to the response being sent, in chronological order.

### Step 3: Inspect the Span Tree

The `spans` array on each line shows the full scope. The first element is the root span; the last is the leaf. Use this to understand which phase of processing emitted the event:

```bash
cat mcp.log | jq "select(.trace_id == \"$TRACE\") | {level, message: .fields.message, scope: [.spans[].name]}"
```

Example output:

```json
{"level":"INFO","message":"request received","scope":["serve","handle_request"]}
{"level":"DEBUG","message":"searching animals","scope":["serve","handle_request","search_animals"]}
{"level":"INFO","message":"response sent","scope":["serve","handle_request"]}
```

---

## Run without Telemetry

The default build includes zero OpenTelemetry dependencies. Plain-text tracing is emitted to stderr:

```bash
cargo install nsip
# or
cargo build --release
```

Log output uses the standard `tracing_subscriber` text format:

```
2026-03-08T12:00:00.000Z  INFO nsip::mcp::server: MCP server started
```

No `trace_id` or `span_id` fields are present. No OpenTelemetry crates are compiled.

---

## What's Next

The telemetry feature currently provides trace context in structured logs. No OTLP exporter is configured by default. To send spans to Jaeger, Grafana Tempo, or another collector, extend the tracer provider in `crates/mcp/telemetry.rs` with an OTLP exporter pipeline. The JSON log format will not change.

---

## See Also

- [MCP Server Configuration Reference](../reference/MCP-SERVER-CONFIGURATION.md) -- server startup options and environment variables
- [Telemetry Concepts](../explanation/TELEMETRY.md) -- background on distributed tracing and why trace context matters
