use commons::{db::MonitorType, UdpMonitorConfiguration};

use crate::time_now;

use super::{MonitorReport, MonitorWorker, WorkTimeThresholds, WorkerError};

pub struct UdpMonitorWorker;
impl MonitorWorker for UdpMonitorWorker {
    type Config = UdpMonitorConfiguration;

    fn execute(
        _config: &UdpMonitorConfiguration,
        _thresholds: &WorkTimeThresholds,
    ) -> Result<MonitorReport, WorkerError> {
        Ok(MonitorReport {
            monitor_type: MonitorType::UDP,
            exec_time: time_now(),
        })
    }
}
