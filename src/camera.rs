use nokhwa::{
    Buffer, Camera,
    pixel_format::RgbFormat,
    utils::{
        ApiBackend, CameraIndex, FrameFormat, CameraFormat, RequestedFormat, Resolution, RequestedFormatType,
    },
};
use std::time::Instant;
use std::sync::mpsc;

use crate::yolo::{self};


pub fn cam_plus_yolo_detect() -> Result<(), ()> {
    let mut model = yolo::load_model().expect("The model should load");

    let res = Resolution {
        width_x: 640,
        height_y: 360,
    };

    let frame_format = FrameFormat::MJPEG;
    let fps = 30;

    let req_format_type = RequestedFormatType::Exact(CameraFormat::new(res, frame_format, fps));
    let format = RequestedFormat::new::<RgbFormat>(req_format_type);

    let mut camera: Camera =
        Camera::with_backend(CameraIndex::Index(0), format, ApiBackend::Video4Linux)
            .expect("Constructing camera should succeed");

    camera.open_stream().expect("Stream should start");

    loop {
        let buffer = camera.frame().expect("frame should be retrievable");

        let img = buffer.decode_image::<RgbFormat>().expect("decoding image to buffer should work");

        println!("{:?}", yolo::detect(&mut model, img));
    }
}

pub async fn yolo_detect_test() {
    let mut model = yolo::load_model().expect("The model should load");

    // load the yolo model
    let img_path = "../data/IMG_8405.JPG"; // change the path if needed
    let img = image::ImageReader::open(img_path).unwrap().decode().unwrap();

    println!("yolo detect test {:?}", yolo::detect(&mut model, img.into()));
}