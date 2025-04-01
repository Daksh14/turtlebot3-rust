use opencv::prelude::*;
use opencv::{videoio::{VideoCapture, self}, Result};
use opencv::core::OpenCLInitError;

use tokio::sync::mpsc::Sender;

pub async fn camera_process(tx: Sender<Mat>) -> Result<()> {
    let mut cam = VideoCapture::new(0, videoio::CAP_ANY)?;
    let opened = VideoCapture::is_opened(&cam)?;

    if !opened {
        panic!("Unable to open default camera!");
    }

    loop {
        let mut frame = Mat::default();
        cam.read(&mut frame)?;

        if let Err(e) = tx.send(frame).await {
            println!("receiver dropped for camera: {}", e);

            return Err(opencv::error::Error {
                code: OpenCLInitError,
                message: String::from("Camera reading is stopped"),
            });
        }
    }
}
