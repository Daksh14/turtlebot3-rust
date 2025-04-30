/// Script for compiling together different pieces of data per module.
/// will be used for putting into mongodb
use crate::logger::{
    Battery, ErrorDetails, ErrorSeverity, EventType, LidarData, Location, LogEntry, Sensors, Status,
};

// pub type curDirection = Rc<RefCell<Direction>>;

static mut NANGLE_I: f32 = 1.1;
static mut NANGLE_O: f32 = 1.1;

static mut NSPEED: u64 = 1234;
static mut NTT: f64 = 1.2;
static mut NDIST: f64 = 1.2;

pub async fn generate_log_entry() -> LogEntry {
    // println!("Generating log entry");

    let lidar_data = unsafe {
        LidarData {
            angle_increment: NANGLE_I,
            angle_min: NANGLE_O,
        }
    };

    let location = unsafe {
        Location {
            speed: NSPEED,
            travel_time: NTT,
            distance: NDIST,
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
        NANGLE_I = ai;
        NANGLE_O = ao;
    }
}

pub async fn push_nav(spd: u64, tt: f64, dist: f64) {
    unsafe {
        NSPEED = spd;
        NTT = tt;
        NDIST = dist;
    }
}
