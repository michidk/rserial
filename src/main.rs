mod ui;
mod serial;

use bytes::{BytesMut, Bytes};
use color_eyre::eyre::{eyre, Result};
use clap::{command, Parser, Subcommand};
use crossterm::event::{EventStream, KeyCode, Event, KeyEvent, KeyModifiers};
use futures::{StreamExt, FutureExt, select, SinkExt};
use ringbuffer::ConstGenericRingBuffer;
use tokio::{task, sync::mpsc::{self, Sender, Receiver}};
use tokio_serial::{SerialPortType, SerialPortBuilderExt};
use tokio_util::codec::{Decoder, Encoder, BytesCodec};
use std::{env, io::{self, Write}, str, sync::{Arc, Mutex}};

#[derive(Debug, Parser)]
struct Args {
    #[command(subcommand)]
    subcommand: Action,
}

#[derive(Debug, Subcommand)]
enum Action {
    /// List the available serial ports
    #[clap(alias="l")]
    List,
    /// Interactivly open a serial port
    #[clap(alias="i")]
    Interactive {
        /// The serial port to open
        port: String,
        /// The baud rate to use
        #[clap(short, long, default_value="9600")]
        baud: u32,
    },
    #[clap(alias="r")]
    Raw {
        /// The serial port to open
        port: String,
        /// The baud rate to use
        #[clap(short, long, default_value="9600")]
        baud: u32,
    }
}


const BUFFER_SIZE: usize = 64;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();

    let args = Args::parse();

    let (tx, rx): (Sender<Bytes>, Receiver<Bytes>) = mpsc::channel(32);

    match args.subcommand {
        Action::List => {
            serial::list_ports()?;
        }
        Action::Interactive { port, baud } => {

            let mut buffer = Arc::new(Mutex::new(ConstGenericRingBuffer::<String, BUFFER_SIZE>::new()));

            // println!("Interactive mode");
            serial::interactive(port, buffer.clone()).await?;
            // task::spawn_blocking(ui::start(rx)).await.unwrap();
            ui::start(buffer).await?;

        }
        Action::Raw { port, baud } => {
            println!("Raw mode");
        }
    }

    Ok(())
}
