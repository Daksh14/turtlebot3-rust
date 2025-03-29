// // Camera logic
// mod camera;
// Navigation logic
mod nav;
// Graceful error handling
mod errors;

use futures_core::Stream;
use r2r::std_msgs::msg::String as StringMessage;
use r2r::{Context, Node, Publisher};
use r2r::{
    QosProfile,
    geometry_msgs::msg::{Twist, Vector3},
};
use std::sync::Arc;
use tokio::time::{Duration, sleep};

// generate a node with a given name and namespace is set to turtlemove statically
fn generate_node(name: &str) -> r2r::Result<(Context, Node)> {
    let name_space = "turtlemove";
    let ctx = r2r::Context::create()?;
    let mut node = r2r::Node::create(ctx, name, "turtlemove")?;

    Ok((node, ctx))
}

// mutate a node in place to subscribe and publish to topic
fn create_pub_sub<T>(
    node: &mut r2r::Node,
    topic: &str,
) -> r2r::Result<(Publisher<T>, impl Stream<Item = T>)> {
    let subscriber = node.subscribe(topic, QosProfile::default())?;
    let publisher = node.create_publisher(topic, QosProfile::default())?;

    Ok((publisher, subscriber))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let node_name = "";

    match generate_node("nav_node") {
        Ok((ctx, node)) => {
            let mut node = node;
            let (publisher, subscriber) = create_pub_sub::<Twist>(&mut node, "/cmd_vel");

            let publlisher = Arc::new(publisher);

            // Run the publisher in another task
            tokio::spawn(async move {
                // Create a loop to publish velocity commands
                loop {
                    let mut twist = Twist {
                        linear: r2r::geometry_msgs::msg::Vector3 {
                            x: 0.5,
                            y: 0.0,
                            z: 0.0,
                        }, // Move forward
                        angular: r2r::geometry_msgs::msg::Vector3 {
                            x: 0.0,
                            y: 0.0,
                            z: 0.3,
                        }, // Rotate slightly
                    };

                    // Publish the message
                    match publisher.publish(&twist) {
                        Ok(_) => println!(
                            "Published: linear = {}, angular = {}",
                            twist.linear.x, twist.angular.z
                        ),
                        Err(e) => eprintln!("Failed to publish: {}", e),
                    }

                    // Sleep for 1 second between messages
                    sleep(Duration::from_secs(1)).await;
                }
            })?;
        }
        Err(err) => panic!("Couldn't generate node? {}", err),
    }

    // Main loop spins ros.
    loop {
        node.spin_once(std::time::Duration::from_millis(100));
    }
}
