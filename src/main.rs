/// Camera logic
mod camera;
/// Graceful error handling
mod error;
/// lidar module
mod lidar;
/// logger module
mod logger;
/// mongodb module
mod mongodb;
/// Navigation logic
mod nav;
/// Odometer module for swarm algorithm
mod odom;
/// Publisher module
mod publisher;
/// yolo module
mod yolo;

use crate::logger::{
    Battery, ErrorDetails, ErrorSeverity, EventType, Location, LogEntry, Sensors, Status,
};
use crate::yolo::{ModelConfig, load_model_file};
use async_cell::sync::AsyncCell;
use mongodb::MongoLogger;
use r2r::QosProfile;
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
    // Start randomly moving in x, y direction
    RandomMovement,
    // If charm is located, start moving towards it
    TrackingToCharm,
    // Stop
    Stop,
}

/// General log struct creator for easy mongodb logging
/// [NOTE]: right now has mock values, needs to be updated to be realtime
/// this is a great idea if done right !!!
async fn update_and_create_log_entry() -> LogEntry {
    let location = Location {
        x: 10.5,
        y: 20.3,
        orientation: 45.0,
    };

    let battery = Battery {
        level: 85.0,
        voltage: 12.6,
        charging: false,
    };

    let sensors = Sensors {
        proximity: vec![1.5, 2.0, 1.8, 2.2],
        temperature: Some(25.5),
        light: Some(800.0),
    };

    let error = ErrorDetails {
        code: "E001".to_string(),
        severity: ErrorSeverity::Low,
    };

    LogEntry::new(
        "bot_001".to_string(),
        EventType::Info,
        "Navigation".to_string(),
        Status::Success,
        "Successfully completed navigation task".to_string(),
    )
    .with_location(location)
    .with_battery(battery)
    .with_sensors(sensors)
    .with_error(error)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_model_file()?;
    // setup mongo logger
    // NOTE: IMPORTANT - formatting will probably need to change depending on actual layout of the
    // mongo server. this is being tested on my own atlas mongodb server.
    let mongo_logger = MongoLogger::new(
        "mongodb+srv://team3:sjL9N7hFGnbT6wCD@team3logs.grhns2t.mongodb.net/?retryWrites=true&w=majority&appName=team3logs",
        "teamthreedb",
        "teamthreecollection"
    ).await?;

    // create our log entry
    let nlog = update_and_create_log_entry().await;
    mongo_logger.log_entry(nlog).await?;

    // duration of each node spin
    let node_spin_dur = std::time::Duration::from_millis(5);

    // setup all the nodes
    let nav_node = generate_node("nav_node")?;
    let lidar_node = generate_node("lidar_node")?;

    // Launch the lidar communication channel
    let cell_lidar = AsyncCell::shared();
    let weak_lidar = cell_lidar.take_weak();

    let cell_odom = AsyncCell::shared();
    let weak_odom = cell_odom.take_weak();

    let (yolo_tx, yolo_rx) = mpsc::channel::<XyXy>(1000);

    let config_cl = config.clone();
    // camera process + yolo detect
    std::thread::spawn(move || {
        let cam = camera::cam_plus_yolo_detect(yolo_tx, config_cl);

        println!("{:?}", cam)
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

        lidar::lidar_scan(&mut lidar_node_sub, cell_lidar).await;
    });

    let cl = Arc::clone(&nav_node);

    // Odometer process
    tokio::spawn(async move {
        let mut odom_node_sub = {
            let mut lock = cl.lock().expect("Locking the nav node should work");
            let qos = QosProfile::default();
            lock.subscribe("/odom", qos)
                .expect("Subscribing to odom should work")
        };

        odom::listen(&mut odom_node_sub, cell_odom).await;
    });

    let cl = Arc::clone(&nav_node);
    let config_cl = config.clone();
    // navigation process
    tokio::spawn(async move {
        // this is what the bot is doing at any point in time
        let start_sequence = Sequence::RandomMovement;

        let x = nav::move_process(
            start_sequence,
            cl,
            weak_lidar,
            yolo_rx,
            weak_odom,
            config_cl,
        )
        .await;
        println!("{:?}", x);
    });

    loop {
        if let Ok(mut nav_handle) = nav_node.lock() {
            nav_handle.spin_once(node_spin_dur);
        }

        if let Ok(mut lidar_handle) = lidar_node.lock() {
            lidar_handle.spin_once(node_spin_dur);
        }
    }
}
