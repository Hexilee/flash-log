use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::sync::mpsc::{sync_channel, SyncSender, TryRecvError};
use std::thread::JoinHandle;
use std::time::Instant;

use anyhow::{anyhow, Result};
use bytes::Bytes;
use tokio::sync::oneshot;

#[cfg(test)]
mod test;

enum IOMessage {
    Exit,
    Write {
        data: Bytes,
        waker: oneshot::Sender<()>,
    },
}

enum WakeMessage {
    Exit,
    Wake(Vec<Waker>),
}

struct Waker(oneshot::Sender<()>);

pub struct Logger {
    io_sender: SyncSender<IOMessage>,
    io_worker: Option<JoinHandle<()>>,
    waker_sender: SyncSender<WakeMessage>,
    waker_worker: Option<JoinHandle<()>>,
}

impl Waker {
    fn wake(self) -> Result<()> {
        self.0.send(()).map_err(|_| anyhow!("fail to send signal"))
    }
}

impl Logger {
    const DEFAULT_MAX_BUFFER: usize = 512 * 1024 * 1024;
    const BLOCK_SIZE: usize = 4096;
    const AVG_MSG_SIZE: usize = 100;

    pub fn open(
        path: impl AsRef<Path>,
        max_buffer_o: Option<usize>,
        avg_msg_size_o: Option<usize>,
    ) -> Result<Self> {
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .custom_flags(libc::O_DIRECT)
            .open(path)?;
        let max_buffer = max_buffer_o.unwrap_or(Self::DEFAULT_MAX_BUFFER);
        let avg_msg_size = avg_msg_size_o.unwrap_or(Self::AVG_MSG_SIZE);
        let (io_sender, io_receiver) = sync_channel(1000_000);
        let (waker_sender, waker_receiver) = sync_channel::<WakeMessage>(1000);
        let waker_sender_ref = waker_sender.clone();

        let waker_worker = std::thread::spawn(move || {
            while let Ok(msg) = waker_receiver.recv() {
                match msg {
                    WakeMessage::Exit => break,
                    WakeMessage::Wake(wakers) => {
                        for waker in wakers {
                            waker.wake().expect("wake log writer");
                        }
                    }
                }
            }
        });

        let io_worker = std::thread::spawn(move || {
            let mut last_throughput = 0.;
            let mut batch_size = Self::BLOCK_SIZE;
            let mut batch = vec![];
            loop {
                let start = Instant::now();
                batch.clear();
                batch.reserve(batch_size);

                let mut wakers = vec![];
                loop {
                    match io_receiver.try_recv() {
                        Ok(IOMessage::Exit) => return,
                        Err(TryRecvError::Empty) => break,
                        Ok(IOMessage::Write { data, waker }) => {
                            wakers.push(Waker(waker));
                            batch.extend_from_slice(&data);
                            if batch.len() + avg_msg_size > batch_size {
                                break;
                            }
                        }
                        _ => unreachable!("message sender cannot be dropped"),
                    }
                }

                if batch.is_empty() {
                    continue;
                }

                file.write_all(&batch).expect("write data failed");
                waker_sender_ref
                    .send(WakeMessage::Wake(wakers))
                    .expect("waker sender cannot be dropped");
                let throughput = batch.len() as f64 / start.elapsed().as_secs_f64();
                let errs = (throughput - last_throughput) / last_throughput;
                if errs >= 0.1 {
                    batch_size *= 2;
                } else if errs <= -0.1 {
                    batch_size = batch_size * 3 / 4;
                }
                let min_buffer_size = max_buffer.min(batch_size);
                let mut blocks = min_buffer_size / Self::BLOCK_SIZE;
                if min_buffer_size % Self::BLOCK_SIZE > 0 {
                    blocks += 1;
                }
                batch_size = blocks * Self::BLOCK_SIZE;
                last_throughput = throughput;
            }
        });

        Ok(Self {
            io_sender,
            io_worker: Some(io_worker),
            waker_sender,
            waker_worker: Some(waker_worker),
        })
    }

    pub async fn write_log(&self, data: Bytes) -> Result<()> {
        let (waker, receiver) = oneshot::channel();
        let _ = self.io_sender.send(IOMessage::Write { data, waker });
        receiver
            .await
            .map_err(|e| anyhow!("logger worker thread exit: {}", e))
    }

    pub fn shutdown(&mut self) {
        if let Some(handler) = self.io_worker.take() {
            let _ = self.io_sender.send(IOMessage::Exit);
            handler.join().expect("failed to join io worker");
        }
        if let Some(handler) = self.waker_worker.take() {
            let _ = self.waker_sender.send(WakeMessage::Exit);
            handler.join().expect("failed to join wake worker");
        }
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        self.shutdown();
    }
}
