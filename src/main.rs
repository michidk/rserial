mod ui;
mod serial;

use bytes::Bytes;
use color_eyre::eyre::{eyre, Result};
use clap::{command, Parser, Subcommand};
use ringbuffer::ConstGenericRingBuffer;
use std::{str, sync::{Arc, Mutex}, thread};
use crossbeam_channel::{Sender, Receiver};

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


// trait SerialConsumer {
//     f);
// }


fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();

    let args = Args::parse();

    let (tx, rx): (Sender<Bytes>, Receiver<Bytes>) = crossbeam_channel::unbounded();

    match args.subcommand {
        Action::List => {
            serial::list_ports()?;
        }
        Action::Interactive { port, baud } => {

            // let mut buffer = Arc::new(Mutex::new(ConstGenericRingBuffer::<String, BUFFER_SIZE>::new()));

            println!("Interactive mode");
            serial::interactive(port, baud, tx);
            ui::start(rx)?;

        }
        Action::Raw { port, baud } => {
            println!("Raw mode");
        }
    }

    Ok(())
}
