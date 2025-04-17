/// Camera logic
mod camera;
/// Graceful error handling
mod error;
/// lidar module
mod lidar;
/// Navigation logic
mod nav;
/// Publisher module
mod publisher;
/// yolo module
mod yolo;

use async_cell::sync::AsyncCell;
use r2r::QosProfile;
use r2r::sensor_msgs::msg::LaserScan;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

type Num = f32;
pub type XyXy = (Num, Num, Num, Num);
pub type Node = Arc<Mutex<r2r::Node>>;

// generate a node with a given name and namespace is set to turtlemove statically
pub fn generate_node(name: &str) -> r2r::Result<Node> {
    let name_space = "turtlemove";
    let ctx = r2r::Context::create()?;
    let node = r2r::Node::create(ctx, name, name_space)?;
    let mutex = Mutex::new(node);
    let arc = Arc::new(mutex);

    Ok(arc)
}

/// What the bot is doing at any point in time
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
    let node_spin_dur = std::time::Duration::from_millis(10);

    // setup all the nodes
    let nav_node = generate_node("nav_node")?;
    let lidar_node = generate_node("lidar_node")?;

    // Launch the lidar communication channel
    let cell = AsyncCell::shared();
    let weak = cell.take_weak();
    let (yolo_tx, yolo_rx) = mpsc::channel::<XyXy>(100);

    // camera process + yolo detect
    std::thread::spawn(move || {
        let cam = camera::cam_plus_yolo_detect(yolo_tx);

        println!("{:?}", cam);
    });

    let lidar_cl = Arc::clone(&lidar_node);
    // lidar process
    tokio::spawn(async move {
        let mut lidar_node_sub = {
            let mut lock = lidar_cl.lock().expect("failed to lock lidar node");
            // subscribe to lidar node
            let qos = QosProfile::default().best_effort();

            lock.subscribe("/scan", qos)
                .expect("Subscribing to lidar should work")
        };

        lidar::lidar_scan(&mut lidar_node_sub, cell).await;
    });

    let cl = Arc::clone(&nav_node);
    // navigation process
    //     tokio::spawn(async move {
    //         // this is what the bot is doing at any point in time
    //         let start_sequence = Sequence::RandomMovement;
    //
    //         nav::move_process(start_sequence, cl, weak, yolo_rx).await
    //     });

    loop {
        if let Ok(mut nav_handle) = nav_node.lock() {
            nav_handle.spin_once(node_spin_dur);
        }

        if let Ok(mut lidar_handle) = lidar_node.lock() {
            lidar_handle.spin_once(node_spin_dur);
        }
    }
}
