use nokhwa::{
    Buffer, Camera,
    pixel_format::RgbFormat,
    utils::{
        ApiBackend, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType, Resolution,
    },
};
use std::sync::mpsc;

use crate::yolo::{self};


pub fn cam_plus_yolo_detect() -> Result<(), ()> {
    let mut model = yolo::load_model().expect("The model should load");

    let format = RequestedFormat::with_formats(
        RequestedFormatType::AbsoluteHighestFrameRate,
        &[FrameFormat::YUYV],
    );

    let mut camera: Camera =
        Camera::with_backend(CameraIndex::Index(0), format, ApiBackend::Video4Linux)
            .expect("Constructing camera should succeed");

    camera
        .set_resolution(Resolution {
            width_x: 640,
            height_y: 360,
        })
        .expect("setting res should work");


    camera.open_stream().expect("Stream should start");

    let (tx, rx) = mpsc::channel::<Buffer>();

    std::thread::spawn(move || {
        loop {
            if let Ok(buffer) = rx.recv() {
                let img = buffer.decode_image::<RgbFormat>()
                    .expect("decoding image to buffer should work");
                
                println!("detected bboxes: {:?}", yolo::detect(&mut model, img));
            }
        }
    });

    loop {
        let buffer = camera.frame().expect("frame should be retrievable");

        tx.send(buffer).expect("Should be able to send over channel");
    }
}

pub async fn yolo_detect_test() {
    let mut model = yolo::load_model().expect("The model should load");

    // load the yolo model
    let img_path = "../data/IMG_8405.JPG"; // change the path if needed
    let img = image::ImageReader::open(img_path).unwrap().decode().unwrap();

    println!("yolo detect test {:?}", yolo::detect(&mut model, img.into()));
}