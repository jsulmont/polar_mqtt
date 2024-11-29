use polar_mqtt::{Client, QoS};
use std::sync::mpsc;
use std::time::Duration;
use uuid::Uuid;

fn main() -> polar_mqtt::Result<()> {
    println!("Initializing MQTT monitor...");

    let (stop_tx, stop_rx) = mpsc::channel();

    let client_id = format!("RustMonitor_{}", Uuid::new_v4());
    println!("Client ID: {}", client_id);

    let (state_tx, state_rx) = mpsc::channel();
    let (error_tx, error_rx) = mpsc::channel();

    let mut client = Client::new(
        &client_id,
        move |msg| {
            let payload = msg.payload();
            let preview =
                if let Ok(s) = String::from_utf8(payload[..payload.len().min(20)].to_vec()) {
                    if payload.len() > 20 {
                        format!("{}...", s)
                    } else {
                        s
                    }
                } else {
                    format!("{:02X?}", &payload[..payload.len().min(10)])
                };
            println!(
                "Topic: {:<50} | Length: {:>4} | Preview: {}",
                msg.topic(),
                payload.len(),
                preview
            );
        },
        move |state| {
            let _ = state_tx.send(state);
        },
        move |code, msg| {
            let _ = error_tx.send((code, msg.to_string()));
        },
    )?;

    let error_handler = std::thread::spawn(move || {
        while let Ok((code, msg)) = error_rx.recv() {
            if stop_rx.try_recv().is_ok() {
                break;
            }

            println!("MQTT Error {}: {}", code, msg);
            match code {
                -1 => println!("Connection lost, will automatically reconnect"),
                -2 => println!("Protocol violation"),
                _ => println!("Unexpected error"),
            }
        }
    });

    println!("Connecting to test.mosquitto.org...");
    client.connect("test.mosquitto.org", 1883)?;

    match state_rx.recv_timeout(Duration::from_secs(5)) {
        Ok(state) => println!("Connection state: {:?}", state),
        Err(_) => {
            println!("Timeout waiting for connection");
            return Ok(());
        }
    }

    println!("Subscribing to all topics (#)...");
    let sub_handle = client.subscribe("#", QoS::AtMostOnce)?;
    println!("Subscribed successfully");

    println!("\nMonitoring messages (Press Ctrl+C to stop)...");
    println!("────────────────────────────────────────────────────────────────────────────────");

    let (tx, rx) = mpsc::channel();
    ctrlc::set_handler(move || {
        let _ = tx.send(());
    })
    .expect("Error setting Ctrl-C handler");

    let _ = rx.recv();

    println!("\nReceived Ctrl-C, cleaning up...");
    let _ = stop_tx.send(());
    client.unsubscribe(sub_handle)?;
    println!("Unsubscribed. Exiting.");

    if let Err(e) = error_handler.join() {
        eprintln!("Error handler thread panicked: {:?}", e);
    }

    Ok(())
}
