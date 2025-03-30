use r2r::sensor_msgs::msg::LaserScan;

use futures::{
    Stream, future,
    stream::{BoxStream, StreamExt},
};

pub async fn lidar_scan<'a>(stream: BoxStream<'a, LaserScan>) {
    // block and keep recivin messages
    let mut stream = stream;
    while let Some(message) = stream.next().await {
        println!("Received: {:?}", message);
    }
}
