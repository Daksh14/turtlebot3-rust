// // Camera logic
// mod camera;
// Navigation logic
mod nav;
// Graceful error handling
mod errors;
// yolo module
// mod yolo;
// lidar module
mod lidar;

use futures::{Stream, StreamExt};
use lidar::Direction;
use r2r::QosProfile;
use r2r::sensor_msgs::msg::LaserScan;
use r2r::{Node, Publisher};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};

// generate a node with a given name and namespace is set to turtlemove statically
pub fn generate_node(name: &str) -> r2r::Result<Node> {
    let name_space = "turtlemove";
    let ctx = r2r::Context::create()?;
    let node = r2r::Node::create(ctx, name, name_space)?;

    Ok(node)
}

// mutate a node in place to subscribe and publish to topic
pub fn create_pub_sub<T: r2r::WrappedTypesupport + 'static>(
    node: &mut r2r::Node,
    topic: &str,
) -> r2r::Result<(Publisher<T>, impl Stream<Item = T>)> {
    let subscriber = node.subscribe(topic, QosProfile::default())?;
    let publisher = node.create_publisher(topic, QosProfile::default())?;

    Ok((publisher, subscriber))
}

enum Sequence {
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
    let nav_node = Arc::new(Mutex::new(generate_node("nav_node")?));
    let nav_node_cl = Arc::clone(&nav_node);

    let lidar_node = Arc::new(Mutex::new(generate_node("lidar")?));
    let liadr_node_cl = Arc::clone(&lidar_node);

    // Launch the lidar communication channel, should be done with redis? I disagree
    let (tx, mut rx) = mpsc::channel::<LaserScan>(100);

    tokio::spawn({
        let mut lock = liadr_node_cl.lock().await;
        // subscribe to lidar node
        let qos = QosProfile::default().best_effort();
        let lidar_node_sub = lock.subscribe("/scan", qos).unwrap().boxed();

        lidar::lidar_scan(lidar_node_sub, tx)
    });

    tokio::spawn(async move {
        let nav_node_cl = Arc::clone(&nav_node_cl);
        // this is what the bot is doing at any point in time
        let mut current_sequence = Sequence::RandomMovement;

        loop {
            let cl = Arc::clone(&nav_node_cl);

            match current_sequence {
                Sequence::Intial360Rotation => {
                    nav::rotate360(cl).await;

                    sleep(Duration::from_secs(3)).await;

                    current_sequence = Sequence::RandomMovement;
                }
                Sequence::RandomMovement => {
                    let cl = Arc::clone(&cl);
                    let cl_2 = Arc::clone(&cl);

                    tokio::spawn(async move {
                        nav::nav_move(cl, 10.0, 0.0).await;
                    });

                    match rx.recv().await {
                        Some(scan) => {
                            if let Some(direction) = lidar::lidar_data(scan) {
                                println!("Detected direction: {:?}", direction);

                                if let Direction::North = direction {
                                    println!("{:?}", direction);

                                    nav::nav_stop(Arc::clone(&cl_2)).await;
                                    sleep(Duration::from_secs(3)).await;
                                    nav::nav_move(Arc::clone(&cl_2), -20.0, 0.0).await;
                                    nav::nav_move(cl_2, 20.0, -0.3).await;
                                }
                            }
                        }

                        None => {}
                    }
                }
                Sequence::TrackingToCharm => {
                    // TrackingToCharm
                }
                Sequence::SharmCollected => {
                    // SharmCollected
                }
                Sequence::Stop => {
                    nav::nav_stop(nav_node_cl).await;

                    println!("Stopping stop stop");

                    break;
                }
            }
        }
    });

    let node_spin_dur = std::time::Duration::from_millis(100);

    loop {
        nav_node.lock().await.spin_once(node_spin_dur);
        lidar_node.lock().await.spin_once(node_spin_dur);
    }
}
