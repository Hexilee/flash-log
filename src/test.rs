use std::time::Instant;

use crate::Logger;

use bytesize::ByteSize;
use rand::RngCore;

#[tokio::test]
async fn test_large_throughput() -> anyhow::Result<()> {
    let logger = Logger::open("test.log", None).unwrap();
    let mut rng = rand::thread_rng();
    let mut data = vec![0; 100];
    let task_size = 10_000_000;

    let mut tasks = Vec::with_capacity(task_size);
    rng.fill_bytes(&mut data);
    for _ in 0..tasks.capacity() {
        tasks.push(async {
            let start = Instant::now();
            logger.write_log(&data).await?;
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
        .map(|d| d.as_millis())
        .sum::<u128>()
        / task_size as u128;
    println!(
        "write {}bytes/s, avg latency: {}ms",
        ByteSize::b(task_size as u64 * 100 / total_cost.as_secs()),
        avg_latency
    );
    Ok(())
}
