use commons::{db::MonitorType, TcpMonitorConfiguration};

use crate::time_now;

use super::{MonitorReport, MonitorWorker, WorkTimeThresholds, WorkerError};

pub struct TcpMonitorWorker;
impl MonitorWorker for TcpMonitorWorker {
    type Config = TcpMonitorConfiguration;

    fn execute(
        _config: &TcpMonitorConfiguration,
        _thresholds: &WorkTimeThresholds,
    ) -> Result<MonitorReport, WorkerError> {
        Ok(MonitorReport {
            monitor_type: MonitorType::TCP,
            exec_time: time_now(),
        })
    }
}
