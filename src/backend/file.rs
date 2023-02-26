use super::Backend;
use async_fs::File;
use bytes::Bytes;
use color_eyre::eyre::Result;
use crossbeam_channel::{Receiver, Sender};
use futures::io::AsyncBufReadExt;
use futures::io::BufReader;
use tokio::task;
use tokio::time::{sleep, Duration};

pub struct FileBackend {
    // reader: BufReader<File>,
    path: String,
    tx: Sender<Bytes>,
    rx: Receiver<Bytes>,
}

impl FileBackend {
    pub async fn new(path: &str) -> Result<Self> {


        let (tx, rx) = crossbeam_channel::unbounded();
        Ok(Self { path: path.to_string(), tx, rx })
    }
}

impl Backend for FileBackend {
    fn start(&mut self) -> Result<()> {
        let path = self.path.clone();
        task::spawn(async {
            let file = File::open(path).await.expect("Cannot open file.");
            let mut reader = BufReader::new(file);

            if let Err(e) = run(&mut reader).await {
                println!("Error: {:?}", e);
            }
        });
        Ok(())
    }

    fn get_sender(&mut self) -> &mut Sender<Bytes> {
        &mut self.tx
    }

    fn get_receiver(&mut self) -> &mut Receiver<Bytes> {
        &mut self.rx
    }
}

async fn run(reader: &mut BufReader<File>) -> Result<()> {
    while !reader.buffer().is_empty() {
        let mut buf = Vec::<u8>::new();
        reader.read_until(b'\n', &mut buf).await?;

        // self.tx.send(Bytes::from(buf))?;
        sleep(Duration::from_millis(500)).await;
    }
    Ok(())
}
