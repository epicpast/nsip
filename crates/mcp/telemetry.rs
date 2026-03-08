//! `OpenTelemetry`-integrated tracing setup (feature-gated behind `telemetry`).
//!
//! Provides a custom JSON event formatter ([`OtelJsonFormat`]) that extracts
//! W3C trace context (`trace_id`, `span_id`) from the `OpenTelemetry` span
//! extensions and writes them into every JSON log line. Combined with
//! [`init_tracer_provider`], this gives distributed-tracing-compatible
//! output without requiring an external collector.

use std::fmt;
use std::io;

use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing::{Event, Subscriber};
use tracing_opentelemetry::OtelData;
use tracing_subscriber::fmt::format::{FormatEvent, FormatFields, Writer};
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::fmt::{FmtContext, FormattedFields};
use tracing_subscriber::registry::LookupSpan;

// ---------------------------------------------------------------------------
// Tracer provider
// ---------------------------------------------------------------------------

/// Initialise an `OpenTelemetry` [`SdkTracerProvider`].
///
/// The provider uses the default random ID generator so every span receives a
/// proper W3C-format `trace_id` (32 hex chars) and `span_id` (16 hex chars).
/// No exporter is configured — the provider exists solely to assign trace
/// context that [`OtelJsonFormat`] can surface in JSON logs. An OTLP
/// exporter can be layered on later without changing the log format.
#[must_use]
pub fn init_tracer_provider() -> SdkTracerProvider {
    SdkTracerProvider::builder().build()
}

/// Build the [`tracing_opentelemetry`] layer from a provider.
///
/// The returned layer must be added to the `tracing_subscriber::Registry` so
/// that every tracing span is backed by an `OTel` span with a real `trace_id`.
pub fn otel_layer<S>(
    provider: &SdkTracerProvider,
) -> tracing_opentelemetry::OpenTelemetryLayer<S, opentelemetry_sdk::trace::Tracer>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    let tracer = provider.tracer("nsip-mcp");
    tracing_opentelemetry::layer().with_tracer(tracer)
}

// ---------------------------------------------------------------------------
// JSON event formatter with OTel trace context
// ---------------------------------------------------------------------------

/// JSON event formatter that includes `OpenTelemetry` `trace_id` and `span_id`
/// on every log line.
///
/// Drop-in replacement for `tracing_subscriber::fmt::format::Json`. Expects
/// the subscriber to use [`tracing_subscriber::fmt::format::JsonFields`] as
/// the field formatter so that span fields are pre-formatted as JSON.
pub struct OtelJsonFormat {
    timer: tracing_subscriber::fmt::time::SystemTime,
}

impl Default for OtelJsonFormat {
    fn default() -> Self {
        Self {
            timer: tracing_subscriber::fmt::time::SystemTime,
        }
    }
}

impl<S, N> FormatEvent<S, N> for OtelJsonFormat
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let meta = event.metadata();

        let mut map = serde_json::Map::with_capacity(8);

        // Timestamp
        let mut ts_buf = String::with_capacity(32);
        self.timer
            .format_time(&mut Writer::new(&mut ts_buf))
            .map_err(|_| fmt::Error)?;
        map.insert("timestamp".to_owned(), serde_json::Value::String(ts_buf));

        // Level
        map.insert(
            "level".to_owned(),
            serde_json::Value::String(meta.level().to_string()),
        );

        // OTel trace context — walk the span tree to find OtelData.
        let has_otel = ctx
            .lookup_current()
            .is_some_and(|leaf| extract_otel_context(&leaf, &mut map));
        if !has_otel {
            map.insert("trace_id".to_owned(), serde_json::Value::Null);
            map.insert("span_id".to_owned(), serde_json::Value::Null);
        }

        // Fields (the event's own key-value pairs).
        let mut fields_map = serde_json::Map::new();
        event.record(&mut JsonMapVisitor(&mut fields_map));
        map.insert("fields".to_owned(), serde_json::Value::Object(fields_map));

        // Target
        map.insert(
            "target".to_owned(),
            serde_json::Value::String(meta.target().to_owned()),
        );

        // Current span (innermost).
        if let Some(leaf) = ctx.lookup_current() {
            map.insert("span".to_owned(), span_to_json::<S, N>(&leaf));

            // Full span list (root -> leaf) with fields.
            let spans: Vec<serde_json::Value> = leaf
                .scope()
                .from_root()
                .map(|s| span_to_json::<S, N>(&s))
                .collect();
            map.insert("spans".to_owned(), serde_json::Value::Array(spans));
        }

        // Write the JSON line.
        let json =
            serde_json::to_string(&serde_json::Value::Object(map)).map_err(|_| fmt::Error)?;
        writeln!(writer, "{json}")?;
        Ok(())
    }
}

