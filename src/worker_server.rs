use std::time::Duration;

use commons::message::{topic, MonitorMessage};
use futures::TryStreamExt;
use pulsar::{Consumer, DeserializeMessage, Pulsar, SubType, TokioExecutor};

use crate::{executor::WorkerExecutor, worker::*, WorkerInformation};

type PulsarConsumer<T> = Consumer<T, TokioExecutor>;

pub struct WorkerServer {
    executor: WorkerExecutor,
    msg_consumer: PulsarConsumer<MonitorMessage>,
}

impl WorkerServer {
    pub async fn new(executor: WorkerExecutor, uri: &str) -> Self {
        let pulsar = Pulsar::builder(uri, TokioExecutor)
            .build()
            .await
            .expect("Pulsar could not be created");

        let msg_consumer = consumer(
            &pulsar,
            "monitor",
            WorkerInformation::get().worker_id.as_str(),
        )
        .await;
        Self {
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
            tokio::time::sleep(Duration::from_secs(10)).await;
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

                self.executor
                    .create_worker(monitor_id, timer, thesholds, config)
            }
            MonitorMessage::Pause(pause_msg) => {
                let monitor_id = MonitorId(pause_msg.monitor_id);
                self.executor.pause_worker(&monitor_id);
            }
            MonitorMessage::Resume(resume_msg) => {
                let monitor_id = MonitorId(resume_msg.monitor_id);
                self.executor.resume_worker(&monitor_id);
            }
            MonitorMessage::Kill(kill_msg) => {
                let monitor_id = MonitorId(kill_msg.monitor_id);
                self.executor.kill_worker(&monitor_id);
            }
        }
    }
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
    };

    #[test]
    fn test_main() {
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

        executor.create_worker(id1.clone(), timer1, thresholds1, config1);
        println!("worker-1 created");
        thread::sleep(Duration::from_secs(15));

        executor.create_worker(id2.clone(), timer2, thresholds2, config2);
        println!("worker-2 created");
        thread::sleep(Duration::from_secs(15));

        executor.pause_worker(&id1);
        println!("worker-1 paused");
        thread::sleep(Duration::from_secs(30));

        executor.resume_worker(&id1);
        println!("worker-1 resumed");
        thread::sleep(Duration::from_secs(15));

        executor.kill_worker(&id2);
        println!("worker-2 killed");
        thread::sleep(Duration::from_secs(15));

        executor.kill_worker(&id1);
        println!("worker-1 killed");
        thread::sleep(Duration::from_secs(60));

        println!("-- THE END --");
    }
}
