use std::{thread, time::Duration, io::{BufReader, Read}};

use bytes::{Bytes, BytesMut};
use color_eyre::eyre::{eyre, Result};
use crossbeam_channel::{Receiver, Sender};
use serialport::SerialPortType;

pub fn list_ports() -> Result<()> {
    match serialport::available_ports() {
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

// inspired by dhylands/serial-monitor
// Converts key events from crossterm into appropriate character/escape sequences which are then
// sent over the serial connection.
// fn handle_key_event(key_event: KeyEvent) -> Result<Option<Bytes>> {
//     // The following escape sequeces come from the MicroPython codebase.
//     //
//     //  Up      ESC [A
//     //  Down    ESC [B
//     //  Right   ESC [C
//     //  Left    ESC [D
//     //  Home    ESC [H  or ESC [1~
//     //  End     ESC [F  or ESC [4~
//     //  Del     ESC [3~
//     //  Insert  ESC [2~

//     let mut buf = [0; 4];

//     let key_str: Option<&[u8]> = match key_event.code {
//         KeyCode::Backspace => Some(b"\x08"),
//         KeyCode::Enter => Some(b"\r\n"), // CRLF
//         KeyCode::Left => Some(b"\x1b[D"),
//         KeyCode::Right => Some(b"\x1b[C"),
//         KeyCode::Home => Some(b"\x1b[H"),
//         KeyCode::End => Some(b"\x1b[F"),
//         KeyCode::Up => Some(b"\x1b[A"),
//         KeyCode::Down => Some(b"\x1b[B"),
//         KeyCode::Tab => Some(b"\x09"),
//         KeyCode::Delete => Some(b"\x1b[3~"),
//         KeyCode::Insert => Some(b"\x1b[2~"),
//         KeyCode::Esc => Some(b"\x1b"),
//         KeyCode::Char(ch) => {
//             if key_event.modifiers & KeyModifiers::CONTROL == KeyModifiers::CONTROL {
//                 buf[0] = ch as u8;
//                 if (ch >= 'a' && ch <= 'z') || (ch == ' ') {
//                     buf[0] &= 0x1f;
//                     Some(&buf[0..1])
//                 } else if ch >= '4' && ch <= '7' {
//                     // crossterm returns Control-4 thru 7 for \x1c thru \x1f
//                     buf[0] = (buf[0] + 8) & 0x1f;
//                     Some(&buf[0..1])
//                 } else {
//                     Some(ch.encode_utf8(&mut buf).as_bytes())
//                 }
//             } else {
//                 Some(ch.encode_utf8(&mut buf).as_bytes())
//             }
//         }
//         _ => None,
//     };
//     if let Some(key_str) = key_str {

//         if let Ok(val) = std::str::from_utf8(key_str) {
//             print!("{}", val);
//             std::io::stdout().flush()?;
//         }

//         Ok(Some(Bytes::copy_from_slice(key_str)))
//     } else {
//         Ok(None)
//     }
// }

pub fn interactive(port: String, baud_rate: u32, sender: Sender<Bytes>) {
    let port = serialport::new(port, baud_rate)
        .timeout(Duration::from_millis(10))
        .open()
        .expect("Failed to open serial port.");

    // #[cfg(unix)]
    // port.set_exclusive(false)
        // .expect("Unable to set serial port exclusive to false");

    thread::spawn(move || {
            // let mut bytes = BytesMut::with_capacity(1024);
            let mut bytes = [0u8; 1024];
            let mut buf = BufReader::with_capacity(1024, port);
            loop {
                match buf.read(&mut bytes) {
                    Ok(0) => {
                        // println!("No data");
                        break;
                    }
                    Ok(n) => {
                        // println!("Read {} bytes", n);
                        // let data = bytes.split_to(n);
                        // sender.send(data.freeze()).unwrap();
                        sender.send(Bytes::copy_from_slice(&bytes[0..n])).unwrap();
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        break;
                    }
                }
            }
    });
}
