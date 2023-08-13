use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use crate::worker::{
    MonitorId, MonitorReport, MonitorWorker, WorkTimeThresholds, WorkTimer, WorkerError,
    WorkerHandle,
};

pub struct WorkerExecutor {
    workers: HashMap<MonitorId, WorkerHandle>,
}

impl WorkerExecutor {
    pub fn new() -> Self {
        Self {
            workers: HashMap::new(),
        }
    }

    pub fn spawn_worker<TWorker: MonitorWorker>(
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

    pub fn kill_worker(&mut self, id: &MonitorId) {
        let worker_handle = self.workers.remove(id);
        if let Some(worker_handle) = worker_handle {
            worker_handle.kill();
        }
    }

    pub async fn run_once(&self) {}
}

fn save_report(worker_id: &MonitorId, report: MonitorReport) {
    println!("worker: {:?}, report: {:?}", worker_id, report);
}

fn handle_error(worker_id: &MonitorId, error: WorkerError) {
    println!("worker: {:?}, error: {:?}", worker_id, error);
}
