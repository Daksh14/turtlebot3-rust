// Camera logic
mod camera;
// Navigation logic
mod nav;
// Graceful error handling
mod errors;
// yolo module
mod yolo;
// lidar module
mod lidar;

use r2r::sensor_msgs::msg::LaserScan;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

// generate a node with a given name and namespace is set to turtlemove statically
pub fn generate_node(name: &str) -> r2r::Result<r2r::Node> {
    let name_space = "turtlemove";
    let ctx = r2r::Context::create()?;
    let node = r2r::Node::create(ctx, name, name_space)?;

    Ok(node)
}

pub enum Sequence {
    Intial360Rotation,
    // Start randomly moving in x, y direction
    RandomMovement,
    // If charm is located, start moving towards it
    TrackingToCharm,
    // Charm is collected
    SharmCollected,
    // Stop
    Stop,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // duration of each node spin
    let node_spin_dur = std::time::Duration::from_millis(100);

    // setup all the nodes
    let nav_node = Arc::new(Mutex::new(generate_node("nav_node")?));
    let nav_node_cl = Arc::clone(&nav_node);

    let lidar_node = Arc::new(Mutex::new(generate_node("lidar")?));
    let liadr_node_cl = Arc::clone(&lidar_node);

    // Launch the lidar communication channel, should be done with redis? I disagree
    let (lidar_tx, lidar_rx) = mpsc::channel::<LaserScan>(100);

    // lidar process
    tokio::spawn(lidar::lidar_scan(liadr_node_cl, lidar_tx));

    // camera process + yolo detect
    std::thread::spawn(move || { camera::cam_plus_yolo_detect() });
    // just yolo detect
    // tokio::spawn(camera::yolo_detect_test());

    // navigation process
    tokio::spawn(async move {
        let nav_node_cl = Arc::clone(&nav_node_cl);
        // this is what the bot is doing at any point in time
        let start_sequence = Sequence::RandomMovement;

        nav::move_process(start_sequence, nav_node_cl, lidar_rx).await
    });

    loop {
        nav_node.lock().await.spin_once(node_spin_dur);
        lidar_node.lock().await.spin_once(node_spin_dur);
    }
}
