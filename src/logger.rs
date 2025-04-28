//use crate::lidar::Direction;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// i was not in the right headspace writing these?
// temperature sensor, seriously??

#[derive(Debug, Serialize, Deserialize)]
pub enum EventType {
    Info,
    Warning,
    Error,
    Debug,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    Success,
    Failed,
    InProgress,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LidarData {
    pub angle_increment: f32,
    pub angle_min: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Location {
    pub distance: f64,
    pub speed: u64,
    pub travel_time: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Battery {
    pub level: f64,
    pub voltage: f64,
    pub charging: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sensors {
    pub proximity: Vec<f64>,
    //pub temperature: Option<f64>,
    //pub light: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub code: String,
    pub severity: ErrorSeverity,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub bot_id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
    pub operation: String,
    pub status: Status,
    pub message: String,
    pub lidar: Option<LidarData>,
    pub location: Option<Location>,
    pub battery: Option<Battery>,
    pub sensors: Option<Sensors>,
    pub error: Option<ErrorDetails>,
}

impl LogEntry {
    pub fn new(
        bot_id: String,
        event_type: EventType,
        operation: String,
        status: Status,
        message: String,
    ) -> Self {
        Self {
            bot_id,
            timestamp: Utc::now(),
            event_type,
            operation,
            status,
            message,
            lidar: None,
            location: None,
            battery: None,
            sensors: None,
            error: None,
        }
    }
    pub fn with_lidar(mut self, lidar: LidarData) -> Self {
        self.lidar = Some(lidar);
        self
    }
    pub fn with_location(mut self, location: Location) -> Self {
        self.location = Some(location);
        self
    }

    pub fn with_battery(mut self, battery: Battery) -> Self {
        self.battery = Some(battery);
        self
    }

    pub fn with_sensors(mut self, sensors: Sensors) -> Self {
        self.sensors = Some(sensors);
        self
    }

    pub fn with_error(mut self, error: ErrorDetails) -> Self {
        self.error = Some(error);
        self
    }
}
