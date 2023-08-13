use commons::{
    message::{topic, MonitorMessage},
    MonitorConfiguration,
};
use futures::TryStreamExt;
use pulsar::{Consumer, DeserializeMessage, Pulsar, SubType, TokioExecutor};

use crate::{executor::WorkerExecutor, worker::*, CacheEntry, WorkerCache, WorkerInformation};

type PulsarConsumer<T> = Consumer<T, TokioExecutor>;

pub struct WorkerServer {
    cache: WorkerCache,
    executor: WorkerExecutor,
    msg_consumer: PulsarConsumer<MonitorMessage>,
}

impl WorkerServer {
    pub async fn new(cache: WorkerCache, executor: WorkerExecutor, uri: &str) -> Self {
        let pulsar = Pulsar::builder(uri, TokioExecutor)
            .build()
            .await
            .expect("Pulsar could not be created");

        let msg_consumer =
            consumer(&pulsar, "msg", WorkerInformation::get().worker_id.as_str()).await;
        Self {
            cache,
            executor,
            msg_consumer,
        }
    }

    pub async fn run(&mut self) {
        loop {
            let msg = self.msg_consumer.try_next().await;
            match msg {
                Ok(msg) => match msg {
                    Some(msg) => {
                        if let Err(err_ack) = self.msg_consumer.ack(&msg).await {
                            println!("Error on ack: {:?}", err_ack);
                        }

                        let data = match msg.deserialize() {
                            Ok(data) => Some(data),
                            Err(err_ds) => {
                                println!("Error on desialize: {:?}", err_ds);
                                None
                            }
                        };

                        if let Some(msg) = data {
                            self.handle_msg(msg).await;
                        }
                    }
                    None => println!("No message"),
                },
                Err(err1) => {
                    println!("Error on try_next: {:?}", err1);
                }
            }
        }
    }

    async fn handle_msg(&mut self, msg: MonitorMessage) {
        match msg {
            MonitorMessage::Create(create_msg) => {
                let monitor_id = MonitorId(create_msg.monitor_id);
                let timer = WorkTimer::new(create_msg.timing.repeat_secs);
                let thesholds = WorkTimeThresholds {
                    healthy: create_msg.timing.healthy,
                    timeout: create_msg.timing.timeout,
                };
                let config = create_msg.monitor;

                create_worker(
                    &mut self.executor,
                    &mut self.cache,
                    monitor_id,
                    timer,
                    thesholds,
                    config,
                )
            }
            MonitorMessage::Pause(pause_msg) => {
                let monitor_id = MonitorId(pause_msg.monitor_id);
                pause_worker(&mut self.executor, &monitor_id);
            }
            MonitorMessage::Resume(resume_msg) => {
                let monitor_id = MonitorId(resume_msg.monitor_id);
                resume_worker(&mut self.executor, &self.cache, &monitor_id);
            }
            MonitorMessage::Kill(kill_msg) => {
                let monitor_id = MonitorId(kill_msg.monitor_id);
                kill_worker(&mut self.executor, &mut self.cache, &monitor_id);
            }
        }
    }
}

fn spawn_worker(
    executor: &mut WorkerExecutor,
    id: MonitorId,
    timer: WorkTimer,
    thresholds: WorkTimeThresholds,
    config: MonitorConfiguration,
) {
    match config {
        MonitorConfiguration::Ping(config) => {
            executor.spawn_worker::<PingMonitorWorker>(id, timer, thresholds, config)
        }
        MonitorConfiguration::Http(config) => {
            executor.spawn_worker::<HttpMonitorWorker>(id, timer, thresholds, config)
        }
        MonitorConfiguration::Https(config) => {
            executor.spawn_worker::<HttpsMonitorWorker>(id, timer, thresholds, config)
        }
        MonitorConfiguration::Tcp(config) => {
            executor.spawn_worker::<TcpMonitorWorker>(id, timer, thresholds, config)
        }
        MonitorConfiguration::Udp(config) => {
            executor.spawn_worker::<UdpMonitorWorker>(id, timer, thresholds, config)
        }
    }
}

fn create_worker(
    executor: &mut WorkerExecutor,
    cache: &mut WorkerCache,
    id: MonitorId,
    timer: WorkTimer,
    thresholds: WorkTimeThresholds,
    config: MonitorConfiguration,
) {
    cache.entries.insert(
        id.clone(),
        CacheEntry {
            timer: timer.clone(),
            thresholds: thresholds.clone(),
            config: config.clone(),
        },
    );
    spawn_worker(executor, id, timer, thresholds, config);
}

fn pause_worker(executor: &mut WorkerExecutor, id: &MonitorId) {
    executor.kill_worker(id);
}

fn resume_worker(executor: &mut WorkerExecutor, cache: &WorkerCache, id: &MonitorId) {
    let Some(work_entry) = cache.entries.get(id) else {
        return;
    };
    spawn_worker(
        executor,
        id.clone(),
        work_entry.timer.clone(),
        work_entry.thresholds.clone(),
        work_entry.config.clone(),
    );
}

fn kill_worker(executor: &mut WorkerExecutor, cache: &mut WorkerCache, id: &MonitorId) {
    cache.entries.remove(id);
    executor.kill_worker(id);
}

async fn consumer<T: DeserializeMessage>(
    pulsar: &Pulsar<TokioExecutor>,
    act: &str,
    worker_id: &str,
) -> PulsarConsumer<T> {
    pulsar
        .consumer()
        .with_topic(topic(act, worker_id))
        .with_consumer_name(format!("consumer-{}", worker_id))
        .with_subscription_type(SubType::Exclusive)
        .with_subscription(format!("subscription-{}", worker_id))
        .build()
        .await
        .expect(&format!("Consumer [{act}] could not be created"))
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use commons::MonitorConfiguration;

    use crate::{
        executor::WorkerExecutor,
        worker::{MonitorId, WorkTimeThresholds, WorkTimer},
        worker_server::{create_worker, kill_worker, pause_worker, resume_worker},
        WorkerCache,
    };

    #[test]
    fn test_main() {
        let mut cache = WorkerCache::new();
        let mut executor = WorkerExecutor::new();

        let id1 = MonitorId("worker-1".to_string());
        let timer1 = WorkTimer::new(5);
        let thresholds1 = WorkTimeThresholds {
            healthy: 5,
            timeout: 10,
        };
        let config1 = MonitorConfiguration::Ping(commons::PingMonitorConfiguration {
            host: "test".to_string(),
        });

        let id2 = MonitorId("worker-2".to_string());
        let timer2 = WorkTimer::new(10);
        let thresholds2 = WorkTimeThresholds {
            healthy: 5,
            timeout: 10,
        };
        let config2 = MonitorConfiguration::Tcp(commons::TcpMonitorConfiguration {
            host: "test".to_string(),
            port: 4000,
            send: None,
            receive: None,
        });

        create_worker(
            &mut executor,
            &mut cache,
            id1.clone(),
            timer1,
            thresholds1,
            config1,
        );
        println!("worker-1 created");
        thread::sleep(Duration::from_secs(15));

        create_worker(
            &mut executor,
            &mut cache,
            id2.clone(),
            timer2,
            thresholds2,
            config2,
        );
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
