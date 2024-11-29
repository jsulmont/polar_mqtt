use polar_mqtt::{self, Client, ConnectionState, Error as MqttError, Message, QoS};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::RecvTimeoutError,
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};
use uuid::Uuid;

#[derive(Debug)]
enum AppError {
    Mqtt(MqttError),
    ThreadJoin(String),
}

impl std::error::Error for AppError {}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Mqtt(e) => write!(f, "MQTT error: {}", e),
            AppError::ThreadJoin(e) => write!(f, "Thread error: {}", e),
        }
    }
}

impl From<MqttError> for AppError {
    fn from(err: MqttError) -> Self {
        AppError::Mqtt(err)
    }
}

#[derive(Default)]
struct TopicStats {
    message_count: u64,
    total_bytes: u64,
    last_message_time: Option<Instant>,
}

fn main() -> Result<(), AppError> {
    let stats = Arc::new(Mutex::new(HashMap::<String, TopicStats>::new()));
    let client_id = format!("rust-multi-client-{}", Uuid::new_v4());
    let should_exit = Arc::new(AtomicBool::new(false));

    let (state_tx, state_rx) = std::sync::mpsc::channel();
    let (error_tx, error_rx) = std::sync::mpsc::channel();
    let stats_clone = Arc::clone(&stats);
    let error_tx = Arc::new(Mutex::new(error_tx));

    let mut client = Client::new(
        &client_id,
        {
            let stats = Arc::clone(&stats);
            move |msg| {
                let mut stats = stats.lock().unwrap();
                let topic_stats = stats.entry(msg.topic().to_string()).or_default();
                topic_stats.message_count += 1;
                topic_stats.total_bytes += msg.payload().len() as u64;
                topic_stats.last_message_time = Some(Instant::now());

                println!("\nReceived message on {}", msg.topic());
                println!("Payload size: {} bytes", msg.payload().len());
                if let Ok(payload) = String::from_utf8(msg.payload().to_vec()) {
                    if payload.len() < 100 {
                        println!("Content: {}", payload);
                    }
                }
            }
        },
        move |state| {
            println!("Connection state: {:?}", state);
            let _ = state_tx.send(state);
        },
        move |code, msg| {
            if let Ok(tx) = error_tx.lock() {
                let _ = tx.send((code, msg.to_string()));
            }
        },
    )?;

    let error_handler = thread::spawn({
        let should_exit = Arc::clone(&should_exit);
        move || {
            while !should_exit.load(Ordering::Relaxed) {
                match error_rx.recv_timeout(Duration::from_millis(100)) {
                    Ok((code, msg)) => {
                        println!("MQTT Error occurred:");
                        println!("  Code: {}", code);
                        println!("  Message: {}", msg);

                        match code {
                            -1 => println!("  Action: Connection lost, will retry"),
                            -2 => println!("  Action: Protocol error"),
                            _ => println!("  Action: Unhandled error"),
                        }
                    }
                    Err(RecvTimeoutError::Timeout) => continue,
                    Err(_) => break,
                }
            }
        }
    });

    println!("Connecting to broker...");
    client.connect("test.mosquitto.org", 1883)?;

    while let Ok(state) = state_rx.recv_timeout(Duration::from_secs(5)) {
        if state == ConnectionState::Connected {
            break;
        }
    }

    let subscriptions = [
        ("test/sensors/#", QoS::AtLeastOnce),
        ("test/control/#", QoS::ExactlyOnce),
        ("test/status/#", QoS::AtMostOnce),
    ];

    let sub_handles: Vec<_> = subscriptions
        .iter()
        .filter_map(|(topic, qos)| match client.subscribe(topic, *qos) {
            Ok(handle) => {
                println!("Subscribed to {} with handle {}", topic, handle);
                Some(handle)
            }
            Err(e) => {
                eprintln!("Failed to subscribe to {}: {}", topic, e);
                None
            }
        })
        .collect();

    let publisher_thread = thread::spawn({
        let client_id = format!("rust-publisher-{}", Uuid::new_v4());
        move || -> Result<(), AppError> {
            let mut publisher = Client::new(
                &client_id,
                |_| {},
                |state| println!("Publisher state: {:?}", state),
                |code, msg| println!("Publisher error: {} - {}", code, msg),
            )?;

            publisher.connect("test.mosquitto.org", 1883)?;
            thread::sleep(Duration::from_secs(1));

            for i in 0..10 {
                let messages = vec![
                    Message::new(
                        "test/sensors/temperature",
                        format!("{{\"value\": {}, \"unit\": \"C\"}}", 20 + (i % 5)),
                    ),
                    Message::new(
                        "test/control/pump",
                        format!("{{\"state\": {}}}", i % 2 == 0),
                    ),
                    Message::new(
                        "test/status/system",
                        format!(
                            "{{\"uptime\": {}, \"load\": {:.1}}}",
                            i * 60,
                            i as f32 * 0.1
                        ),
                    ),
                ];

                for msg in messages {
                    let msg_with_qos = msg.with_qos(QoS::AtLeastOnce);
                    if let Err(e) = publisher.publish(&msg_with_qos) {
                        eprintln!("Failed to publish to {}: {}", msg_with_qos.topic(), e);
                    }
                    thread::sleep(Duration::from_millis(500));
                }
            }
            Ok(())
        }
    });

    let stats_thread = thread::spawn(move || {
        let last_print = Instant::now();
        while last_print.elapsed() < Duration::from_secs(30) {
            thread::sleep(Duration::from_secs(2));

            let stats = stats_clone.lock().unwrap();
            println!("\nTopic Statistics:");
            println!("{:-<60}", "");
            for (topic, stats) in stats.iter() {
                let age = stats
                    .last_message_time
                    .map(|t| t.elapsed().as_secs())
                    .unwrap_or(0);

                println!(
                    "{}\n  Messages: {}, Bytes: {}, Last msg: {}s ago",
                    topic, stats.message_count, stats.total_bytes, age
                );
            }
        }
    });

    publisher_thread
        .join()
        .map_err(|_| AppError::ThreadJoin("Publisher thread panicked".to_string()))?
        .map_err(|e| AppError::ThreadJoin(e.to_string()))?;

    stats_thread
        .join()
        .map_err(|_| AppError::ThreadJoin("Stats thread panicked".to_string()))?;

    error_handler
        .join()
        .map_err(|_| AppError::ThreadJoin("Error handler thread panicked".to_string()))?;

    for handle in sub_handles {
        if let Err(e) = client.unsubscribe(handle) {
            eprintln!("Failed to unsubscribe handle {}: {}", handle, e);
        }
    }

    Ok(())
}
