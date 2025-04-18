use r2r::example_interfaces::msg::Float32;
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::logging::{LogEntry, EventType, Status, Sensors};

/// Handles the supersonic sensor data processing and publishing
/// 
/// - creates a ROS2 publisher for distance readings
/// - continuously reads from the sensor channel
/// - publishes distance data and handles obstacle detection
/// - packs data into JSON payload for MongoDB storage
pub async fn supersonic_process(
    node: Arc<Mutex<r2r::Node>>,
    mut supersonic_rx: mpsc::Receiver<f32>,
    logger: Arc<Mutex<crate::mongodb::MongoLogger>>,
    bot_id: String,
) {
    // lock the node and create a publisher for distance readings
    let mut node = node.lock().await;

    // create ros2 publisher
    let publisher = match node.create_publisher::<Float32>("supersonic_distance", r2r::QosProfile::default()) {
        Ok(publisher) => publisher,
        Err(e) => {
            eprintln!("Failed to create publisher: {}", e);
            return;
        }
    };
    
    // main processing loop
    loop {
        // try to receive new distance reading from the ros channel
        if let Ok(distance) = supersonic_rx.try_recv() {
            let mut msg = Float32::default();
            msg.data = distance;

            // publish the distance reading
            if let Err(e) = publisher.publish(&msg) {
                eprintln!("Failed to publish message: {}", e);
            }

            // obstacle detection - warn if object is too close
            // note: this was for testing purposes, doesn't need to be here..
            if distance < 0.3 { // 30cm threshold
                println!("Warning: Obstacle detected at {:.2}m", distance);
            }

            // Create log entry for the sensor reading
            let log_entry = LogEntry::new(
                bot_id.clone(),
                EventType::Info,
                "supersonic_reading".to_string(),
                Status::Success,
                format!("Supersonic sensor reading: {:.2}m", distance),
            )
            .with_sensors(Sensors {
                proximity: vec![distance as f64],
                temperature: None,
                light: None,
            });

            // Log the entry
            if let Err(e) = logger.lock().await.log_entry(log_entry).await {
                eprintln!("Failed to log supersonic reading: {}", e);
            }
        }
        else {
            println!("No supersonic sensor reading available!");
        }

        // go to sleep to control the processing rate
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
} 