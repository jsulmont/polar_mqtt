use polar_mqtt::{Client, QoS};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

fn main() -> polar_mqtt::Result<()> {
    println!("Initializing MQTT topic monitor...");

    let client_id = format!("RustMonitor_{}", Uuid::new_v4());
    println!("Client ID: {}", client_id);

    let (state_tx, state_rx) = mpsc::channel();

    // Topic statistics: message count and data in bytes
    let topic_stats = Arc::new(Mutex::new(HashMap::new()));
    let topic_stats_clone = Arc::clone(&topic_stats);
    let shutdown_flag = Arc::new(AtomicBool::new(false));

    let mut client = Client::new(
        &client_id,
        move |msg| {
            let topic = msg.topic().to_string();
            let payload_size = msg.payload().len();

            let mut stats = topic_stats_clone.lock().unwrap();
            let entry = stats.entry(topic).or_insert((0, 0));
            entry.0 += 1; // Increment message count
            entry.1 += payload_size; // Add payload size
        },
        move |state| {
            let _ = state_tx.send(state);
        },
        move |code, msg| {
            println!("Error occurred: {} ({})", code, msg);
        },
    )?;

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

    let display_thread = thread::spawn({
        let topic_stats = Arc::clone(&topic_stats);
        let shutdown_flag = Arc::clone(&shutdown_flag);
        move || {
            let mut last_check = Instant::now();
            loop {
                if shutdown_flag.load(Ordering::SeqCst) {
                    break;
                }

                thread::sleep(Duration::from_secs(2));
                let now = Instant::now();
                let elapsed_secs = now.duration_since(last_check).as_secs_f64();
                last_check = now;

                let mut stats = topic_stats.lock().unwrap();

                // Calculate message rate and data rate for each topic
                let mut rate_stats: Vec<_> = stats
                    .iter()
                    .map(|(topic, &(msg_count, data_size))| {
                        (
                            topic.clone(),
                            msg_count as f64 / elapsed_secs,
                            (data_size as f64 / 1024.0) / elapsed_secs, // Data rate in KiB/s
                        )
                    })
                    .collect();

                // Sort by message rate first, then by data rate
                rate_stats.sort_by(|a, b| {
                    b.1.partial_cmp(&a.1)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal))
                });

                // Print top 10 topics
                println!("Top 10 Topics by Message Rate and Data Rate:");
                println!("{:<50} {:>10} {:>10}", "Topic", "Msg/s", "KiB/s");
                println!("──────────────────────────────────────────────────────────────────────");
                for (topic, msg_rate, data_rate) in rate_stats.iter().take(10) {
                    println!("{:<50} {:>10.2} {:>10.2}", topic, msg_rate, data_rate);
                }
                println!();

                // Reset stats for the next interval
                stats.clear();
            }
        }
    });

    ctrlc::set_handler({
        let shutdown_flag = Arc::clone(&shutdown_flag);
        move || {
            shutdown_flag.store(true, Ordering::SeqCst);
        }
    })
    .expect("Error setting Ctrl-C handler");

    while !shutdown_flag.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(100));
    }

    println!("\nReceived shutdown signal, cleaning up...");
    client.unsubscribe(sub_handle)?;
    println!("Unsubscribed. Exiting.");

    display_thread.join().expect("Failed to join thread");

    Ok(())
}
