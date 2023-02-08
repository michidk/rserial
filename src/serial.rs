use bytes::Bytes;
use ::futures::pin_mut;
use color_eyre::eyre::{Result, self};
use crossbeam_channel::{Receiver, Sender};
use tokio_util::codec::BytesCodec;
use ::futures::StreamExt;
use tokio::task;
use tokio_serial::{SerialPortBuilderExt};

pub async fn interactive(
    port: String,
    baud_rate: u32,
    tx: Sender<Bytes>,
    _rx: Receiver<Bytes>,
) -> Result<()> {
    let stream = tokio_serial::new(port, baud_rate).open_native_async()?;
    let (rx_stream, _tx_stream) = tokio::io::split(stream);

    task::spawn(async move {
        if let Err(e) = run(rx_stream, _tx_stream, tx, _rx).await {
            println!("Error: {:?}", e);
        }
    });

    Ok(())
}

async fn run(
    rx_stream: tokio::io::ReadHalf<tokio_serial::SerialStream>,
    _tx_stream: tokio::io::WriteHalf<tokio_serial::SerialStream>,
    tx: Sender<Bytes>,
    _rx: Receiver<Bytes>,
) -> Result<()> {
    // TODO: writo own decoder similar to https://github.com/dhylands/serial-monitor/blob/master/src/string_decoder.rs to insert replacement characters
    let serial_reader = tokio_util::codec::FramedRead::new(rx_stream, BytesCodec::new());
    pin_mut!(serial_reader);

    loop {
        match serial_reader.next().await {
            Some(Ok(serial_event)) => {
                // println!("Serial Event:{:?}\r", serial_event);
                let bytes = serial_event.freeze();
                tx.send(bytes)?;

            }
            Some(Err(e)) => {
                println!("Serial Error: {:?}\r", e);
                // This most likely means that the serial port has been unplugged.
                break;
            }
            None => {
                println!("maybe_serial returned None\r");
                break;
            }
        }
    }

    Ok(())
}
