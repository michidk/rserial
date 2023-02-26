use bytes::Bytes;
use crossbeam_channel::{Sender, Receiver};
use color_eyre::eyre::Result;



pub struct Backend {
    tx: Sender<Bytes>,
    rx: Receiver<Bytes>,
    consumer: Box<dyn BackendConsumer>,
    provider: Box<dyn BackendProvider>,
}

impl Backend {
    fn new(consumer: Box<dyn BackendConsumer>, provider: Box<dyn BackendProvider>) -> Backend {
        let (tx, rx) = crossbeam_channel::unbounded();
        Backend {
            tx, rx, consumer, provider
        }
    }
}

pub struct Frontend {
    tx: Sender<Bytes>,
    rx: Receiver<Bytes>,
    consumer: Box<dyn FrontendConsumer>,
    provider: Box<dyn FrontendProvider>,
}

impl Frontend {
    fn new(consumer: Box<dyn FrontendConsumer>, provider: Box<dyn FrontendProvider>) -> Frontend {
        let (tx, rx) = crossbeam_channel::unbounded();
        Frontend {
            rx, tx, consumer, provider
        }
    }
}

pub struct SerialApp {
    backend: Backend,
    frontend: Frontend,
}

impl SerialApp {
    pub fn new(backend: Backend, frontend: Frontend) -> SerialApp {
        SerialApp { backend, frontend}
    }
}

pub trait BackendConsumer {
    fn consume(&self, bytes: Bytes) -> Result<()>;
}

pub trait BackendProvider {
    fn provide(&self) -> Result<Bytes>;
}

pub trait FrontendConsumer {
    // fn 
}

pub trait FrontendProvider {

}
