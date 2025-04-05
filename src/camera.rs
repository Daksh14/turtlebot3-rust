use nokhwa::{
    Buffer, Camera,
    pixel_format::RgbFormat,
    utils::{
        ApiBackend, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType, Resolution,
    },
};
use tokio::sync::mpsc;

use crate::yolo::{self};


pub async fn cam_plus_yolo_detect() -> Result<(), ()> {
    let mut model = yolo::load_model().expect("The model should load");

    let format = RequestedFormat::with_formats(
        RequestedFormatType::AbsoluteHighestFrameRate,
        &[FrameFormat::MJPEG],
    );

    let mut camera: Camera =
        Camera::with_backend(CameraIndex::Index(0), format, ApiBackend::Video4Linux)
            .expect("Constructing camera should succeed");

    camera
        .set_resolution(Resolution {
            width_x: 800,
            height_y: 600,
        })
        .expect("setting res should work");


    camera.open_stream().expect("Stream should start");

    let (tx, mut rx) = mpsc::channel::<Buffer>(100);

    tokio::spawn(async move {
        loop {
            let buffer = camera.frame().expect("frame should be retrievable");

            tx.send(buffer).await
                .expect("Should be able to send over channel");
        }
    });

    loop {
        if let Some(buffer) = rx.recv().await {
            let img = buffer
                .decode_image::<RgbFormat>()
                .expect("decoding imgae to buffer should work");

            match yolo::detect(&mut model, img) {
                Ok(_) => (),
                Err(e) => println!("err: {:?}", e),
            }
        }
    }
}

pub async fn yolo_detect_test() {
    let mut model = yolo::load_model().expect("The model should load");

    // load the yolo model
    let img_path = "../data/IMG_8405.JPG"; // change the path if needed
    let img = image::ImageReader::open(img_path).unwrap().decode().unwrap();

    println!("yolo detect test {:?}", yolo::detect(&mut model, img.into()));
}