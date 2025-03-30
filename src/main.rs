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

use futures::Stream;
use r2r::QosProfile;
use r2r::{Node, Publisher};
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut nav_node = generate_node("nav_node")?;
    let mut lidar_node = generate_node("lidar")?;
    // subscribe to lidar node
    let lidar_node_sub = lidar_node.subscribe("/scan", QosProfile::default())?;
    let publisher = nav_node.create_publisher("/cmd_vel", QosProfile::default())?;
    // 1. First instruction as the binary is run to move the bot 10 units x direction
    // and 5 units y direction
    nav::nav_move(&publisher, 10.0, 5.0).await;

    lidar::lidar_scan(&lidar_node, lidar_node_sub);

    // this is what the bot is doing at any point in time
    let mut current_sequence = Sequence::Intial360Rotation;
    let node_spin_dur = std::time::Duration::from_millis(100);

    loop {
        match current_sequence {
            Sequence::Intial360Rotation => {
                nav::rotate360(&publisher);

                sleep(Duration::from_secs(3)).await;
            }
            Sequence::RandomMovement => {
                // RandomMovement
            }
            Sequence::TrackingToCharm => {
                // TrackingToCharm
            }
            Sequence::SharmCollected => {
                // SharmCollected
            }
            _ => {
                // Default case
            }
        }

        nav_node.spin_once(node_spin_dur);
        lidar_node.spin_once(node_spin_dur);
    }
}
