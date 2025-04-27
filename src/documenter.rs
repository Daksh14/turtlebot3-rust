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

pub async fn generate_log_entry(lidar_data: LidarData) -> LogEntry {
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

// for lidar module
pub fn push_lidar(scan: &LaserScan) -> LidarData {
    LidarData {
        angle_min: scan.angle_min as f64,
        angle_increment: scan.angle_increment as f64,
        ranges: scan.ranges.clone(),
    }
}
