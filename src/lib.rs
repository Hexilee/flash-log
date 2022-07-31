use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::thread::JoinHandle;
use std::time::Instant;

use anyhow::{anyhow, Result};
use bytes::Bytes;
use tokio::sync::oneshot;

#[cfg(test)]
mod test;

enum Message {
    Exit,
    Write {
        data: Bytes,
        waker: oneshot::Sender<()>,
    },
}

pub struct Logger {
    sender: Sender<Message>,
    worker_handler: Option<JoinHandle<()>>,
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
        let (sender, receiver) = channel();
        let worker_handler = std::thread::spawn(move || {
            let mut last_throughput = 0.;
            let mut batch_size = Self::BLOCK_SIZE;
            let mut batch = vec![];
            loop {
                batch.clear();
                batch.reserve(batch_size);

                let mut wakers = vec![];
                let start = Instant::now();
                loop {
                    match receiver.try_recv() {
                        Ok(Message::Exit) => return,
                        Err(TryRecvError::Empty) => break,
                        Ok(Message::Write { data, waker }) => {
                            wakers.push(waker);
                            batch.extend_from_slice(&data);
                            if batch.len() + avg_msg_size > batch_size {
                                break;
                            }
                        }
                        _ => unreachable!("message sender cannot be dropped"),
                    }
                }

                file.write_all(&batch).expect("write data failed");
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
                for waker in wakers {
                    waker.send(()).expect("Fail to wake log writer")
                }
            }
        });

        Ok(Self {
            sender,
            worker_handler: Some(worker_handler),
        })
    }

    pub async fn write_log(&self, data: Bytes) -> Result<()> {
        let (waker, receiver) = oneshot::channel();
        let _ = self.sender.send(Message::Write { data, waker });
        receiver
            .await
            .map_err(|e| anyhow!("Logger worker thread exit: {}", e))
    }

    pub fn shutdown(&mut self) {
        if let Some(handler) = self.worker_handler.take() {
            let _ = self.sender.send(Message::Exit);
            handler.join().expect("Failed to join logger thread");
        }
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        self.shutdown();
    }
}
