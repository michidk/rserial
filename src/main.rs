mod backend;
mod frontend;
mod app;

use backend::{file::FileBackend, serial::SerialBackend, Backend};
use bytes::Bytes;
use clap::{command, Parser, Subcommand};
use color_eyre::eyre::Result;
use crossbeam_channel::{Receiver, Sender};
use std::str;
use tokio::task;

#[derive(Debug, Parser)]
struct Args {
    #[command(subcommand)]
    subcommand: Action,
}

#[derive(Debug, Subcommand)]
enum Action {
    /// List the available serial ports
    #[clap(alias = "l")]
    List,
    /// Interactivly open a serial port
    #[clap(alias = "i")]
    Interactive {
        /// The serial port to open
        port: String,
        /// The baud rate to use
        #[clap(short, long, default_value = "9600")]
        baud: u32,
    },
    #[clap(alias = "r")]
    Raw {
        /// The serial port to open
        port: String,
        /// The baud rate to use
        #[clap(short, long, default_value = "9600")]
        baud: u32,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();

    let args = Args::parse();

    // task::spawn(async {
    //     let mut file_backend = FileBackend::new("test.txt")
    //         .await
    //         .expect("Could not initialize file backend");
    //     if let Err(e) = file_backend.start().await {
    //         println!("Error: {:?}", e);
    //     }
    // });

    // task::spawn(async {
    //     let mut serial_backend =
    //         SerialBackend::new("COM3".into(), 9600).expect("Could not initialize serial backend");
    //     if let Err(e) = serial_backend.interactive().await {
    //         println!("Error: {:?}", e);
    //     }
    // });

    // let (messages_tx, messages_rx): (Sender<Bytes>, Receiver<Bytes>) = crossbeam_channel::unbounded();
    // let (commands_tx, commands_rx): (Sender<Bytes>, Receiver<Bytes>) = crossbeam_channel::unbounded();

    // match args.subcommand {
    //     Action::List => {
    //         //     serial::list_ports()?;
    //     }
    //     Action::Interactive { port, baud } => {

    //         // let mut buffer = Arc::new(Mutex::new(ConstGenericRingBuffer::<String, BUFFER_SIZE>::new()));

    //         println!("Interactive mode");
    //         serial::interactive(port, baud, messages_tx, commands_rx).await?;
    //         ui::start(messages_rx, commands_tx)?;

    //     }
    //     Action::Raw { port, baud } => {
    //         println!("Raw mode");
    //     }
    // }

    Ok(())
}
