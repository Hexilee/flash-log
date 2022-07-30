use std::io::Write;
use std::{fs::OpenOptions, path::Path, thread::JoinHandle, time::Instant};

use anyhow::{anyhow, Result};
use crossbeam::channel::{bounded, Sender, TryRecvError};

enum Message {
    Exit,
    Write { data: Vec<u8>, waker: Sender<()> },
}

pub struct Logger {
    sender: Sender<Message>,
    worker_handler: Option<JoinHandle<()>>,
}

impl Logger {
    const DEFAULT_MAX_BUFFER: usize = 512 * 1024 * 1024;
    const DEFAULT_AVG_MSG_SIZE: usize = 100;
    const START_BATCH_SIZE: usize = 4096;

    pub fn open(
        path: impl AsRef<Path>,
        max_buffer_o: Option<usize>,
        avg_msg_size_o: Option<usize>,
    ) -> Result<Self> {
        let mut file = OpenOptions::new().append(true).create(true).open(path)?;
        let max_buffer = max_buffer_o.unwrap_or(Self::DEFAULT_MAX_BUFFER);
        let avg_msg_size = avg_msg_size_o.unwrap_or(Self::DEFAULT_AVG_MSG_SIZE);
        let (sender, receiver) = bounded(max_buffer / avg_msg_size);

        let worker_handler = std::thread::spawn(move || {
            let mut maxThroughput = 0.;
            let mut batchSize = Self::START_BATCH_SIZE;
            loop {
                let mut wakers = vec![];
                let start = Instant::now();
                let mut writen = 0;

                loop {
                    match receiver.try_recv() {
                        Ok(Message::Exit) => return (),
                        Err(TryRecvError::Empty) => break,
                        Ok(Message::Write { data, waker }) => {
                            wakers.push(waker);
                            file.write_all(&data).expect("write data failed");
                            writen += data.len();
                            if writen > batchSize {
                                break;
                            }
                        }
                        _ => unreachable!("message sender cannot be dropped"),
                    }
                }
                file.sync_data().expect("sync data failed");
                let throughput = writen as f64 / start.elapsed().as_secs_f64();
                if throughput >= maxThroughput {
                    maxThroughput = throughput;
                    batchSize *= 2;
                } else {
                    batchSize = batchSize * 3 / 4;
                }
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

    pub fn write_log(&self, data: &[u8]) -> Result<()> {
        let (waker, receiver) = bounded(1);
        let _ = self.sender.send(Message::Write {
            data: data.to_vec(),
            waker,
        });
        receiver
            .recv()
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
