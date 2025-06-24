use anyhow::Result;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, SpanExporter, WithExportConfig};
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

use crate::config::TelemetryConfig;

pub fn initialize_telemetry_with_configuration(config: &TelemetryConfig) -> Result<()> {
    if !config.enabled {
        initialize_basic_logging(&config.log_level, &config.excluded_modules)?;
        return Ok(());
    }

    let env_filter = create_environment_filter(&config.log_level, &config.excluded_modules)?;

    let logger_provider = create_logger_provider(config)?;
    let logger_layer = OpenTelemetryTracingBridge::new(&logger_provider);

    let tracer_provider = create_tracer_provider(config)?;
    let tracer = tracer_provider.tracer(config.service_name.clone());
    let tracer_layer = OpenTelemetryLayer::new(tracer);

    let fmt_layer = tracing_subscriber::fmt::layer().with_thread_names(true);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .with(logger_layer)
        .with(tracer_layer)
        .init();

    Ok(())
}

fn initialize_basic_logging(log_level: &str, excluded_modules: &[String]) -> Result<()> {
    let environment_filter = create_environment_filter(log_level, excluded_modules)?;
    let fmt_layer = tracing_subscriber::fmt::layer().with_thread_names(true);

    tracing_subscriber::registry()
        .with(environment_filter)
        .with(fmt_layer)
        .init();

    Ok(())
}

fn create_environment_filter(log_level: &str, excluded_modules: &[String]) -> Result<EnvFilter> {
    let mut filter = EnvFilter::new(log_level);

    for module in excluded_modules {
        let directive = format!("{}=off", module);
        filter = filter.add_directive(directive.parse()?);
    }

    Ok(filter)
}

fn create_opentelemetry_resource(service_name: String) -> Resource {
    Resource::builder().with_service_name(service_name).build()
}

fn create_tracer_provider(config: &TelemetryConfig) -> Result<SdkTracerProvider> {
    let exporter_builder = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&config.otlp_endpoint);

    let exporter = exporter_builder
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create span exporter: {}", e))?;

    let tracer_provider = SdkTracerProvider::builder()
        .with_resource(create_opentelemetry_resource(config.service_name.clone()))
        .with_batch_exporter(exporter)
        .build();

    Ok(tracer_provider)
}

fn create_logger_provider(config: &TelemetryConfig) -> Result<SdkLoggerProvider> {
    let exporter_builder = LogExporter::builder()
        .with_tonic()
        .with_endpoint(&config.otlp_endpoint);

    let exporter = exporter_builder
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create log exporter: {}", e))?;

    let logger_provider = SdkLoggerProvider::builder()
        .with_resource(create_opentelemetry_resource(config.service_name.clone()))
        .with_batch_exporter(exporter)
        .build();

    Ok(logger_provider)
}
