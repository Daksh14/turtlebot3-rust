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
