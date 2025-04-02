use opencv::prelude::*;
use opencv::{
    Result,
    videoio::{self, VideoCapture},
};

use crate::yolo;

use std::sync::Mutex;
use std::time::Instant;

pub async fn cam_plus_yolo_detect() -> Result<()> {
    let mut cam = VideoCapture::new(1, videoio::CAP_ANY)?;

    let opened = VideoCapture::is_opened(&cam)?;
    let mut frame_count = 0;
    let mut last_time = Instant::now();
    // load the yolo model
    let mut model = yolo::load_model().expect("The model should load");
    let buffer: Mutex<Vec<Mat>> = Mutex::new(Vec::new());

    if !opened {
        panic!("Unable to open default camera!");
    }

    // loop {
    //     let mut frame = Mat::default();
    //     cam.read(&mut frame)?;
    // }

    // tokio::spawn({
    //     loop {
    //         let mut frame = Mat::default();
    //         cam.read(&mut frame)?;

    //         if let Ok(mut lock) = buffer.lock() {
    //             lock.push(frame);
    //         }
    //     }
    // })

    loop {
        let mut frame = Mat::default();
        cam.read(&mut frame)?;

        frame_count += 1;

        let elapsed = last_time.elapsed();
        if elapsed.as_secs() >= 1 {
            let fps = frame_count as f64 / elapsed.as_secs_f64();
            println!("FPS: {:.2}", fps);
            frame_count = 0;
            last_time = Instant::now();
        }
    }
}
