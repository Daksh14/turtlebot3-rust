use std::f32;

use futures::{
    Stream, future,
    stream::{BoxStream, StreamExt},
};
use r2r::sensor_msgs::msg::LaserScan;
use tokio::sync::mpsc::Sender;

#[derive(Debug)]
pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

pub async fn lidar_scan<'a>(stream: BoxStream<'a, LaserScan>, tx: Sender<LaserScan>) {
    // block and keep recivin messages
    let mut stream = stream;

    while let Some(message) = stream.next().await {
        if let Err(_) = tx.send(message).await {
            println!("receiver dropped");

            return;
        }
    }
}

pub fn lidar_data(data: LaserScan) -> Option<Direction> {
    let lidar_data = data.ranges;
    let threshold = 0.5; // Distance threshold for detecting objects
    let len = lidar_data.len();

    let size_of_quater = 23;

    // there should be 8 quaters, the approx length of the lidar range is 203 elements
    let quater_one = &lidar_data[0..size_of_quater];
    let quater_two = &lidar_data[size_of_quater..size_of_quater * 2];
    let quater_three = &lidar_data[size_of_quater * 2..size_of_quater * 3];
    let quater_four = &lidar_data[size_of_quater * 3..size_of_quater * 4];
    let quater_five = &lidar_data[size_of_quater * 4..size_of_quater * 5];
    let quater_six = &lidar_data[size_of_quater * 5..size_of_quater * 6];
    let quater_seven = &lidar_data[size_of_quater * 6..size_of_quater * 7];
    let quater_eight = &lidar_data[size_of_quater * 7..size_of_quater * 8];

    let min_and_avg_q_one = find_average(find_n_min_values(quater_one.to_owned()));
    let min_and_avg_q_two = find_average(find_n_min_values(quater_two.to_owned()));
    let min_and_avg_q_three = find_average(find_n_min_values(quater_three.to_owned()));
    let min_and_avg_q_four = find_average(find_n_min_values(quater_four.to_owned()));
    let min_and_avg_q_five = find_average(find_n_min_values(quater_five.to_owned()));
    let min_and_avg_q_six = find_average(find_n_min_values(quater_six.to_owned()));
    let min_and_avg_q_seven = find_average(find_n_min_values(quater_seven.to_owned()));
    let min_and_avg_q_eight = find_average(find_n_min_values(quater_eight.to_owned()));

    if min_and_avg_q_one < threshold {
        return Some(Direction::North);
    }

    if min_and_avg_q_two < threshold {
        return Some(Direction::NorthEast);
    }

    if min_and_avg_q_three < threshold {
        return Some(Direction::East);
    }

    if min_and_avg_q_four < threshold {
        return Some(Direction::SouthEast);
    }

    if min_and_avg_q_five < threshold {
        return Some(Direction::South);
    }

    if min_and_avg_q_six < threshold {
        return Some(Direction::SouthWest);
    }

    if min_and_avg_q_seven < threshold {
        return Some(Direction::West);
    }

    if min_and_avg_q_eight < threshold {
        return Some(Direction::NorthWest);
    }

    None
}

fn find_n_min_values(arr: Vec<f32>) -> Vec<f32> {
    let n = 3;
    let mut data = arr.clone();

    data.retain(|&x| !x.is_nan());

    // Sort the array
    data.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Get the n smallest values
    data.iter().take(n).cloned().collect()
}

fn find_average(arr: Vec<f32>) -> f32 {
    let sum: f32 = arr.iter().sum();
    sum / arr.len() as f32
}
