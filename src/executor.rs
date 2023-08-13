use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use commons::MonitorConfiguration;

use crate::worker::{
    HttpMonitorWorker, HttpsMonitorWorker, MonitorId, MonitorReport, MonitorWorker,
    PingMonitorWorker, TcpMonitorWorker, UdpMonitorWorker, WorkTimeThresholds, WorkTimer,
    WorkerError, WorkerHandle,
};

pub struct CacheEntry {
    pub timer: WorkTimer,
    pub thresholds: WorkTimeThresholds,
    pub config: MonitorConfiguration,
}

pub struct WorkerExecutor {
    cache: HashMap<MonitorId, CacheEntry>,
    workers: HashMap<MonitorId, WorkerHandle>,
}

impl WorkerExecutor {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            workers: HashMap::new(),
        }
    }

    fn spawn_worker_inner<TWorker: MonitorWorker>(
        &mut self,
        id: MonitorId,
        timer: WorkTimer,
        thresholds: WorkTimeThresholds,
        config: TWorker::Config,
    ) {
        let running_arc = Arc::new(AtomicBool::new(true));

        let monitor_id = id.clone();
        let running = running_arc.clone();
        let join = thread::spawn(move || {
            let monitor_id = monitor_id;
            let mut timer = timer;
            let thresholds = thresholds;
            let config = config;

            while running.load(Ordering::Relaxed) {
                if timer.should_run() {
                    timer.save_execution();

                    match TWorker::execute(&config, &thresholds) {
                        Ok(report) => save_report(&monitor_id, report),
                        Err(error) => handle_error(&monitor_id, error),
                    }
                }

                // TODO: Maybe no should_run, but sleep period_secs
                thread::sleep(Duration::from_millis(500));
            }
        });

        let handle = WorkerHandle {
            join,
            running: running_arc.clone(),
        };
        self.workers.insert(id, handle);
    }

    fn spawn_worker(
        &mut self,
        id: MonitorId,
        timer: WorkTimer,
        thresholds: WorkTimeThresholds,
        config: MonitorConfiguration,
    ) {
        match config {
            MonitorConfiguration::Ping(config) => {
                self.spawn_worker_inner::<PingMonitorWorker>(id, timer, thresholds, config)
            }
            MonitorConfiguration::Http(config) => {
                self.spawn_worker_inner::<HttpMonitorWorker>(id, timer, thresholds, config)
            }
            MonitorConfiguration::Https(config) => {
                self.spawn_worker_inner::<HttpsMonitorWorker>(id, timer, thresholds, config)
            }
            MonitorConfiguration::Tcp(config) => {
                self.spawn_worker_inner::<TcpMonitorWorker>(id, timer, thresholds, config)
            }
            MonitorConfiguration::Udp(config) => {
                self.spawn_worker_inner::<UdpMonitorWorker>(id, timer, thresholds, config)
            }
        }
    }

    pub fn create_worker(
        &mut self,
        id: MonitorId,
        timer: WorkTimer,
        thresholds: WorkTimeThresholds,
        config: MonitorConfiguration,
    ) {
        self.cache.insert(
            id.clone(),
            CacheEntry {
                timer: timer.clone(),
                thresholds: thresholds.clone(),
                config: config.clone(),
            },
        );
        self.spawn_worker(id, timer, thresholds, config);
    }

    pub fn pause_worker(&mut self, id: &MonitorId) {
        self.kill_worker(id);
    }

    pub fn resume_worker(&mut self, id: &MonitorId) {
        let Some(work_entry) = self.cache.get(id) else {
            return;
        };
        self.spawn_worker(
            id.clone(),
            work_entry.timer.clone(),
            work_entry.thresholds.clone(),
            work_entry.config.clone(),
        );
    }

    pub fn kill_worker(&mut self, id: &MonitorId) {
        self.cache.remove(id);
        let worker_handle = self.workers.remove(id);
        if let Some(worker_handle) = worker_handle {
            worker_handle.kill();
        }
    }
}

fn save_report(worker_id: &MonitorId, report: MonitorReport) {
    println!("worker: {:?}, report: {:?}", worker_id, report);
}

fn handle_error(worker_id: &MonitorId, error: WorkerError) {
    println!("worker: {:?}, error: {:?}", worker_id, error);
}
