use tokio::sync::mpsc;

use std::time::Instant;

use crate::yolo;

pub async fn cam_plus_yolo_detect() -> Result<()> {
    let mut cam = VideoCapture::new(0, videoio::CAP_V4L)?;

    let fourcc = VideoWriter::fourcc('M', 'J', 'P', 'G')?;

    cam.set(videoio::CAP_PROP_FRAME_WIDTH, 640.0)?;
    cam.set(videoio::CAP_PROP_FRAME_HEIGHT, 480.0)?;
    cam.set(videoio::CAP_PROP_FOURCC, fourcc as f64)?;

    let opened = VideoCapture::is_opened(&cam)?;

    // load the yolo model
    let mut model = yolo::load_model().expect("The model should load");
    // let img_path = "./data/test.jpg"; // change the path if needed
    // let img = imgcodecs::imread(img_path, imgcodecs::IMREAD_COLOR)?;

    // println!("yolo detect test {:?}", yolo::detect(&mut model, &img));

    let (tx, mut rx) = mpsc::channel::<Mat>(100);

    if !opened {
        panic!("Unable to open default camera!");
    }

    std::thread::spawn(move || {
        loop {
            let mut frame = Mat::default();
            cam.read(&mut frame).expect("should be able to read frame");

            tx.blocking_send(frame)
                .expect("Should be able to send frame");
            // currently getting 30 frames per second here
        }
    });

    let mut frame_count = 0;
    let mut last_time = Instant::now();

    loop {
        if let Some(x) = rx.recv().await {
            frame_count += 1;
            let elapsed = last_time.elapsed();

            if elapsed.as_secs() >= 1 {
                let fps = frame_count as f64 / elapsed.as_secs_f64();
                println!("FPS: {:.2}", fps);
                frame_count = 0;
                last_time = Instant::now();
            }

            // we're getting 1 fps here because of the yolo model inference
            match yolo::detect(&mut model, &x) {
                Ok(_) => {
                    println!("Detected something");
                }
                _ => (),
            }
        }
    }
}
