use crate::Logger;

use rand::RngCore;

#[tokio::test]
async fn test_single_threads_low_throughput() {
    let mut logger = Logger::open("test.log", None, None).unwrap();
    let mut rng = rand::thread_rng();
    let mut data = vec![0; 100];
    rng.fill_bytes(&mut data);
    // for _ in 0..10000 {
    //     logger.write_log(&data).unwrap();
    // }
    // let mut file = File::open("test.log").unwrap();
    // let mut buf = vec![];
    // file.read_to_end(&mut buf).unwrap();
    // assert_eq!(buf.len(), 1024 * 100);
}
