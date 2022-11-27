mod ui;
mod serial;

use bytes::Bytes;
use color_eyre::eyre::{Result};
use clap::{command, Parser, Subcommand};
use std::{str};
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


fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();

    let args = Args::parse();

    let (messages_tx, messages_rx): (Sender<Bytes>, Receiver<Bytes>) = crossbeam_channel::unbounded();
    let (commands_tx, commands_rx): (Sender<Bytes>, Receiver<Bytes>) = crossbeam_channel::unbounded();

    match args.subcommand {
        Action::List => {
            serial::list_ports()?;
        }
        Action::Interactive { port, baud } => {

            // let mut buffer = Arc::new(Mutex::new(ConstGenericRingBuffer::<String, BUFFER_SIZE>::new()));

            println!("Interactive mode");
            serial::interactive(port, baud, messages_tx, commands_rx);
            ui::start(messages_rx, commands_tx)?;

        }
        Action::Raw { port, baud } => {
            println!("Raw mode");
        }
    }

    Ok(())
}
