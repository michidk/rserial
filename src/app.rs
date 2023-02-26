use crate::{backend::Backend, frontend::Frontend};
use color_eyre::eyre::Result;
use tokio::task;

struct SerialApp {
    // backend: B,
    // frontend: F,
}

impl SerialApp {
    pub fn spawn<B: Backend, F: Frontend>(backend: B, frontend: F) -> Result<()> {


        Ok(())
    }
}
