pub mod tui;

use bytes::Bytes;
use crossbeam_channel::{Receiver, Sender};

// pub struct Frontend {
//     rx: Receiver<Bytes>, // reading bytes from the frontend
//     tx: Sender<Bytes>,   // writing bytes to the frontend
// }

pub trait Frontend {
    fn get_sender(&mut self) -> &mut Sender<Bytes>;
    fn get_receiver(&mut self) -> &mut Receiver<Bytes>;
}
