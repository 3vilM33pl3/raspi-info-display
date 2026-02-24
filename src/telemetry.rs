use std::env;

use opentelemetry::global;
use opentelemetry::KeyValue;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_sdk::{trace as sdktrace, Resource};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use crate::errors::{AppError, Result};

pub struct Telemetry {
    _tracer_provider: sdktrace::TracerProvider,
    _meter_provider: SdkMeterProvider,
}

pub fn init() -> Result<Telemetry> {
    let service_name = env::var("OTEL_SERVICE_NAME")
        .unwrap_or_else(|_| "raspi-info-display".to_string());
    let endpoint = env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://monolith:4317".to_string());

    global::set_text_map_propagator(TraceContextPropagator::new());

    let resource = Resource::new(vec![KeyValue::new("service.name", service_name)]);

    let tracer_provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint.clone()),
        )
        .with_trace_config(
            sdktrace::Config::default()
                .with_sampler(sdktrace::Sampler::AlwaysOn)
                .with_resource(resource.clone()),
        )
        .install_batch(Tokio)
        .map_err(|e| AppError::application(&format!("Telemetry trace init failed: {}", e)))?;
    let tracer = tracer_provider.tracer("info_display");

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .try_init()
        .map_err(|e| AppError::application(&format!("Telemetry logging init failed: {}", e)))?;

    let meter_provider = opentelemetry_otlp::new_pipeline()
        .metrics(Tokio)
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint),
        )
        .with_resource(resource)
        .build()
        .map_err(|e| AppError::application(&format!("Telemetry metrics init failed: {}", e)))?;
    global::set_meter_provider(meter_provider.clone());

    Ok(Telemetry {
        _tracer_provider: tracer_provider,
        _meter_provider: meter_provider,
    })
}
