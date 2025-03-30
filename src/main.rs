// // Camera logic
// mod camera;
// Navigation logic
mod nav;
// Graceful error handling
mod errors;

mod yolo;

use std::thread::current;

use futures_core::Stream;
use r2r::QosProfile;
use r2r::{Node, Publisher};

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
    let mut node = generate_node("nav_node")?;
    let publisher = node.create_publisher("/cmd_vel", QosProfile::default())?;

    let handle = tokio::task::spawn_blocking(move || {
        let current_sequence = Sequence::Intial360Rotation;

        loop {
            node.spin_once(std::time::Duration::from_millis(100));
        }
    });

    tokio::task::spawn(async move || {
        loop {
            match current_sequence {
                Sequence::Intial360Rotation => {}
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
        }
    });

    nav::nav_move(&publisher, 2.0, 3.0).await;

    handle.await?;

    Ok(())
}