/// Walk the span scope from root to leaf and extract the first `OTel`
/// `trace_id` / `span_id` into `map`.
///
/// Returns `true` if at least one of `trace_id` or `span_id` was inserted.
fn extract_otel_context<S>(
    leaf: &tracing_subscriber::registry::SpanRef<'_, S>,
    map: &mut serde_json::Map<String, serde_json::Value>,
) -> bool
where
    S: for<'lookup> LookupSpan<'lookup>,
{
    for span_ref in leaf.scope().from_root() {
        let otel_data = {
            let extensions = span_ref.extensions();
            extensions
                .get::<OtelData>()
                .map(|d| (d.trace_id(), d.span_id()))
        };
        let Some((trace_id, span_id)) = otel_data else {
            continue;
        };
        if let Some(tid) = trace_id {
            map.insert(
                "trace_id".to_owned(),
                serde_json::Value::String(tid.to_string()),
            );
        }
        if let Some(sid) = span_id {
            map.insert(
                "span_id".to_owned(),
                serde_json::Value::String(sid.to_string()),
            );
        }
        return true;
    }
    false
}

/// Serialize a span reference to a JSON object with `name` and pre-formatted
/// fields from [`FormattedFields<N>`].
fn span_to_json<'a, S, N>(span: &tracing_subscriber::registry::SpanRef<'a, S>) -> serde_json::Value
where
    S: for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    let mut obj = serde_json::Map::with_capacity(4);
    obj.insert(
        "name".to_owned(),
        serde_json::Value::String(span.name().to_owned()),
    );

    // FormattedFields<N> stores pre-formatted key-value pairs.
    // For JsonFields, the content looks like: `"key":"value","key2":"value2"`
    // Wrap in braces to parse as a JSON object. Drop the extensions guard
    // early to satisfy clippy::significant_drop_tightening.
    let raw = {
        let extensions = span.extensions();
        extensions
            .get::<FormattedFields<N>>()
            .map(std::string::ToString::to_string)
    };
    if let Some(raw) = raw.filter(|s| !s.is_empty()) {
        let wrapped = format!("{{{raw}}}");
        if let Ok(serde_json::Value::Object(parsed)) = serde_json::from_str(&wrapped) {
            obj.extend(parsed);
        }
    }

    serde_json::Value::Object(obj)
}

// ---------------------------------------------------------------------------
// serde_json visitor for event fields
// ---------------------------------------------------------------------------

/// Visits tracing event fields and writes them into a `serde_json::Map`.
struct JsonMapVisitor<'a>(&'a mut serde_json::Map<String, serde_json::Value>);

impl tracing::field::Visit for JsonMapVisitor<'_> {
    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.0
            .insert(field.name().to_owned(), serde_json::Value::from(value));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.0
            .insert(field.name().to_owned(), serde_json::Value::from(value));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.0
            .insert(field.name().to_owned(), serde_json::Value::from(value));
    }

    fn record_i128(&mut self, field: &tracing::field::Field, value: i128) {
        self.0.insert(
            field.name().to_owned(),
            serde_json::Value::from(value.to_string()),
        );
    }

    fn record_u128(&mut self, field: &tracing::field::Field, value: u128) {
        self.0.insert(
            field.name().to_owned(),
            serde_json::Value::from(value.to_string()),
        );
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.0
            .insert(field.name().to_owned(), serde_json::Value::from(value));
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.0.insert(
            field.name().to_owned(),
            serde_json::Value::String(value.to_owned()),
        );
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn fmt::Debug) {
        self.0.insert(
            field.name().to_owned(),
            serde_json::Value::String(format!("{value:?}")),
        );
    }
}

// ---------------------------------------------------------------------------
// Writer adapter: io::Write -> fmt::Write
// ---------------------------------------------------------------------------

/// Wraps an [`io::Write`] implementor so it can be used with
/// [`tracing_subscriber::fmt::Layer::with_writer`].
pub struct FmtWriteAdapter<W>(
    /// The inner writer.
    pub W,
);

