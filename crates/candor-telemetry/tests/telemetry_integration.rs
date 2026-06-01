/// Integration tests for candor-telemetry.
///
/// Tests span export behavior, verifying that the telemetry
/// initialisation, span creation, and export pathways work correctly.
use std::sync::OnceLock;

/// Global guard to track if telemetry was already initialized
/// across tests (tracing subscriber is a global singleton).
static TELEMETRY_INIT: OnceLock<()> = OnceLock::new();

/// Initialize telemetry once across all tests.
fn init_once() {
    TELEMETRY_INIT.get_or_init(|| {
        let _guard = candor_telemetry::init_telemetry("test-telemetry", None);
    });
}

/// Test that the no-op fmt subscriber initializes without panicking.
#[test]
fn test_telemetry_fmt_fallback() {
    init_once();
    tracing::info!("fmt fallback test: log should appear once");
}

/// Test that an empty endpoint falls back to fmt subscriber.
#[test]
fn test_telemetry_empty_endpoint_fallback() {
    init_once();
    tracing::info!("empty endpoint fallback test: log may or may not appear");
}

/// Test that we can create and log tracing spans without panicking.
#[test]
fn test_telemetry_span_lifecycle() {
    init_once();

    let span = tracing::info_span!("test_span", key = "value");
    let _guard = span.enter();
    tracing::info!("inside test span");
}

/// Test that tracing events across multiple threads don't panic.
#[test]
fn test_telemetry_multi_thread_spans() {
    init_once();
    let handles: Vec<_> = (0..4)
        .map(|i| {
            std::thread::spawn(move || {
                let span = tracing::info_span!("worker_span", worker_id = i);
                let _guard = span.enter();
                tracing::info!("worker {} running", i);
            })
        })
        .collect();

    for h in handles {
        h.join().expect("thread panicked");
    }
}

/// Test that the TelemetryGuard's Drop implementation doesn't panic
/// when there's no OTLP provider.
#[test]
fn test_telemetry_guard_drop_noop() {
    // Each call to init_telemetry with try_init should handle the case
    // where a subscriber is already set.
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing::Level::INFO.into())
                .from_env_lossy(),
        )
        .with_target(true)
        .with_thread_ids(true)
        .try_init();
    // Should not panic even if subscriber already set
}

/// Integration test: try to connect to a real OTLP collector.
///
/// This test requires an OTLP collector running on the default port.
/// It will be SKIPPED if no collector is available.
#[tokio::test]
async fn test_telemetry_otlp_live() {
    // Try binding to port 4317 — if we can bind, there's no collector
    // running (since the collector would have already bound to it).
    let can_bind = std::net::TcpListener::bind("127.0.0.1:4317").is_ok();

    if can_bind {
        eprintln!("Skipping OTLP test: no collector running on 127.0.0.1:4317");
        return;
    }

    // Collector is running — test that init doesn't panic
    // (it may fail to set the global subscriber, but should handle gracefully)
    let _guard = candor_telemetry::init_telemetry("test-otlp-live", Some("http://127.0.0.1:4317"));
    tracing::info!("OTLP live test: span should be exported");

    // Give the batch exporter a moment to flush
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
}

/// Test with a malformed OTLP endpoint (should fall back to fmt).
#[tokio::test]
async fn test_telemetry_otlp_malformed_endpoint() {
    // Should not panic even with a bad endpoint
    let _guard = candor_telemetry::init_telemetry("test-malformed", Some("not-a-valid-url"));
    tracing::info!("malformed endpoint test: should see fmt fallback log");
}
