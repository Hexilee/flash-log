use std::sync::Arc;
use std::time::{Duration, Instant};

use bytes::Bytes;
use bytesize::ByteSize;
use futures::future::join_all;
use rand::RngCore;

use crate::Logger;

#[test]
fn test_write_data() -> anyhow::Result<()> {
    test_throughput_and_latency(1)?;
    test_throughput_and_latency(10)?;
    test_throughput_and_latency(100)?;
    test_throughput_and_latency(1000)?;
    test_throughput_and_latency(10_000)?;
    test_throughput_and_latency(100_000)?;
    test_throughput_and_latency(1000_000)?;
    test_throughput_and_latency(10_000_000)?;
    Ok(())
}

fn test_throughput_and_latency(task_size: usize) -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
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

    let start = Instant::now();

    let (results, total_cost) = if task_size > 10000 {
        let mut task_groups = Vec::new();
        for _ in 0..16 {
            task_groups.push(tasks.drain(..task_size / 16).collect::<Vec<_>>());
        }
        task_groups.push(tasks);

        let ret = rt.block_on(join_all(
            task_groups
                .into_iter()
                .map(|group| rt.spawn(futures::future::join_all(group))),
        ));
        let elapsed = start.elapsed();
        (
            ret.into_iter()
                .flatten()
                .flatten()
                .collect::<anyhow::Result<Vec<_>>>()?,
            elapsed,
        )
    } else {
        let ret = rt.block_on(futures::future::join_all(tasks));
        let elapsed = start.elapsed();
        (
            ret.into_iter().collect::<anyhow::Result<Vec<_>>>()?,
            elapsed,
        )
    };

    if let Ok(report) = guard.report().build() {
        let file = std::fs::File::create("flamegraph.svg")?;
        report.flamegraph(file)?;
    };

    assert_eq!(task_size, results.len());
    let avg_latency = results.iter().sum::<Duration>() / results.len() as u32;
    println!(
        "write {} in {:?}, avg latency: {:?}",
        ByteSize::b((task_size * MSG_SIZE) as u64),
        total_cost,
        avg_latency,
    );
    Ok(())
}