impl<W: io::Write> io::Write for FmtWriteAdapter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn tracer_provider_initialises() {
        let provider = init_tracer_provider();
        let _tracer = provider.tracer("test");
    }

    #[test]
    fn otel_json_format_default_creates() {
        let _fmt = OtelJsonFormat::default();
    }

    #[test]
    fn json_map_visitor_records_types() {
        use tracing::field::Visit;

        let mut map = serde_json::Map::new();
        let mut visitor = JsonMapVisitor(&mut map);

        let field_set = tracing::field::FieldSet::new(
            &["test_str", "test_num", "test_bool"],
            tracing::callsite::Identifier(&CALLSITE),
        );
        let str_field = field_set.field("test_str").unwrap();
        let num_field = field_set.field("test_num").unwrap();
        let bool_field = field_set.field("test_bool").unwrap();

        visitor.record_str(&str_field, "hello");
        visitor.record_i64(&num_field, 42);
        visitor.record_bool(&bool_field, true);

        assert_eq!(map["test_str"], serde_json::json!("hello"));
        assert_eq!(map["test_num"], serde_json::json!(42));
        assert_eq!(map["test_bool"], serde_json::json!(true));
    }

    #[test]
    fn json_map_visitor_records_f64() {
        use tracing::field::Visit;

        let mut map = serde_json::Map::new();
        let mut visitor = JsonMapVisitor(&mut map);

        let field_set =
            tracing::field::FieldSet::new(&["val"], tracing::callsite::Identifier(&CALLSITE_F64));
        let field = field_set.field("val").unwrap();
        visitor.record_f64(&field, 2.78);
        assert_eq!(map["val"], serde_json::json!(2.78));
    }

    #[test]
    fn json_map_visitor_records_u64() {
        use tracing::field::Visit;

        let mut map = serde_json::Map::new();
        let mut visitor = JsonMapVisitor(&mut map);

        let field_set =
            tracing::field::FieldSet::new(&["val"], tracing::callsite::Identifier(&CALLSITE_F64));
        let field = field_set.field("val").unwrap();
        visitor.record_u64(&field, 999);
        assert_eq!(map["val"], serde_json::json!(999));
    }

    #[test]
    fn json_map_visitor_records_debug() {
        use tracing::field::Visit;

        let mut map = serde_json::Map::new();
        let mut visitor = JsonMapVisitor(&mut map);

        let field_set =
            tracing::field::FieldSet::new(&["val"], tracing::callsite::Identifier(&CALLSITE_F64));
        let field = field_set.field("val").unwrap();
        visitor.record_debug(&field, &vec![1, 2, 3]);
        assert_eq!(map["val"], serde_json::json!("[1, 2, 3]"));
    }

    #[test]
    fn json_map_visitor_records_i128() {
        use tracing::field::Visit;

        let mut map = serde_json::Map::new();
        let mut visitor = JsonMapVisitor(&mut map);

        let field_set =
            tracing::field::FieldSet::new(&["val"], tracing::callsite::Identifier(&CALLSITE_F64));
        let field = field_set.field("val").unwrap();
        visitor.record_i128(&field, 42_i128);
        assert_eq!(map["val"], serde_json::json!("42"));
    }

    #[test]
    fn json_map_visitor_records_u128() {
        use tracing::field::Visit;

        let mut map = serde_json::Map::new();
        let mut visitor = JsonMapVisitor(&mut map);

        let field_set =
            tracing::field::FieldSet::new(&["val"], tracing::callsite::Identifier(&CALLSITE_F64));
        let field = field_set.field("val").unwrap();
        visitor.record_u128(&field, 42_u128);
        assert_eq!(map["val"], serde_json::json!("42"));
    }

    #[test]
    fn fmt_write_adapter_delegates() {
        let buf = Vec::new();
        let mut adapter = FmtWriteAdapter(buf);
        io::Write::write_all(&mut adapter, b"hello").unwrap();
        io::Write::flush(&mut adapter).unwrap();
        assert_eq!(&adapter.0, b"hello");
    }

    // Dummy callsites for field tests.
    static CALLSITE: FakeCallsite = FakeCallsite;
    static CALLSITE_F64: FakeCallsite = FakeCallsite;

    struct FakeCallsite;

    impl tracing::callsite::Callsite for FakeCallsite {
        fn set_interest(&self, _: tracing::subscriber::Interest) {}
        fn metadata(&self) -> &tracing::Metadata<'_> {
            // This is only used in tests and the static is safe here.
            // We use a Box::leak pattern since LazyLock with self-referential
            // fields gets complicated.
            static META: std::sync::LazyLock<tracing::Metadata<'static>> =
                std::sync::LazyLock::new(|| {
                    tracing::Metadata::new(
                        "test",
                        "test",
                        tracing::Level::INFO,
                        None,
                        None,
                        None,
                        tracing::field::FieldSet::new(
                            &["test_str", "test_num", "test_bool"],
                            tracing::callsite::Identifier(&CALLSITE),
                        ),
                        tracing::metadata::Kind::EVENT,
                    )
                });
            &META
        }
    }
}
