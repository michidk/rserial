use bytes::{BytesMut, Bytes};
use color_eyre::eyre::{eyre, Result};
use clap::{command, Parser, Subcommand};
use crossterm::event::{EventStream, KeyCode, Event, KeyEvent, KeyModifiers};
use futures::{StreamExt, FutureExt, select, SinkExt};
use tokio_serial::{SerialPortType, SerialPortBuilderExt};
use tokio_util::codec::{Decoder, Encoder, BytesCodec};
use std::{env, io::{self, Write}, str};

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
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();

    let args = Args::parse();

    match args.subcommand {
        Action::List => {
           list_ports()?;
        }
        Action::Interactive { port, baud } => {
            println!("Interactive mode");
            interactive_mode(port).await?;
        }
    }

    Ok(())
}

fn list_ports() -> Result<()> {
    match tokio_serial::available_ports() {
        Ok(ports) => {
            match ports.len() {
                0 => println!("No ports found."),
                1 => println!("Found 1 port:"),
                n => println!("Found {} ports:", n),
            };
            for p in ports {
                println!("  {}", p.port_name);
                match p.port_type {
                    SerialPortType::UsbPort(info) => {
                        println!("    Type: USB");
                        println!("    VID:{:04x} PID:{:04x}", info.vid, info.pid);
                        println!(
                            "     Serial Number: {}",
                            info.serial_number.as_ref().map_or("", String::as_str)
                        );
                        println!(
                            "      Manufacturer: {}",
                            info.manufacturer.as_ref().map_or("", String::as_str)
                        );
                        println!(
                            "           Product: {}",
                            info.product.as_ref().map_or("", String::as_str)
                        );
                        // waiting for the serialport 4.2.0 release (https://github.com/serialport/serialport-rs/issues/57)
                        // println!(
                        //     "         Interface: {}",
                        //     info.interface
                        //         .as_ref()
                        //         .map_or("".to_string(), |x| format!("{:02x}", *x))
                        // );
                    }
                    SerialPortType::BluetoothPort => {
                        println!("    Type: Bluetooth");
                    }
                    SerialPortType::PciPort => {
                        println!("    Type: PCI");
                    }
                    SerialPortType::Unknown => {
                        println!("    Type: Unknown");
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("{:?}", e);
            eprintln!("Error listing serial ports");
        }
    }
    Ok(())
}


// inspired by esp-rs/espflash
struct RawModeGuard;

impl RawModeGuard {
    pub fn new() -> Result<Self> {
        crossterm::terminal::enable_raw_mode()?;
        Ok(RawModeGuard)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        if let Err(e) = crossterm::terminal::disable_raw_mode() {
            eprintln!("Failed to disable raw mode: {}", e);
        }
    }
}

// inspired by dhylands/serial-monitor
// Converts key events from crossterm into appropriate character/escape sequences which are then
// sent over the serial connection.
fn handle_key_event(key_event: KeyEvent) -> Result<Option<Bytes>> {
    // The following escape sequeces come from the MicroPython codebase.
    //
    //  Up      ESC [A
    //  Down    ESC [B
    //  Right   ESC [C
    //  Left    ESC [D
    //  Home    ESC [H  or ESC [1~
    //  End     ESC [F  or ESC [4~
    //  Del     ESC [3~
    //  Insert  ESC [2~

    let mut buf = [0; 4];

    let key_str: Option<&[u8]> = match key_event.code {
        KeyCode::Backspace => Some(b"\x08"),
        KeyCode::Enter => Some(b"\r\n"), // CRLF
        KeyCode::Left => Some(b"\x1b[D"),
        KeyCode::Right => Some(b"\x1b[C"),
        KeyCode::Home => Some(b"\x1b[H"),
        KeyCode::End => Some(b"\x1b[F"),
        KeyCode::Up => Some(b"\x1b[A"),
        KeyCode::Down => Some(b"\x1b[B"),
        KeyCode::Tab => Some(b"\x09"),
        KeyCode::Delete => Some(b"\x1b[3~"),
        KeyCode::Insert => Some(b"\x1b[2~"),
        KeyCode::Esc => Some(b"\x1b"),
        KeyCode::Char(ch) => {
            if key_event.modifiers & KeyModifiers::CONTROL == KeyModifiers::CONTROL {
                buf[0] = ch as u8;
                if (ch >= 'a' && ch <= 'z') || (ch == ' ') {
                    buf[0] &= 0x1f;
                    Some(&buf[0..1])
                } else if ch >= '4' && ch <= '7' {
                    // crossterm returns Control-4 thru 7 for \x1c thru \x1f
                    buf[0] = (buf[0] + 8) & 0x1f;
                    Some(&buf[0..1])
                } else {
                    Some(ch.encode_utf8(&mut buf).as_bytes())
                }
            } else {
                Some(ch.encode_utf8(&mut buf).as_bytes())
            }
        }
        _ => None,
    };
    if let Some(key_str) = key_str {

        if let Ok(val) = std::str::from_utf8(key_str) {
            print!("{}", val);
            std::io::stdout().flush()?;
        }

        Ok(Some(Bytes::copy_from_slice(key_str)))
    } else {
        Ok(None)
    }
}

async fn monitor(stream: &mut tokio_serial::SerialStream) -> Result<()> {
    let mut reader = EventStream::new();
    let (rx_stream, tx_stream) = tokio::io::split(stream);

    // TODO: writo own decoder similar to https://github.com/dhylands/serial-monitor/blob/master/src/string_decoder.rs to insert replacement characters
    let mut serial_reader = tokio_util::codec::FramedRead::new(rx_stream, BytesCodec::new());
    let mut serial_sink = tokio_util::codec::FramedWrite::new(tx_stream, BytesCodec::new());

    loop {
        let mut event = reader.next().fuse();
        let mut serial_event = serial_reader.next().fuse();

        select! {
            maybe_event = event => {
                match maybe_event {
                    Some(Ok(event)) => {

                        if event == Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)) {
                            break;
                        }
                        if let Event::Key(key_event) = event {
                            if let Some(key) = handle_key_event(key_event)? {
                                serial_sink.send(key).await?;
                            }
                        } else {
                            println!("Unrecognized Event::{:?}\r", event);
                        }
                    }
                    Some(Err(e)) => println!("crossterm Error: {:?}\r", e),
                    None => {
                        println!("maybe_event returned None\r");
                    },
                }
            },
            maybe_serial = serial_event => {
                match maybe_serial {
                    Some(Ok(serial_event)) => {
                        println!("Serial Event:{:?}\r", serial_event);
                        // print!("{}", serial_event);
                        std::io::stdout().flush()?;
                    },
                    Some(Err(e)) => {
                        println!("Serial Error: {:?}\r", e);
                        // This most likely means that the serial port has been unplugged.
                        break;
                    },
                    None => {
                        println!("maybe_serial returned None\r");
                        break;
                    },
                }
            },
        };
    }

    Ok(())
}

async fn interactive_mode(port: String) -> Result<()> {
    let mut port = tokio_serial::new(port, 9600).open_native_async()?;

    // allowing multiple intances to connect at once, will "load balance" the incomming traffic. which is not what we want
    // #[cfg(unix)]
    // port.set_exclusive(false)
    //     .expect("Unable to set serial port exclusive to false");

    let _raw_mode = RawModeGuard::new()?;

    let result = monitor(&mut port).await;
    // let mut reader = LineCodec.framed(port);

    // while let Some(line_result) = reader.next().await {
    //     let line = line_result.expect("Failed to read line");
    //     println!("{}", line);
    // }

    Ok(())
}
