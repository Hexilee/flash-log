use std::sync::Arc;
use std::time::{Duration, Instant};

use bytes::Bytes;
use bytesize::ByteSize;
use futures::future::join_all;
use rand::RngCore;

use crate::Logger;

#[test]
fn test_write_data() -> anyhow::Result<()> {
    test_throughput_and_latency(1024, true)?; // warm up

    test_throughput_and_latency(1, false)?;
    test_throughput_and_latency(10, false)?;
    test_throughput_and_latency(100, false)?;
    test_throughput_and_latency(1000, false)?;
    test_throughput_and_latency(10_000, false)?;
    test_throughput_and_latency(100_000, false)?;
    test_throughput_and_latency(1000_000, false)?;
    Ok(())
}

fn test_throughput_and_latency(task_size: usize, silent: bool) -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    const MSG_SIZE: usize = 1000;
    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(1000)
        .build()?;

    let logger = Arc::new(Logger::open("test.log", None, None)?);
    let mut rng = rand::thread_rng();
    let mut data = vec![0; MSG_SIZE];

    let mut tasks = Vec::with_capacity(task_size);
    rng.fill_bytes(&mut data);

    let bytes = Bytes::from(data);

    // construct tasks
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

    let mut task_groups = Vec::new();
    let writers = num_cpus::get() - 2;
    for _ in 0..writers {
        // schedule tasks in multiple threads
        task_groups.push(tasks.drain(..task_size / writers).collect::<Vec<_>>());
    }
    task_groups.push(tasks);

    // execute all tasks
    let ret = rt.block_on(join_all(
        task_groups
            .into_iter()
            .map(|group| rt.spawn(futures::future::join_all(group))),
    ));

    let total_cost = start.elapsed();

    // collect the latency of each log message
    let mut results = ret
        .into_iter()
        .flatten()
        .flatten()
        .collect::<anyhow::Result<Vec<_>>>()?;

    if let Ok(report) = guard.report().build() {
        let file = std::fs::File::create("flamegraph.svg")?;
        report.flamegraph(file)?;
    };

    results.sort();
    assert_eq!(task_size, results.len());
    let avg_latency = results.iter().sum::<Duration>() / results.len() as u32;

    if !silent {
        println!(
            "write {} in {:?}, avg latency: {:?}. 50%({:?}), 90%({:?}), 95%({:?}), 99%({:?})",
            ByteSize::b((task_size * MSG_SIZE) as u64),
            total_cost,
            avg_latency,
            results[results.len() / 2],
            results[results.len() * 9 / 10],
            results[results.len() * 95 / 100],
            results[results.len() * 99 / 100],
        );
    }
    Ok(())
}
