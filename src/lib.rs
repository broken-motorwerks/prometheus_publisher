use anyhow::Context;
use futures::stream::StreamExt;
use prometheus::Gauge;
use telemetry_types::PostCardCodec;
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tokio_util::codec::{Decoder, Framed};

pub type PostCardFramed = Framed<SerialStream, PostCardCodec>;

/// Create a port and return the [Framed] struct for reading
pub fn configure_serial() -> anyhow::Result<PostCardFramed> {
    // Configure serial port

    let mut port = tokio_serial::new("/dev/ttyUSB0", 115200).open_native_async()?;

    // CH: not sure if this is needed but it was in the docs
    #[cfg(unix)]
    port.set_exclusive(false)
        .context("Unable to set serial port exclusive to false")?;

    Ok(PostCardCodec.framed(port))
}

/// This function converts structured Frames and updates Prometheus
pub async fn populate_metrics(mut framed: PostCardFramed, gauge: Gauge) {
    while let Some(telem) = framed.next().await {
        let telem = telem.unwrap();

        println!("Received: {:?}", telem);

        gauge.set(telem.rpm.into());
    }
}
