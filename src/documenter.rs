use chrono::{DateTime, Utc};
use mongodb::Client;
use serde::{Deserialize, Serialize};
/// Script for compiling together different pieces of data per module.
/// will be used for putting into mongodb
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::lidar::Direction;
use crate::mongodb::MongoLogger;

use crate::logger::{
    Battery, ErrorDetails, ErrorSeverity, EventType, LidarData, Location, LogEntry, Sensors, Status,
};

use r2r::sensor_msgs::msg::LaserScan;

use std::cell::RefCell;
use std::rc::Rc;
//pub type curDirection = Rc<RefCell<Direction>>;

static mut nangle_i: f32 = 1.1;
static mut nangle_o: f32 = 1.1;

static mut nspeed: u64 = 1234;
static mut ntt: f64 = 1.2;
static mut ndist: f64 = 1.2;

pub async fn generate_log_entry() -> LogEntry {
    println!("Generating log entry");

    let lidar_data = unsafe {
        LidarData {
            angle_increment: nangle_i,
            angle_min: nangle_o,
        }
    };

    let location = unsafe {
        Location {
            speed: nspeed,
            travel_time: ntt,
            distance: ndist,
        }
    };

    let battery = Battery {
        level: 85.0,
        voltage: 12.6,
        charging: false,
    };

    let sensors = Sensors {
        proximity: vec![1.5, 2.0, 1.8, 2.2],
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
    .with_lidar(lidar_data)
    .with_location(location)
    .with_battery(battery)
    .with_sensors(sensors)
    .with_error(error)
}

pub async fn push_lidar(ai: f32, ao: f32) {
    unsafe {
        nangle_i = ai;
        nangle_o = ao;
    }
}

pub async fn push_nav(spd: u64, tt: f64, dist: f64) {
    unsafe {
        nspeed = spd;
        ntt = tt;
        ndist = dist;
    }
}
