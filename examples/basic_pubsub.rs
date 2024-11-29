use polar_mqtt::{Client, Message, QoS};
use std::sync::mpsc;
use std::time::Duration;

fn main() -> polar_mqtt::Result<()> {
    println!("Initializing API...");

    // Create channel for message communication
    let (tx, rx) = mpsc::channel();

    // Create channel for connection state
    let (state_tx, state_rx) = mpsc::channel();

    // Generate test topic with random suffix to avoid interference
    let test_topic = format!("test/topic/{}", std::process::id());
    let test_topic_clone = test_topic.clone();

    // Create client with callbacks
    let mut client = Client::new(
        &format!("TestClient_{}", uuid::Uuid::new_v4()),
        move |msg| {
            println!("\nReceived message in callback:");
            println!("  Topic: {}", msg.topic());
            println!("  Payload length: {}", msg.payload().len());
            println!("  Payload (hex): {:02X?}", msg.payload());
            println!(
                "  Payload (utf8): {}",
                String::from_utf8_lossy(msg.payload())
            );
            println!("  QoS: {:?}", msg.qos());
            println!("  Retained: {}", msg.is_retained());

            if msg.topic() == test_topic_clone {
                let message = Message::new(msg.topic(), msg.payload().to_vec())
                    .with_qos(msg.qos())
                    .with_retain(msg.is_retained());
                if let Err(e) = tx.send(message) {
                    println!("Error sending message through channel: {}", e);
                }
            }
        },
        move |state| {
            println!("\nConnection state changed to: {:?}", state);
            if let Err(e) = state_tx.send(state) {
                println!("Error sending state through channel: {}", e);
            }
        },
        |code, err| {
            println!("\nError occurred in callback:");
            println!("  Code: {}", code);
            println!("  Message: {}", err);
        },
    )?;

    println!("Initial connection state: {:?}", client.state());
    println!("Connecting to broker...");
    client.connect("broker.emqx.io", 1883)?;

    // Wait for connected state
    let timeout = Duration::from_secs(5);
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        match state_rx.try_recv() {
            Ok(state) => {
                println!("Connection state update: {:?}", state);
                if let polar_mqtt::ConnectionState::Connected = state {
                    break;
                }
            }
            Err(mpsc::TryRecvError::Empty) => {
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                println!("Error receiving state: {}", e);
                break;
            }
        }
    }

    println!("Current connection state: {:?}", client.state());

    println!("Subscribing to topic: {}", test_topic);
    let sub_handle = client.subscribe(&test_topic, QoS::AtLeastOnce)?;
    println!("Subscription handle: {}", sub_handle);

    // Wait for subscription to establish
    std::thread::sleep(Duration::from_secs(1));

    // Prepare test message with random data and timestamp
    let test_payload = format!(
        "{{\"test\":true,\"id\":{},\"time\":{}}}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );

    println!("\nPublishing test message...");
    println!("Topic: {}", test_topic);
    println!("Payload: {}", test_payload);
    println!("Payload bytes: {:02X?}", test_payload.as_bytes());

    let msg = Message::new(&test_topic, test_payload.clone())
        .with_qos(QoS::AtLeastOnce)
        .with_retain(false);

    let msg_id = client.publish(&msg)?;
    println!("Published message id: {}", msg_id);

    // Wait for message with timeout
    println!("\nWaiting for message...");
    match rx.recv_timeout(Duration::from_secs(5)) {
        Ok(msg) => {
            let received_payload = String::from_utf8_lossy(msg.payload());
            println!("\nReceived message in main:");
            println!("  Topic: {}", msg.topic());
            println!("  Payload length: {}", msg.payload().len());
            println!("  Payload (hex): {:02X?}", msg.payload());
            println!("  Payload (utf8): {}", received_payload);
            println!("  QoS: {:?}", msg.qos());
            println!("  Retained: {}", msg.is_retained());

            if msg.topic() == test_topic && received_payload == test_payload {
                println!("\nSuccess: Message payloads match!");
            } else {
                println!("\nError: Message mismatch");
                println!("Expected topic: {}", test_topic);
                println!("Received topic: {}", msg.topic());
                println!("Expected payload: {}", test_payload);
                println!("Received payload: {}", received_payload);
                println!("\nPayload comparison:");
                println!("Expected length: {}", test_payload.len());
                println!("Received length: {}", msg.payload().len());
                println!("Expected bytes: {:02X?}", test_payload.as_bytes());
                println!("Received bytes: {:02X?}", msg.payload());
            }
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            println!("Error: Timeout waiting for message");
            println!("Current connection state: {:?}", client.state());
        }
        Err(e) => {
            println!("Error receiving message: {}", e);
            println!("Current connection state: {:?}", client.state());
        }
    }

    // Clean up
    println!("\nCleaning up...");
    client.unsubscribe(sub_handle)?;
    println!("Unsubscribed successfully");
    println!("Final connection state: {:?}", client.state());

    Ok(())
}
