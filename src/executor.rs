use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use crate::worker::{MonitorWorker, WorkTimer, WorkerHandle, WorkerId, self, MonitorReport, WorkerError};

pub struct WorkerExecutor {
    workers: HashMap<WorkerId, WorkerHandle>,
}

impl WorkerExecutor {
    pub fn new() -> Self {
        Self {
            workers: HashMap::new(),
        }
    }

    pub fn spawn_worker<TWorker: MonitorWorker>(
        &mut self,
        id: WorkerId,
        timer: WorkTimer,
        config: TWorker::Config,
    ) {
        let running_arc = Arc::new(AtomicBool::new(true));

        let worker_id = id.clone();
        let running = running_arc.clone();
        let join = thread::spawn(move || {
            let worker_id = worker_id;
            let mut timer = timer;
            let config = config;

            while running.load(Ordering::Relaxed) {
                if timer.should_run() {
                    timer.save_execution();

                    match TWorker::execute(&config) {
                        Ok(report) => save_report(&worker_id, report),
                        Err(error) => handle_error(&worker_id, error),
                    }
                }

                thread::sleep(Duration::from_millis(500));
            }
        });

        let handle = WorkerHandle {
            join,
            running: running_arc.clone(),
        };
        self.workers.insert(id, handle);
    }

    pub fn kill_worker(&mut self, id: &WorkerId) {
        let worker_handle = self.workers.remove(id);
        if let Some(worker_handle) = worker_handle {
            worker_handle.kill();
        }
    }

    pub async fn run_once(&self) {}
}

fn save_report(worker_id: &WorkerId, report: MonitorReport) {
    println!("worker: {:?}, report: {:?}", worker_id, report);
}

fn handle_error(worker_id: &WorkerId, error: WorkerError) {
    println!("worker: {:?}, error: {:?}", worker_id, error);
}
