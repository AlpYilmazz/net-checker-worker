use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::JoinHandle,
};

use commons::{
    db::MonitorType, HttpMonitorConfiguration, HttpsMonitorConfiguration, PingMonitorConfiguration,
    TcpMonitorConfiguration, UdpMonitorConfiguration,
};

use crate::time_now;

#[derive(Debug)]
pub struct MonitorReport {
    monitor_type: MonitorType,
    exec_time: u64,
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

pub struct PingMonitorWorker;
impl MonitorWorker for PingMonitorWorker {
    type Config = PingMonitorConfiguration;

    fn execute(
        config: &PingMonitorConfiguration,
        thresholds: &WorkTimeThresholds,
    ) -> Result<MonitorReport, WorkerError> {
        Ok(MonitorReport {
            monitor_type: MonitorType::PING,
            exec_time: time_now(),
        })
    }
}

pub struct HttpMonitorWorker;
impl MonitorWorker for HttpMonitorWorker {
    type Config = HttpMonitorConfiguration;

    fn execute(
        config: &HttpMonitorConfiguration,
        thresholds: &WorkTimeThresholds,
    ) -> Result<MonitorReport, WorkerError> {
        Ok(MonitorReport {
            monitor_type: MonitorType::HTTP,
            exec_time: time_now(),
        })
    }
}

pub struct HttpsMonitorWorker;
impl MonitorWorker for HttpsMonitorWorker {
    type Config = HttpsMonitorConfiguration;

    fn execute(
        config: &HttpsMonitorConfiguration,
        thresholds: &WorkTimeThresholds,
    ) -> Result<MonitorReport, WorkerError> {
        Ok(MonitorReport {
            monitor_type: MonitorType::HTTPS,
            exec_time: time_now(),
        })
    }
}

pub struct TcpMonitorWorker;
impl MonitorWorker for TcpMonitorWorker {
    type Config = TcpMonitorConfiguration;

    fn execute(
        config: &TcpMonitorConfiguration,
        thresholds: &WorkTimeThresholds,
    ) -> Result<MonitorReport, WorkerError> {
        Ok(MonitorReport {
            monitor_type: MonitorType::TCP,
            exec_time: time_now(),
        })
    }
}

pub struct UdpMonitorWorker;
impl MonitorWorker for UdpMonitorWorker {
    type Config = UdpMonitorConfiguration;

    fn execute(
        config: &UdpMonitorConfiguration,
        thresholds: &WorkTimeThresholds,
    ) -> Result<MonitorReport, WorkerError> {
        Ok(MonitorReport {
            monitor_type: MonitorType::UDP,
            exec_time: time_now(),
        })
    }
}
