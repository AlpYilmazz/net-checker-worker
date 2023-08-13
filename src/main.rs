use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use commons::MonitorConfiguration;
use executor::WorkerExecutor;
use once_cell::sync::OnceCell;
use serde::Deserialize;
use worker::{MonitorId, WorkTimeThresholds, WorkTimer};
use worker_server::WorkerServer;

pub mod executor;
pub mod worker;
pub mod worker_server;

static WORKER_INFO: OnceCell<WorkerInformation> = OnceCell::new();

#[derive(Deserialize)]
pub struct WorkerInformation {
    pub worker_id: String,
}

impl WorkerInformation {
    pub fn initialize(/*path: impl AsRef<Path>*/) {
        // let Ok(file) = File::open(path) else {
        //     panic!("Config file not found.")
        // };
        // let reader = BufReader::new(file);
        // let config = ron::de::from_reader(reader).expect("Config could not be deserialized.");
        // let _ = PROGRAM_CONFIG.set(config);
        let worker_info = WorkerInformation {
            worker_id: "worker-1".to_string(),
        };
        let _ = WORKER_INFO.set(worker_info);
    }

    pub fn get() -> &'static Self {
        WORKER_INFO
            .get()
            .expect("Program config is not initialized")
    }
}

pub struct CacheEntry {
    pub timer: WorkTimer,
    pub thresholds: WorkTimeThresholds,
    pub config: MonitorConfiguration,
}

pub struct WorkerCache {
    pub entries: HashMap<MonitorId, CacheEntry>,
}

impl WorkerCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
}

fn time_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[tokio::main]
async fn main() {
    let pulsar_connection_string = "pulsar://127.0.0.1:6650";

    let cache = WorkerCache::new();
    let executor = WorkerExecutor::new();
    let mut server =
        WorkerServer::new(cache, executor, pulsar_connection_string).await;

    server.run().await;
}
