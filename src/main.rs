use prometheus::{Encoder, Gauge, Registry, TextEncoder};
use prometheus_publisher::{configure_serial, populate_metrics};
use std::net::SocketAddr;
use warp::{Filter, Rejection, Reply};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create a Prometheus registry
    let registry = Registry::new();

    // Define a gauge to track speed
    let speed = Gauge::new("racecar_speed", "Current speed of the racecar")?;

    // CH: why is this cloned?
    registry.register(Box::new(speed.clone()))?;

    let serial_framed = configure_serial()?;

    // Spawn a task that reads from the serial port and populates metrics
    tokio::spawn(async move {
        populate_metrics(serial_framed, speed).await;
    });

    // Define a route to expose metrics in Prometheus format
    let metrics_route = warp::path!("metrics")
        .and_then(move || get_metrics(registry.clone()))
        .with(warp::cors().allow_any_origin());

    // Start the server
    let addr: SocketAddr = ([0, 0, 0, 0], 9100).into(); // Change the port if needed
    println!("Server listening on {}", addr);
    warp::serve(metrics_route).run(addr).await;

    Ok(())
}

/// Function to generate the encoded protobuf response
async fn get_metrics(registry: Registry) -> Result<impl Reply, Rejection> {
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();

    Ok(warp::reply::with_header(
        String::from_utf8(buffer).unwrap(),
        "Content-Type",
        encoder.format_type(),
    ))
}
