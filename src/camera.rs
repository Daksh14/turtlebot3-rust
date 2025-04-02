use opencv::prelude::*;
use opencv::{
    Result,
    videoio::{self, VideoCapture},
};

use crate::yolo;

use std::time::Instant;

pub async fn cam_plus_yolo_detect() -> Result<()> {
    let mut cam = VideoCapture::new(0, videoio::CAP_ANY)?;
    let opened = VideoCapture::is_opened(&cam)?;
    let mut frame_count = 0;
    let mut last_time = Instant::now();
    // load the yolo model
    let mut model = yolo::load_model().expect("The model should load");

    if !opened {
        panic!("Unable to open default camera!");
    }

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

        match yolo::detect(&mut model, &frame, 0.5, 0.5) {
            Ok(_) => {
                println!("Found something!");
            }
            Err(e) => {
                println!("Err {:?}", e);
            }
        }
    }
}
