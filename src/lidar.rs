use r2r::sensor_msgs::msg::LaserScan;

use futures::{future, Stream, stream::StreamExt};

pub async fn lidar_scan(lidar_node: &r2r::Node, stream: impl Stream<Item = LaserScan>) {
    stream.for_each(|msg| {
        println!("topic: scan lidar data: {:?}", msg);
        future::ready(())
    })
    .await;
}