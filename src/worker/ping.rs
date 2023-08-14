use commons::{db::MonitorType, PingMonitorConfiguration};

use crate::time_now;

use super::{MonitorReport, MonitorWorker, WorkTimeThresholds, WorkerError};

pub struct PingMonitorWorker;
impl MonitorWorker for PingMonitorWorker {
    type Config = PingMonitorConfiguration;

    fn execute(
        _config: &PingMonitorConfiguration,
        _thresholds: &WorkTimeThresholds,
    ) -> Result<MonitorReport, WorkerError> {
        Ok(MonitorReport {
            monitor_type: MonitorType::PING,
            exec_time: time_now(),
        })
    }
}
