use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::JoinHandle,
};

use commons::db::MonitorType;

use crate::time_now;

pub mod http;
pub mod ping;
pub mod tcp;
pub mod udp;

#[derive(Debug)]
pub struct MonitorReport {
    pub monitor_type: MonitorType,
    pub exec_time: u64,
}

#[derive(Debug)]
pub struct WorkerError {}

#[derive(Clone)]
pub struct WorkTimer {
    pub period_secs: u32,
    last_execution: u64,
}

impl WorkTimer {
    pub fn new(period_secs: u32) -> Self {
        Self {
            period_secs,
            last_execution: time_now(),
        }
    }

    pub fn should_run(&self) -> bool {
        let now = time_now();
        (now - self.last_execution) >= self.period_secs as u64
    }

    pub fn save_execution(&mut self) {
        self.last_execution = time_now();
    }
}

#[derive(Clone)]
pub struct WorkTimeThresholds {
    pub healthy: u32,
    pub timeout: u32,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct MonitorId(pub String);

pub trait MonitorWorker {
    type Config: Send + Sync + 'static;

    fn execute(
        config: &Self::Config,
        thresholds: &WorkTimeThresholds,
    ) -> Result<MonitorReport, WorkerError>;
}

pub struct WorkerHandle {
    pub join: JoinHandle<()>,
    pub running: Arc<AtomicBool>,
}

impl WorkerHandle {
    pub fn kill(self) {
        self.running.store(false, Ordering::Relaxed);
        let _thread_result = self.join.join();
    }
}
