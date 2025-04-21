use std::f32;
use std::sync::Arc;

use async_cell::sync::AsyncCell;
use futures::Stream;
use futures::stream::StreamExt;
use r2r::sensor_msgs::msg::LaserScan;

#[derive(Debug)]
pub struct Direction {
    pub north: bool,
    pub north_east: bool,
    pub east: bool,
    pub south_east: bool,
    pub south: bool,
    pub south_west: bool,
    pub west: bool,
    pub north_west: bool,
}

pub async fn lidar_scan<T: Stream<Item = LaserScan> + Unpin>(
    mut stream: T,
    tx: Arc<AsyncCell<LaserScan>>,
) {
    loop {
        match stream.next().await {
            Some(msg) => {
                tx.set(msg);
            }
            // dont do anything if we dont get any lidar data
            None => (),
        }
    }
}

pub fn lidar_data(scan: LaserScan) -> Direction {
    let detection_threshold = 0.3;
    let angle_min = scan.angle_min;
    let angle_increment = scan.angle_increment;
    let ranges = &scan.ranges;

    let mut dir = Direction {
        north: false,
        north_east: false,
        east: false,
        south_east: false,
        south: false,
        south_west: false,
        west: false,
        north_west: false,
    };

    for (i, &range) in ranges.iter().enumerate() {
        if range.is_nan() || range > detection_threshold {
            continue;
        }

        let angle = angle_min + (i as f32) * angle_increment;
        // Convert angle to degrees and normalize between 0-360
        let deg = ((angle.to_degrees() + 360.0 + 15.0) % 360.0) as u32;
        let sector = (deg / 45) as usize;

        match sector {
            0 => dir.north = true,
            1 => dir.north_east = true,
            2 => dir.east = true,
            3 => dir.south_east = true,
            4 => dir.south = true,
            5 => dir.south_west = true,
            6 => dir.west = true,
            7 => dir.north_west = true,
            _ => (),
        };
    }

    dir
}
