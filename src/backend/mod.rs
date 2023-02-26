pub mod file;
pub mod serial;

use bytes::Bytes;
use crossbeam_channel::{Receiver, Sender};
use color_eyre::eyre::Result;

pub trait Backend {
    fn start(&mut self) -> Result<()>;
    fn get_sender(&mut self) -> &mut Sender<Bytes>;
    fn get_receiver(&mut self) -> &mut Receiver<Bytes>;
}
