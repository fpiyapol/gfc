use anyhow::Result;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, SpanExporter};
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use std::sync::OnceLock;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

pub fn init_telemetry() -> Result<()> {
    let filter = EnvFilter::new("info")
        .add_directive("hyper=off".parse().unwrap())
        .add_directive("tonic=off".parse().unwrap())
        .add_directive("h2=off".parse().unwrap())
        .add_directive("reqwest=off".parse().unwrap())
        .add_directive("opentelemetry=off".parse().unwrap());

    let logger_provider = init_logs();
    let logger_layer = OpenTelemetryTracingBridge::new(&logger_provider);

    let fmt_layer = tracing_subscriber::fmt::layer().with_thread_names(true);

    let tracer_provider = init_tracer();
    let tracer = tracer_provider.tracer("gfc");
    let tracer_layer = OpenTelemetryLayer::new(tracer);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .with(logger_layer)
        .with(tracer_layer)
        .init();

    Ok(())
}

fn get_resource() -> Resource {
    static RESOURCE: OnceLock<Resource> = OnceLock::new();
    RESOURCE
        .get_or_init(|| Resource::builder().with_service_name("gfc").build())
        .clone()
}

fn init_tracer() -> SdkTracerProvider {
    let exporter = SpanExporter::builder()
        .with_tonic()
        .build()
        .expect("Failed to create span exporter");

    SdkTracerProvider::builder()
        .with_resource(get_resource())
        .with_batch_exporter(exporter)
        .build()
}

fn init_logs() -> SdkLoggerProvider {
    let exporter = LogExporter::builder()
        .with_tonic()
        .build()
        .expect("Failed to create log exporter");

    SdkLoggerProvider::builder()
        .with_resource(get_resource())
        .with_batch_exporter(exporter)
        .build()
}
