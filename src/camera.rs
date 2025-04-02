use opencv::prelude::*;
use opencv::{
    Result,
    videoio::{self, VideoCapture, VideoWriter},
};
use tokio::sync::mpsc;

use crate::yolo;

pub async fn cam_plus_yolo_detect() -> Result<()> {
    let mut cam = VideoCapture::new(0, videoio::CAP_V4L)?;

    let fourcc = VideoWriter::fourcc('M', 'J', 'P', 'G')?;

    cam.set(videoio::CAP_PROP_FOURCC, fourcc as f64)?;

    let opened = VideoCapture::is_opened(&cam)?;

    // load the yolo model
    let mut model = yolo::load_model().expect("The model should load");
    let (tx, mut rx) = mpsc::channel::<Mat>(100);

    if !opened {
        panic!("Unable to open default camera!");
    }

    tokio::spawn(async move {
        loop {
            let mut frame = Mat::default();
            cam.read(&mut frame).expect("should be able to read frame");

            tx.send(frame).await.expect("Should be able to send frame");
        }
    });

    loop {
        if let Some(x) = rx.recv().await {
            println!("{:?}", x);
            match yolo::detect(&mut model, &x, 0.5, 0.5) {
                Ok(_) => {
                    println!("Detected something");
                }
                _ => (),
            }
        }
    }
}
