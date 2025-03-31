use std::f32;

use futures::{
    Stream, future,
    stream::{BoxStream, StreamExt},
};
use r2r::sensor_msgs::msg::LaserScan;
use tokio::sync::mpsc::Sender;

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

pub fn lidar_data(data: LaserScan) {
    let lidar_data = data.ranges;
    let threshold = 0.5; // Distance threshold for detecting objects
    let len = lidar_data.len();

    println!("{}", len);

    // the bot is facing in a certain direction for us to determine the left, center, and right region
    let bottom_left = &lidar_data[0..20];
    let bottom_right = &lidar_data[20..40];
    let right = &lidar_data[40..60];
    let top_right = &lidar_data[60..80];
    let top_left = &lidar_data[80..100];
    let left = &lidar_data[100..120];
    let bottom_left = &lidar_data[120..140];
    let bottom_right = &lidar_data[140..160];

    let min_bottom_left = find_n_min_values(bottom_left.to_vec());
    let avg = find_average(min_bottom_left);
    let mut close = false;

    if avg < threshold {
        close = true;
    }

    // Print results
    if close {
        println!("Object detected nearby on the bottom left side!");
    } else {
        println!("No close object detected.");
    }
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
