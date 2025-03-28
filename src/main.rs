use r2r::{geometry_msgs::msg::Twist, QosProfile};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the ROS 2 context
    r2r::init()?;

    // Create a ROS 2 node
    let node = r2r::Node::create("cmd_vel_publisher", "")?;

    // Create a publisher for the /cmd_vel topic
    let publisher = node.create_publisher::<Twist>("/cmd_vel", QosProfile::default())?;

    println!("Publishing to /cmd_vel...");

    // Create a loop to publish velocity commands
    loop {
        let mut twist = Twist {
            linear: r2r::geometry_msgs::msg::Vector3 { x: 0.5, y: 0.0, z: 0.0 },   // Move forward
            angular: r2r::geometry_msgs::msg::Vector3 { x: 0.0, y: 0.0, z: 0.3 }, // Rotate slightly
        };

        // Publish the message
        match publisher.publish(&twist) {
            Ok(_) => println!("Published: linear = {}, angular = {}", twist.linear.x, twist.angular.z),
            Err(e) => eprintln!("Failed to publish: {}", e),
        }

        // Sleep for 1 second between messages
        sleep(Duration::from_secs(1)).await;
    }
}
