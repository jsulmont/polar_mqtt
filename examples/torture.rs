use polar_mqtt::{self, Client, Message, QoS};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let mut threads = vec![];

    // Publisher thread
    let running_pub = Arc::clone(&running);
    threads.push(thread::spawn(move || {
        println!("Starting publisher");
        let mut client = Client::new(
            "debug-pub",
            |_| {},
            |state| println!("Publisher state: {:?}", state),
            |code, err| eprintln!("Publisher error: {} - {}", code, err),
        )
        .unwrap();

        println!("Publisher connecting");
        client.connect("test.mosquitto.org", 1883).unwrap();
        thread::sleep(Duration::from_secs(1));

        while running_pub.load(Ordering::Relaxed) {
            let msg = Message::new("test/debug", "test").with_qos(QoS::AtLeastOnce);
            println!("Publishing message");
            let _ = client.publish(&msg);
            thread::sleep(Duration::from_millis(100));
        }
    }));

    // Subscriber thread
    let running_sub = Arc::clone(&running);
    threads.push(thread::spawn(move || {
        println!("Starting subscriber");
        let mut client = Client::new(
            "debug-sub",
            |msg| println!("Received: {:?}", msg.topic()),
            |state| println!("Subscriber state: {:?}", state),
            |code, err| eprintln!("Subscriber error: {} - {}", code, err),
        )
        .unwrap();

        println!("Subscriber connecting");
        client.connect("test.mosquitto.org", 1883).unwrap();
        thread::sleep(Duration::from_secs(1));

        println!("Subscribing");
        if let Ok(handle) = client.subscribe("test/#", QoS::AtLeastOnce) {
            while running_sub.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(100));
            }
            println!("Unsubscribing");
            let _ = client.unsubscribe(handle);
        }
    }));

    thread::sleep(Duration::from_secs(30));
    running.store(false, Ordering::Relaxed);

    for t in threads {
        let _ = t.join();
    }
}
