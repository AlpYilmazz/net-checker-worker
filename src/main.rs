use std::{collections::HashMap, thread, time::Duration};

use commons::MonitorConfiguration;
use executor::WorkerExecutor;
use worker::{
    HttpMonitorWorker, HttpsMonitorWorker, PingMonitorWorker, TcpMonitorWorker, UdpMonitorWorker,
    WorkTimer, WorkerId,
};

pub mod executor;
pub mod worker;

pub struct WorkerCache {
    pub configs: HashMap<WorkerId, (WorkTimer, MonitorConfiguration)>,
}

impl WorkerCache {
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
        }
    }
}

fn spawn_worker(
    executor: &mut WorkerExecutor,
    id: WorkerId,
    timer: WorkTimer,
    config: MonitorConfiguration,
) {
    match config {
        MonitorConfiguration::Ping(config) => {
            executor.spawn_worker::<PingMonitorWorker>(id, timer, config)
        }
        MonitorConfiguration::Http(config) => {
            executor.spawn_worker::<HttpMonitorWorker>(id, timer, config)
        }
        MonitorConfiguration::Https(config) => {
            executor.spawn_worker::<HttpsMonitorWorker>(id, timer, config)
        }
        MonitorConfiguration::Tcp(config) => {
            executor.spawn_worker::<TcpMonitorWorker>(id, timer, config)
        }
        MonitorConfiguration::Udp(config) => {
            executor.spawn_worker::<UdpMonitorWorker>(id, timer, config)
        }
    }
}

fn create_worker(
    executor: &mut WorkerExecutor,
    cache: &mut WorkerCache,
    id: &WorkerId,
    timer: &WorkTimer,
    config: &MonitorConfiguration,
) {
    cache
        .configs
        .insert(id.clone(), (timer.clone(), config.clone()));
    spawn_worker(executor, id.clone(), timer.clone(), config.clone());
}

fn pause_worker(executor: &mut WorkerExecutor, id: &WorkerId) {
    executor.kill_worker(id);
}

fn resume_worker(executor: &mut WorkerExecutor, cache: &WorkerCache, id: &WorkerId) {
    let Some((timer, config)) = cache.configs.get(id) else {
        return;
    };
    spawn_worker(executor, id.clone(), timer.clone(), config.clone());
}

fn kill_worker(executor: &mut WorkerExecutor, cache: &mut WorkerCache, id: &WorkerId) {
    cache.configs.remove(id);
    executor.kill_worker(id);
}

fn main() {
    println!("Main");
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use commons::MonitorConfiguration;

    use crate::{
        create_worker,
        executor::WorkerExecutor,
        kill_worker, pause_worker, resume_worker,
        worker::{WorkTimer, WorkerId},
        WorkerCache,
    };

    #[test]
    fn test_main() {
        let mut cache = WorkerCache::new();
        let mut executor = WorkerExecutor::new();

        let id1 = WorkerId("worker-1".to_string());
        let timer1 = WorkTimer::new(5);
        let config1 = MonitorConfiguration::Ping(commons::PingMonitorConfiguration {
            host: "test".to_string(),
        });

        let id2 = WorkerId("worker-2".to_string());
        let timer2 = WorkTimer::new(10);
        let config2 = MonitorConfiguration::Tcp(commons::TcpMonitorConfiguration {
            host: "test".to_string(),
            port: 4000,
            send: None,
            receive: None,
        });

        create_worker(&mut executor, &mut cache, &id1, &timer1, &config1);
        println!("worker-1 created");
        thread::sleep(Duration::from_secs(15));

        create_worker(&mut executor, &mut cache, &id2, &timer2, &config2);
        println!("worker-2 created");
        thread::sleep(Duration::from_secs(15));

        pause_worker(&mut executor, &id1);
        println!("worker-1 paused");
        thread::sleep(Duration::from_secs(30));

        resume_worker(&mut executor, &cache, &id1);
        println!("worker-1 resumed");
        thread::sleep(Duration::from_secs(15));

        kill_worker(&mut executor, &mut cache, &id2);
        println!("worker-2 killed");
        thread::sleep(Duration::from_secs(15));

        kill_worker(&mut executor, &mut cache, &id1);
        println!("worker-1 killed");
        thread::sleep(Duration::from_secs(60));

        println!("-- THE END --");
    }
}
