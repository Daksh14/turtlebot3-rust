use r2r::sensor_msgs::msg::LaserScan;
use r2r::{
    Node, Publisher, QosProfile,
    geometry_msgs::msg::{Twist, Vector3},
};
use rand::distr::{Distribution, Uniform};
use tokio::sync::mpsc::Receiver;
use tokio::time::{Duration, sleep};

use crate::lidar::{self, Direction};
use crate::{Sequence, XyXy, publisher::TwistPublisher};

// main navigation logic
pub async fn move_process(
    // sequence to start the nav move from
    starting_seq: Sequence,
    nav_node: crate::Node,
    mut lidar_rx: Receiver<LaserScan>,
    mut yolo_rx: Receiver<XyXy>,
) {
    let mut current_sequence = starting_seq;

    let publisher = TwistPublisher::new(nav_node.clone());
    let distance_step = Uniform::new(400, 500).expect("Failed to create distance step");

    loop {
        match current_sequence {
            Sequence::Intial360Rotation => {
                rotate360(publisher.clone());

                sleep(Duration::from_secs(3)).await;

                current_sequence = Sequence::RandomMovement;
            }
            Sequence::RandomMovement => {
                let publisher_cl = publisher.clone();

                let random_movement_handle = tokio::spawn({
                    let mut rng = rand::rng();

                    let distance = distance_step.sample(&mut rng) as f64;

                    nav_move(distance, 0.17, publisher_cl)
                });

                match lidar_rx.recv().await {
                    Some(scan) => {
                        let direction = lidar::lidar_data(scan);
                        println!("Detected direction: {:?}", direction);

                        if direction.north {
                            random_movement_handle.abort();
                            nav_move(35.0, -0.2, publisher.clone()).await;
                            rotate(2.0, publisher.clone()).await;
                        }

                        if direction.north_west {
                            rotate(2.0, publisher.clone()).await;
                        }

                        if direction.north_east {
                            rotate(-2.0, publisher.clone()).await;
                        }

                        if direction.south_west && direction.west {
                            rotate(-2.0, publisher.clone()).await;
                        }

                        if direction.east && direction.south_east {
                            rotate(2.0, publisher.clone()).await;
                        }
                    }
                    None => {}
                }
            }
            Sequence::TrackingToCharm => {
                if let Some((x1, _, _, y2)) = yolo_rx.recv().await {
                    println!("{:?}", y2);

                    if x1 >= 200.0 && x1 <= 280.0 {
                        nav_stop(publisher.clone());
                        println!("centered: publisher forward: {}", scale_600_to_0(y2));
                    } else {
                        let scaled = scale_0_to_200(x1);
                        rotate(scaled as f64, publisher.clone()).await;
                    }
                }
            }
            Sequence::SharmCollected => {
                // SharmCollected
            }
            Sequence::Stop => {
                nav_stop(publisher.clone());
                println!("Stopping stop stop");

                break;
            }
        }
    }
}

// move x units in x direction and y units in y direction
pub async fn nav_move(distance_x: f64, speed: f64, publisher: TwistPublisher) {
    // stop before putting new instruction
    nav_stop(publisher.clone());

    let mut speed = speed;

    if (speed == 0.0) {
        speed = 0.1;
    }

    let travel_time = (distance_x / speed.abs()).ceil() as u64;

    println!("Speed: {}", speed);
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
            z: 0.0,
        }, // Rotate slightly
    };

    // Publish the initial move message
    match publisher.publish(&twist) {
        Ok(_) => (),
        Err(e) => eprintln!("Failed to publish intial move instructions: {}", e),
    }

    // Sleep for time needed to reach distance
    sleep(Duration::from_millis(travel_time)).await;

    nav_stop(publisher);
}

pub async fn rotate(z: f64, publisher: TwistPublisher) {
    nav_stop(publisher.clone());

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

    sleep(Duration::from_millis(100)).await;

    nav_stop(publisher);
}

pub async fn rotate360(publisher: TwistPublisher) {
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

    nav_stop(publisher);
}

pub fn nav_stop(publisher: TwistPublisher) {
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
    let new_min = 0.5;
    let new_max = -0.5;
    let old_min = 0.0;
    let old_max = 500.0;

    let normalized = (value - old_min) / (old_max - old_min);
    new_min + normalized * (new_max - new_min)
}

fn scale_600_to_0(value: f32) -> f32 {
    let new_min = 0.0;
    let new_max = 0.1;
    let old_min = 200.0;
    let old_max = 600.0;

    let normalized = (value - old_min) / (old_max - old_min);
    new_min + normalized * (new_max - new_min)
}
