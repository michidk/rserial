use ::futures::pin_mut;
use ::futures::StreamExt;
use bytes::Bytes;
use color_eyre::eyre::{self, Result};
use crossbeam_channel::{Receiver, Sender};
use tokio_serial::SerialPortBuilderExt;
use tokio_util::codec::BytesCodec;

use super::Backend;


pub struct SerialBackend {
    tx: Sender<Bytes>,
    rx: Receiver<Bytes>,
// }
// pub struct SerialBackendProperties {
    port: String,
    baud_rate: u32,
}

impl SerialBackend {
    pub fn new(port: String, baud_rate: u32) -> Result<Self> {
        let (tx, rx) = crossbeam_channel::unbounded();

        Ok(Self {
            port,
            baud_rate,
            tx,
            rx,
        })
    }

    pub async fn interactive(&mut self) -> Result<()> {
        let stream = tokio_serial::new(self.port.clone(), self.baud_rate).open_native_async()?;
        let (rx_stream, _tx_stream) = tokio::io::split(stream);

        // TODO: write own decoder similar to https://github.com/dhylands/serial-monitor/blob/master/src/string_decoder.rs to insert replacement characters
        let serial_reader = tokio_util::codec::FramedRead::new(rx_stream, BytesCodec::new());
        pin_mut!(serial_reader);

        loop {
            match serial_reader.next().await {
                Some(Ok(serial_event)) => {
                    // println!("Serial Event:{:?}\r", serial_event);
                    let bytes = serial_event.freeze();
                    self.tx.send(bytes)?;
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
}

// impl Backend for SerialBackend {
//     fn get_sender(&mut self) -> &mut Sender<Bytes> {
//         &mut self.tx
//     }

//     fn get_receiver(&mut self) -> &mut Receiver<Bytes> {
//         &mut self.rx
//     }
// }
