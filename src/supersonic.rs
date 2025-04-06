use r2r::example_interfaces::msg::Float32;
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn supersonic_process(
    node: Arc<Mutex<r2r::Node>>,
    mut supersonic_rx: mpsc::Receiver<f32>,
) {
    let mut node = node.lock().await;
    let publisher = match node.create_publisher::<Float32>("supersonic_distance", r2r::QosProfile::default()) {
        Ok(publisher) => publisher,
        Err(e) => {
            eprintln!("Failed to create publisher: {}", e);
            return;
        }
    };
    
    loop {
        // read from the supersonic sensor
        if let Ok(distance) = supersonic_rx.try_recv() {
            let mut msg = Float32::default();

            msg.data = distance;
            if let Err(e) = publisher.publish(&msg) {
                eprintln!("Failed to publish message: {}", e);
            }

            // basic check, change later
            if distance < 0.3 { // 30cm 
                println!("Warning: Obstacle detected at {:.2}m", distance);
            }
        }
        else {
            println!("No supersonic sensor reading available!");
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
} 