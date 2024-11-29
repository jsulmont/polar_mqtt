use polar_mqtt::{Client, Message, QoS};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug)]
struct ReceivedMessage {
    timestamp: u64,
    message: Message,
}

fn preview_payload(payload: &[u8]) -> String {
    match String::from_utf8(payload.to_vec()) {
        Ok(s) => {
            let preview = s.chars().take(10).collect::<String>();
            if s.len() > 10 {
                format!("{}...", preview)
            } else {
                preview
            }
        }
        Err(_) => {
            let len = payload.len().min(10);
            format!("{:02X?}...", &payload[..len])
        }
    }
}

fn main() -> polar_mqtt::Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        println!("\nReceived Ctrl+C, shutting down...");
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler");

    let (tx, rx) = mpsc::channel();
    let client_id = format!("rust-client-{}", Uuid::new_v4());
    println!("Starting MQTT client with ID: {}", client_id);

    let mut client = Client::new(
        &client_id,
        {
            let tx = tx.clone();
            move |msg| {
                let received = ReceivedMessage {
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    message: msg.to_owned(),
                };

                if let Err(e) = tx.send(received) {
                    eprintln!("Failed to send message through channel: {}", e);
                }
            }
        },
        |state| println!("Connection state changed to: {:?}", state),
        |code, msg| eprintln!("Error occurred: {} - {}", code, msg),
    )?;

    println!("Connecting to test.mosquitto.org...");
    client.connect("test.mosquitto.org", 1883)?;

    let topic = "#";
    println!("Subscribing to {}", topic);
    let sub_handle = client.subscribe(topic, QoS::AtMostOnce)?;

    println!("Listening for messages. Press Ctrl+C to exit.");

    while running.load(Ordering::SeqCst) {
        match rx.recv_timeout(std::time::Duration::from_millis(100)) {
            Ok(received) => {
                println!("\nReceived at: {}", received.timestamp);
                println!("Topic: {}", received.message.topic());
                println!("Payload: {}", preview_payload(received.message.payload()));
                println!("QoS: {:?}", received.message.qos());
            }
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(e) => {
                eprintln!("Channel error: {}", e);
                break;
            }
        }
    }

    println!("Unsubscribing...");
    client.unsubscribe(sub_handle)?;
    println!("Exiting.");

    Ok(())
}
