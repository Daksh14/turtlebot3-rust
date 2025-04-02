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

use futures::{Stream, StreamExt};
use lidar::Direction;
use r2r::QosProfile;
use r2r::sensor_msgs::msg::LaserScan;
use r2r::{Node, Publisher};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use opencv::prelude::*;
use tokio::time::{Duration, sleep};

// generate a node with a given name and namespace is set to turtlemove statically
pub fn generate_node(name: &str) -> r2r::Result<Node> {
    let name_space = "turtlemove";
    let ctx = r2r::Context::create()?;
    let node = r2r::Node::create(ctx, name, name_space)?;

    Ok(node)
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
    let (tx_cam, mut rx_cam) = mpsc::channel::<Mat>(100);

    // lidar process
    tokio::spawn({
        let mut lock = liadr_node_cl.lock().await;
        // subscribe to lidar node
        let qos = QosProfile::default().best_effort();
        let lidar_node_sub = lock.subscribe("/scan", qos).unwrap().boxed();

        lidar::lidar_scan(lidar_node_sub, tx)
    });

    // camera process
    tokio::spawn({
        crate::camera::camera_process(tx_cam)
    });

    tokio::spawn(async move {
        // load the yolo model
        let mut model = yolo::load_model().expect("The model should load");

        loop {
            match rx_cam.recv().await {
                Some(x) => {
                    println!("Recived camera data!");
                    if let Ok(x) = yolo::detect(&mut model, &x, 0.5, 0.5) {
                        println!("Found something!");
                    }
                },
                None => {},
            }
        }


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

                    // tokio::spawn(async move {
                    //     nav::nav_move(cl, 0.5, 0.0).await;
                    // });

                    // match rx.recv().await {
                    //     Some(scan) => {
                    //         if let Some(direction) = lidar::lidar_data(scan) {
                    //             println!("Detected direction: {:?}", direction);

                    //             match direction {
                    //                 Direction::North => {
                    //                     nav::nav_stop(Arc::clone(&cl_2)).await;
                    //                     nav::nav_move(cl_2, 0.2, 0.3).await;
                    //                 }
                    //                 Direction::NorthWest => {
                    //                     nav::nav_stop(Arc::clone(&cl_2)).await;
                    //                     nav::nav_move(cl_2, 0.2, 0.3).await;
                    //                 }
                    //                 Direction::NorthEast => {
                    //                     nav::nav_stop(Arc::clone(&cl_2)).await;
                    //                     nav::nav_move(cl_2, 0.2, -0.3).await;
                    //                 }
                    //                 _ => (),
                    //             }
                    //         }
                    //     }
// 
                        // None => {}
                    // }
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
