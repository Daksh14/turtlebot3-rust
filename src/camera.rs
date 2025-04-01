use opencv::prelude::*;
use opencv::{videoio, Result};

use tokio::sync::mpsc::Sender;

fn camera_process(tx: Sender<Mat>) -> Result<()> {
    let cam = videioio::VideoCapture::new(0, videoio::CAP_ANY)?;
    let opened = videioio::VideoCapture::is_opened(&cam)?;

    if !opened {
        panic!("Unable to open default camera!");
    }

    loop {
        let mut frame = Mat::default();
        cam.read(&mut frame)?;

        if let Err(e) = tx.send(frame).await {
            println!("receiver dropped for camera: {}", e);

            return;
        }
    }
}