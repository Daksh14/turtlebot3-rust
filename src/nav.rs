use r2r::sensor_msgs::msg::LaserScan;
use r2r::{
    Node, Publisher, QosProfile,
    geometry_msgs::msg::{Twist, Vector3},
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Receiver;
use tokio::time::{Duration, sleep};
use std::sync::mpsc::Receiver as BlockingRecv;

use crate::{Sequence, YoloResult};
use crate::lidar::{self, Direction};
use usls::Bbox;

type NavNode = Arc<Mutex<Node>>;

// main navigation logic
pub async fn move_process(
    // sequence to start the nav move from
    starting_seq: Sequence,
    nav_node: NavNode,
    mut lidar_rx: Receiver<LaserScan>,
    yolo_rx: BlockingRecv<YoloResult>,
) {
    let mut current_sequence = starting_seq;

    // detect at least 3 instances of the object to confirm its existence

    loop {
        let cl = Arc::clone(&nav_node);

        match current_sequence {
            Sequence::Intial360Rotation => {
                rotate360(cl).await;

                sleep(Duration::from_secs(3)).await;

                current_sequence = Sequence::RandomMovement;
            }
            Sequence::RandomMovement => {
                let cl = Arc::clone(&cl);
                let cl_2 = Arc::clone(&cl);

                tokio::spawn(async move {
                    nav_move(cl, 0.5, 0.0).await;
                });

                match lidar_rx.recv().await {
                    Some(scan) => {
                        if let Some(direction) = lidar::lidar_data(scan) {
                            println!("Detected direction: {:?}", direction);

                            match direction {
                                Direction::North => {
                                    nav_stop(Arc::clone(&cl_2)).await;
                                    nav_move(cl_2, 0.2, 0.3).await;
                                }
                                Direction::NorthWest => {
                                    nav_stop(Arc::clone(&cl_2)).await;
                                    nav_move(cl_2, 0.2, 0.3).await;
                                }
                                Direction::NorthEast => {
                                    nav_stop(Arc::clone(&cl_2)).await;
                                    nav_move(cl_2, 0.2, -0.3).await;
                                }
                                _ => (),
                            }
                        }
                    }
                    None => {}
                }
            }
            Sequence::TrackingToCharm => {
                let cl = Arc::clone(&cl);

                match yolo_rx.recv() {
                    Ok(bboxes) => {
                        for bbox in bboxes {
                            let cl_2 = Arc::clone(&cl);
                            let (x1, y1, x2, y2) = bbox.xyxy();

                            let scaled =  scale_0_to_200(x1);

                            rotate(cl_2, scaled as f64).await;
                        }
                    }
                    Err(_) => {}
                }
            }
            Sequence::SharmCollected => {
                // SharmCollected
            }
            Sequence::Stop => {
                nav_stop(cl).await;
                println!("Stopping stop stop");

                break;
            }
        }
    }
}

pub async fn get_pub(node: NavNode) -> Publisher<Twist> {
    let mut lock = node.lock().await;

    let publisher = lock
        .create_publisher::<Twist>("/cmd_vel", QosProfile::default())
        .unwrap();

    publisher
}

// move x units in x direction and y units in y direction
pub async fn nav_move(node: NavNode, distance_x: f64, turn_abs: f64) {
    let publisher = get_pub(node).await;

    let speed: f64 = 0.2;
    let travel_time = (distance_x / speed).ceil() as u64;

    println!("Travel time: {}", travel_time);
    println!("distance: {}", distance_x);

    let twist = Twist {
        linear: Vector3 {
            x: speed,
            y: 0.0,
            z: 0.0,
        }, // Move forward
        angular: Vector3 {
            x: 0.0,
            y: 0.0,
            z: turn_abs,
        }, // Rotate slightly
    };

    // Publish the initial move message
    match publisher.publish(&twist) {
        Ok(_) => println!(
            "Published: linear = {}, angular = {}",
            twist.linear.x, twist.angular.z
        ),
        Err(e) => eprintln!("Failed to publish intial move instructions: {}", e),
    }

    // Sleep for time needed to reach distance
    sleep(Duration::from_secs(travel_time)).await;
}

pub async fn rotate(node: NavNode, z: f64) {
    let cl = Arc::clone(&node);
    let publisher = get_pub(cl).await;

    let twist = Twist {
        linear: Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }, // Move forward
        angular: Vector3 {
            x: 0.0,
            y: 0.0,
            z: z,
        }, // Rotate slightly
    };

    // Publish the rotation message
    match publisher.publish(&twist) {
        Ok(_) => println!("Rotating instruction sent"),
        Err(e) => eprintln!("Failed to publish 360 rotating instructions {}", e),
    }

    sleep(Duration::from_millis(200)).await;

    nav_stop(node).await;
}

pub async fn rotate360(node: NavNode) {
    let cl = Arc::clone(&node);
    let publisher = get_pub(cl).await;

    let twist = Twist {
        linear: Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }, // Move forward
        angular: Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.3,
        }, // Rotate slightly
    };

    // Publish the rotation message
    match publisher.publish(&twist) {
        Ok(_) => println!("Rotating instruction sent"),
        Err(e) => eprintln!("Failed to publish 360 rotating instructions {}", e),
    }

    // Sleep for time needed to reach distance
    sleep(Duration::from_secs(5)).await;

    nav_stop(node).await;
}

pub async fn nav_stop(node: NavNode) {
    let publisher = get_pub(node).await;

    let twist = Twist {
        linear: Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        angular: Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
    };

    // Publish the rotation message
    match publisher.publish(&twist) {
        Ok(_) => println!("Stopping instruction sent"),
        Err(e) => eprintln!("Failed to stop the bot, this is bad {}", e),
    };
}

fn scale_0_to_200(value: f32) -> f32 {
    let new_min = 0.3;
    let new_max = -0.3;
    let old_min = 0.0;
    let old_max = 270.0;

    if value < old_min || value > old_max {
        panic!("Value out of range (0-200): {}", value);
    }

    let normalized = (value - old_min) / (old_max - old_min);
    new_min + normalized * (new_max - new_min)
}