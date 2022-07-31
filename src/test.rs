use std::time::{Duration, Instant};
use std::sync::Arc;

use bytes::Bytes;
use bytesize::ByteSize;
use rand::RngCore;

use crate::Logger;

#[tokio::test]
async fn test_write_data() -> anyhow::Result<()> {
    let guard = pprof::ProfilerGuardBuilder::default().frequency(1000).build()?;
    test_throughput_and_latency(1).await?;
    test_throughput_and_latency(10).await?;
    test_throughput_and_latency(100).await?;
    test_throughput_and_latency(1000).await?;
    test_throughput_and_latency(10_000).await?;
    test_throughput_and_latency(100_000).await?;
    test_throughput_and_latency(1000_000).await?;
    test_throughput_and_latency(10_000_000).await?;
    if let Ok(report) = guard.report().build() {
        let file = std::fs::File::create("flamegraph.svg")?;
        report.flamegraph(file)?;
    };
    Ok(())
}

async fn test_throughput_and_latency(task_size: usize) -> anyhow::Result<()> {
    const MSG_SIZE: usize = 100;
    let logger = Arc::new(Logger::open("test.log", None).unwrap());
    let mut rng = rand::thread_rng();
    let mut data = vec![0; MSG_SIZE];

    let mut tasks = Vec::with_capacity(task_size);
    rng.fill_bytes(&mut data);
    let data_ref = Bytes::from(data);
    for _ in 0..tasks.capacity() {
        let data_ref_clone = data_ref.clone();
        let logger_clone = logger.clone();
        tasks.push(async move {
            let start = Instant::now();
            logger_clone.write_log(data_ref_clone).await?;
            Ok::<_, anyhow::Error>(start.elapsed())
        });
    }

    let start = Instant::now();
    let results = futures::future::join_all(tasks).await;

    let total_cost = start.elapsed();
    let avg_latency = results
        .into_iter()
        .collect::<anyhow::Result<Vec<_>>>()?
        .iter()
        // .map(|d| d.as_millis())
        .sum::<Duration>()
        / task_size as u32;
    println!(
        "write {} in {:?}, avg latency: {:?}",
        ByteSize::b((task_size * MSG_SIZE) as u64),
        total_cost,
        avg_latency,
    );
    Ok(())
}
