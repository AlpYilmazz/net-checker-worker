use commons::{db::MonitorType, HttpMonitorConfiguration, HttpsMonitorConfiguration};

use crate::time_now;

use super::{MonitorReport, MonitorWorker, WorkTimeThresholds, WorkerError};

pub struct HttpMonitorWorker;
impl MonitorWorker for HttpMonitorWorker {
    type Config = HttpMonitorConfiguration;

    fn execute(
        _config: &HttpMonitorConfiguration,
        _thresholds: &WorkTimeThresholds,
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
        _config: &HttpsMonitorConfiguration,
        _thresholds: &WorkTimeThresholds,
    ) -> Result<MonitorReport, WorkerError> {
        Ok(MonitorReport {
            monitor_type: MonitorType::HTTPS,
            exec_time: time_now(),
        })
    }
}
