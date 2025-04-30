use async_cell::sync::AsyncCell;
use futures::Stream;
use futures::stream::StreamExt;
use r2r::nav_msgs::msg::Odometry;
use serde::{Deserialize, Serialize};

use std::sync::Arc;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OdomData {
    pub x1: f64,
    pub y1: f64,
    pub z: f64,
    pub w: f64,
}

pub async fn listen<T: Stream<Item = Odometry> + Unpin>(
    mut stream: T,
    tx: Arc<AsyncCell<OdomData>>,
) {
    loop {
        match stream.next().await {
            Some(msg) => {
                let pose = msg.pose.pose.position;
                let x1 = pose.x;
                let y1 = pose.y;

                let orientation = msg.pose.pose.orientation;
                let z = orientation.z;
                let w = orientation.w;

                let data = OdomData { x1, y1, z, w };

                tx.set(data);
            }
            _ => (),
        }
    }
}
