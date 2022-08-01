use std::sync::Arc;
use std::time::{Duration, Instant};

use bytes::Bytes;
use bytesize::ByteSize;
use rand::RngCore;

use crate::Logger;

#[tokio::test]
async fn test_write_data() -> anyhow::Result<()> {
    test_throughput_and_latency(1).await?;
    test_throughput_and_latency(10).await?;
    test_throughput_and_latency(100).await?;
    test_throughput_and_latency(1000).await?;
    test_throughput_and_latency(10_000).await?;
    test_throughput_and_latency(100_000).await?;
    test_throughput_and_latency(1000_000).await?;
    test_throughput_and_latency(10_000_000).await?;
    Ok(())
}

async fn test_throughput_and_latency(task_size: usize) -> anyhow::Result<()> {
    const MSG_SIZE: usize = 100;
    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(1000)
        .build()?;

    let logger = Arc::new(Logger::open("test.log", None, None)?);
    let mut rng = rand::thread_rng();
    let mut data = vec![0; MSG_SIZE];

    let mut tasks = Vec::with_capacity(task_size);
    rng.fill_bytes(&mut data);

    let bytes = Bytes::from(data);
    for _ in 0..tasks.capacity() {
        let data_ref = bytes.clone();
        let logger_ref = logger.clone();
        tasks.push(async move {
            let start = Instant::now();
            logger_ref.write_log(data_ref).await?;
            Ok::<_, anyhow::Error>(start.elapsed())
        });
    }

    let mut task_groups = Vec::new();
    for _ in 0..256 {
        task_groups.push(tasks.drain(..task_size / 256).collect::<Vec<_>>());
    }

    let start = Instant::now();
    let results = task_groups
        .into_iter()
        .map(|group| {
            std::thread::spawn(move || {
                futures::executor::block_on(futures::future::join_all(group))
            })
            .join()
        })
        .flatten()
        .flatten()
        .collect::<anyhow::Result<Vec<_>>>();

    if let Ok(report) = guard.report().build() {
        let file = std::fs::File::create("flamegraph.svg")?;
        report.flamegraph(file)?;
    };

    let total_cost = start.elapsed();
    let avg_latency = results?.iter().sum::<Duration>() / task_size as u32;
    println!(
        "write {} in {:?}, avg latency: {:?}",
        ByteSize::b((task_size * MSG_SIZE) as u64),
        total_cost,
        avg_latency,
    );
    Ok(())
}
