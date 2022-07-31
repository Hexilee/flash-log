use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::thread::JoinHandle;
// use std::time::Instant;

use anyhow::{anyhow, Result};
use bytes::Bytes;
use futures::executor::block_on;
use tokio::sync::{oneshot, mpsc};
use tokio::task;

#[cfg(test)]
mod test;

#[derive(Debug)]
enum Message {
    Exit,
    Write {
        data: Bytes,
        waker: oneshot::Sender<()>,
    },
}

pub struct Logger {
    batch_sender: mpsc::UnboundedSender<Message>,
    batch_worker: Option<JoinHandle<()>>,
    io_worker: Option<JoinHandle<()>>,
    io_sender: mpsc::UnboundedSender<Message>,
}

impl Logger {
    const DEFAULT_MAX_BUFFER: usize = 512 * 1024 * 1024;
    const BLOCK_SIZE: usize = 4096;

    pub fn open(path: impl AsRef<Path>, max_buffer_o: Option<usize>) -> Result<Self> {
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .custom_flags(libc::O_DIRECT)
            .open(path)?;
        let max_buffer = max_buffer_o.unwrap_or(Self::DEFAULT_MAX_BUFFER);
        let (batch_sender, mut batch_receiver) = mpsc::unbounded_channel();
        let (io_sender, mut io_receiver) = mpsc::unbounded_channel();

        let io_worker = std::thread::spawn(move || {
            block_on(async move {
                loop {
                    match io_receiver.recv().await {
                        Some(Message::Exit) => break,
                        Some(Message::Write {
                            data,
                            waker,
                        }) => {
                            file.write_all(&data).expect("write data failed");
                            waker.send(()).expect("fail to wake batch worker")
                        }
                        _ => unreachable!("message sender cannot be dropped"),
                    }
                }
            })
        });

        let io_sender_clone = io_sender.clone();
        let worker_handler = std::thread::spawn(move || {
            // let mut last_throughput = 0.;
            let batch_size = 1024 * Self::BLOCK_SIZE;

            let runtime = tokio::runtime::Runtime::new().expect("create runtime failed");
            // let mut batch = vec![];
            loop {
                // batch.clear();
                // if batch.capacity() < batch_size * 2 {
                //     // avoid batch reallocation
                //     let min_buffer_size = max_buffer.min(batch_size * 2);
                //     let mut blocks = min_buffer_size / Self::BLOCK_SIZE;
                //     if min_buffer_size % Self::BLOCK_SIZE > 0 {
                //         blocks += 1;
                //     }
                //     batch.reserve(blocks * Self::BLOCK_SIZE);
                // }
                let mut wakers = vec![];
                // let start = Instant::now();
                let mut batch = Vec::with_capacity(batch_size);
                loop {
                    match batch_receiver.try_recv() {
                        Ok(Message::Exit) => return,
                        Err(mpsc::error::TryRecvError::Empty) => break,
                        Ok(Message::Write { data, waker }) => {
                            wakers.push(waker);
                            batch.extend_from_slice(&data);
                            if batch.len() % Self::BLOCK_SIZE + 100 > Self::BLOCK_SIZE {
                                break;
                            }
                        }
                        _ => unreachable!("message sender cannot be dropped"),
                    }
                }

                // file.write_all(&batch).expect("write data failed");
                // let throughput = batch.len() as f64 / start.elapsed().as_secs_f64();
                // let errs = (throughput - last_throughput) / last_throughput;
                // if errs >= 0.1 {
                //     batch_size *= 2;
                // } else if errs <= -0.1 {
                //     batch_size = batch_size * 3 / 4;
                // }
                // last_throughput = throughput;

                let (io_waker, io_receiver) = oneshot::channel();
                io_sender_clone.send(Message::Write {
                    data: batch.into(),
                    waker: io_waker,
                }).expect("io sender cannot be dropped");

                runtime.spawn(async move {
                    io_receiver.await.expect("io receiver cannot be dropped");
                    for waker in wakers {
                        waker.send(()).expect("Fail to wake log writer")
                    }
                });
            }
        });

        Ok(Self {
            batch_sender,
            batch_worker: Some(worker_handler),
            io_sender,
            io_worker: Some(io_worker),
        })
    }

    pub async fn write_log(&self, data: Bytes) -> Result<()> {
        let (waker, receiver) = oneshot::channel();
        let _ = self.batch_sender.send(Message::Write {
            data,
            waker,
        });
        receiver
            .await
            .map_err(|e| anyhow!("Logger worker thread exit: {}", e))
    }

    pub fn shutdown(&mut self) {
        if let Some(handler) = self.batch_worker.take() {
            let _ = self.batch_sender.send(Message::Exit);
            handler.join().expect("Failed to join logger thread");
        }

        if let Some(handler) = self.io_worker.take() {
            let _ = self.io_sender.send(Message::Exit);
            handler.join().expect("Failed to join logger thread");
        }
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        self.shutdown();
    }
}
