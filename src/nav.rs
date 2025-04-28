use async_cell::sync::TakeWeak;
use r2r::geometry_msgs::msg::{Twist, Vector3};
use r2r::sensor_msgs::msg::LaserScan;
use rand::distr::{Distribution, Uniform};
use tokio::net::UdpSocket;
use tokio::sync::mpsc::Receiver;
use tokio::time::{Duration, sleep};

use crate::documenter;
use crate::lidar::{self};
// use crate::logger::Logger;
use crate::odom::OdomData;
use crate::yolo::ModelConfig;
use crate::{Sequence, XyXy, publisher::TwistPublisher};

use std::io::Result;
use std::sync::Arc;

// main navigation logic
pub async fn move_process(
    // sequence to start the nav move from
    starting_seq: Sequence,
    nav_node: crate::Node,
    lidar_rx: TakeWeak<LaserScan>,
    mut yolo_rx: Receiver<XyXy>,
    odom_rx: TakeWeak<OdomData>,
    config: ModelConfig,
    // logger: Logger,
) -> Result<()> {
    let mut current_sequence = starting_seq;

    let publisher = TwistPublisher::new(nav_node.clone());
    let distance_step = Uniform::new(400, 500).expect("Failed to create distance step");
    // listening for swarm data
    let socket = UdpSocket::bind("0.0.0.0:8000").await?;
    socket.connect(&config.addr[0]).await?;

    let socket_arc = Arc::new(socket);

    let mut buf = [0; 1024];

    loop {
        let socket_arc = Arc::clone(&socket_arc);
        tokio::spawn(async move {
            if let Ok(x) = socket_arc.try_recv(&mut buf) {
                println!("recieved from socket bytes: {}", x);
            }
        });

        match current_sequence {
            Sequence::RandomMovement => {
                // move randomly
                tokio::spawn(nav_move(
                    distance_step.sample(&mut rand::rng()) as f64,
                    0.2,
                    publisher.clone(),
                ));

                tokio::select! {
                        lidar = &lidar_rx => {
                            if let Some(scan) = lidar {
                                let direction = lidar::lidar_data(&scan);
                                // println!("{:?}", direction);
                                // logger.direction(direction);

                                if direction.north {
                                    nav_move(10.0, -0.2, publisher.clone()).await;
                                    rotate(0.1, publisher.clone()).await;
                                }

                                if direction.north_west {
                                    rotate(2.0, publisher.clone()).await;
                                }

                                if direction.north_east {
                                    rotate(-2.0, publisher.clone()).await;
                                }
                            }
                    }
                    // check yolo reciever
                    yolo = yolo_rx.recv() => {
                        if yolo.is_some() {
                            current_sequence = Sequence::TrackingToCharm;
                        }
                    }
                }
            }
            Sequence::TrackingToCharm => {
                if let Some((x1, _, _, y2)) = yolo_rx.recv().await {
                    if let Some(odom) = (&odom_rx).await {
                        let string = serde_json::to_string(&odom).expect("convertion should work");
                        let _string = string.as_bytes();
                        // communicate this fact to the bot1 and bot2
                        // s.send(&string).await;
                        println!("Sending swarm info");
                    }

                    if (200.0..=280.0).contains(&x1) {
                        nav_stop(publisher.clone());

                        if y2 < 485.0 {
                            nav_move(10.0, 0.17, publisher.clone()).await;
                        } else {
                            println!("{}", "charm collected");
                        }
                    } else {
                        let scaled = scale_0_to_200(x1);
                        rotate(scaled as f64, publisher.clone()).await;
                    }
                }
            }
            Sequence::Stop => {
                nav_stop(publisher.clone());
                println!("Stopping stop stop");

                break;
            }
        }
    }

    Ok(())
}

// move x units in x direction and y units in y direction
pub async fn nav_move(distance_x: f64, speed: f64, publisher: TwistPublisher) {
    // stop before putting new instruction
    nav_stop(publisher.clone());

    let mut speed = speed;

    if speed == 0.0 {
        speed = 0.1;
    }

    let travel_time = (distance_x / speed.abs()).ceil() as u64;

    println!("Speed: {}", speed);
    println!("Travel time: {}", travel_time);
    println!("distance: {}", distance_x);

    documenter::push_nav(speed as u64, travel_time as f64, distance_x);

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
        angular: Vector3 { x: 0.0, y: 0.0, z }, // Rotate slightly
    };

    // Publish the rotation message
    match publisher.publish(&twist) {
        Ok(_) => println!("Rotating instruction sent"),
        Err(e) => eprintln!("Failed to publish 360 rotating instructions {}", e),
    }

    sleep(Duration::from_millis(100)).await;

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
