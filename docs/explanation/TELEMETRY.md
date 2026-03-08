# Telemetry and Distributed Tracing

> Why the NSIP MCP server includes an optional telemetry system, how distributed tracing works, and the design decisions behind the implementation.

---

## What is distributed tracing?

When a user issues a request to the NSIP MCP server, that single request often fans out into multiple operations: validating parameters, querying the NSIP backend API for breed groups, fetching animal details, and formatting results. If something goes wrong -- or if a request is unexpectedly slow -- understanding which operation caused the problem requires more than flat log lines.

Distributed tracing solves this by assigning a unique **trace ID** to each incoming request and a unique **span ID** to each operation within it. A span represents a unit of work (an HTTP call, a database query, a function invocation). Spans nest: a parent span for the overall request contains child spans for each sub-operation. Together, the trace ID and the span hierarchy let you reconstruct the full lifecycle of any request.

In the NSIP server, this means you can correlate a `search_animals` MCP tool invocation with the underlying HTTP calls to the NSIP Search API, even when multiple requests are being processed concurrently.

---

## W3C Trace Context

The NSIP server uses the [W3C Trace Context](https://www.w3.org/TR/trace-context/) standard for trace propagation. This is the industry-standard format adopted by OpenTelemetry, and it defines two key identifiers:

- **`trace_id`** -- a 32-character hexadecimal string (128 bits) that uniquely identifies the entire request from start to finish. All spans belonging to the same request share the same trace ID.
- **`span_id`** -- a 16-character hexadecimal string (64 bits) that uniquely identifies a single operation within the trace.

By using W3C format, the NSIP server produces trace context that is compatible with any observability backend that supports the standard: Jaeger, Datadog, Grafana Tempo, Honeycomb, and others.

The `SdkTracerProvider` in the telemetry module uses OpenTelemetry's default random ID generator, so every span automatically receives properly formatted W3C identifiers without additional configuration.

---

## Why telemetry is feature-gated

OpenTelemetry support requires four additional crate dependencies:

| Crate | Purpose |
|-------|---------|
| `opentelemetry` | Core trait definitions and trace API |
| `opentelemetry_sdk` | SDK implementation with `SdkTracerProvider` |
| `opentelemetry-otlp` | OTLP exporter protocol support |
| `tracing-opentelemetry` | Bridge between Rust's `tracing` ecosystem and OpenTelemetry |

These dependencies add meaningful compile time and increase the final binary size. For CLI users running `nsip search` from a terminal, observability infrastructure is unnecessary overhead.

The `telemetry` Cargo feature keeps these costs opt-in. The default build compiles without any OpenTelemetry code. Production MCP server deployments that need structured trace context can enable it with `--features telemetry`. This keeps the common path fast and small while making the observability path available without forking or patching.

---

## JSON log format

When telemetry is enabled, the server replaces the default log formatter with `OtelJsonFormat`, a custom implementation of `tracing_subscriber`'s `FormatEvent` trait. Every log line becomes a self-contained JSON object with a consistent schema:

```json
{
  "timestamp": "2026-03-08T14:30:00.123456Z",
  "level": "INFO",
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
  "span_id": "00f067aa0ba902b7",
  "fields": {
    "message": "search completed",
    "result_count": 42
  },
  "target": "nsip::mcp::tools",
  "span": {
    "name": "search_animals",
    "breed_id": "640"
  },
  "spans": [
    { "name": "mcp_request", "request_id": "abc-123" },
    { "name": "search_animals", "breed_id": "640" }
  ]
}
```

The key fields are:

- **`timestamp`** -- when the event occurred, formatted by `SystemTime`.
- **`level`** -- the tracing level (`TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`).
- **`trace_id`** and **`span_id`** -- W3C identifiers extracted from the OpenTelemetry span extensions. When an event occurs outside an instrumented span (such as during startup), these are `null` rather than absent. This preserves schema consistency so log parsers do not need to handle missing fields.
- **`fields`** -- the event's own key-value data, recorded by a `JsonMapVisitor` that handles all Rust primitive types (`str`, `i64`, `u64`, `f64`, `bool`, `i128`, `u128`, and `Debug` fallback).
- **`target`** -- the Rust module path where the event originated.
- **`span`** -- the innermost (current) span, with its name and any recorded fields.
- **`spans`** -- the full span ancestry from root to leaf, allowing you to see the complete call chain for any log line.

---

## Architecture

The telemetry module (`crates/mcp/telemetry.rs`) has three responsibilities:

### 1. Tracer provider initialization

`init_tracer_provider()` creates an `SdkTracerProvider` with default configuration. The provider's random ID generator assigns W3C-format trace and span IDs to every span. No exporter is configured at this stage -- the provider exists solely to generate trace context.

### 2. OpenTelemetry-tracing bridge

`otel_layer()` creates a `tracing_opentelemetry::OpenTelemetryLayer` from the provider. This layer is added to the `tracing_subscriber::Registry`, which means every `#[tracing::instrument]` span in application code is automatically backed by an OpenTelemetry span with a real trace ID. Application code does not need to interact with OpenTelemetry APIs directly.

### 3. Custom JSON formatter

`OtelJsonFormat` implements `FormatEvent` and does the work of extracting trace context from span extensions. For each log event, it:

1. Walks the span scope from root to leaf looking for `OtelData` in span extensions.
2. Extracts `trace_id` and `span_id` from the first span that has them.
3. Serializes the event, its fields, and the span chain into a single JSON line.

This design means the formatter is the only component that needs to understand the boundary between `tracing` and OpenTelemetry. Everything else in the application uses standard `tracing` macros.

The module also provides `FmtWriteAdapter`, a thin wrapper that bridges `io::Write` to the writer interface expected by `tracing_subscriber::fmt::Layer`.

---

## Future extensibility

The current implementation provides trace context in logs without sending spans to an external backend. This is a deliberate starting point: it gives structured, correlated logs with zero infrastructure requirements.

Adding a full tracing pipeline requires only one change: configuring an OTLP exporter on the `SdkTracerProvider`. For example, pointing spans to a Jaeger instance or Grafana Tempo would involve adding an exporter in `init_tracer_provider()` without modifying `OtelJsonFormat` or any application-level tracing instrumentation.

This separation of concerns -- provider generates context, formatter surfaces it in logs, exporter ships it to backends -- means each piece can evolve independently. A future version could add:

- **OTLP export** to send spans to Jaeger, Datadog, Tempo, or any OTLP-compatible backend.
- **Sampling** to reduce volume in high-throughput deployments.
- **Baggage propagation** to carry user-defined context (such as flock IDs) across span boundaries.
- **Metrics** via the OpenTelemetry metrics API, reusing the same provider infrastructure.

None of these additions would require changes to existing application code or the log format.

---

## Further reading

- [How to enable telemetry](../how-to/TELEMETRY.md) -- step-by-step setup instructions
- [MCP server configuration reference](../reference/MCP-SERVER-CONFIGURATION.md) -- all configuration options
- [W3C Trace Context specification](https://www.w3.org/TR/trace-context/) -- the standard behind trace and span IDs
- [OpenTelemetry Rust documentation](https://docs.rs/opentelemetry/latest/opentelemetry/) -- upstream crate docs
