use nokhwa::{
    Buffer, Camera,
    pixel_format::RgbFormat,
    utils::{
        ApiBackend, CameraIndex, FrameFormat, CameraFormat, RequestedFormat, Resolution, RequestedFormatType,
    },
};
use std::time::Instant;
use std::sync::mpsc;
use usls::Bbox;
use std::sync::mpsc::Sender;

use crate::{yolo::{self}, YoloResult};


pub fn cam_plus_yolo_detect(yolo_tx: Sender<YoloResult>) -> Result<(), ()> {
    let mut model = yolo::load_model().expect("The model should load");

    let res = Resolution {
        width_x: 800,
        height_y: 600,
    };

    let frame_format = FrameFormat::MJPEG;
    let fps = 5;
    let req_format_type = RequestedFormatType::Exact(CameraFormat::new(res, frame_format, fps));
    let format = RequestedFormat::new::<RgbFormat>(req_format_type);

    let mut camera: Camera =
        Camera::with_backend(CameraIndex::Index(0), format, ApiBackend::Video4Linux)
            .expect("Constructing camera should succeed");

    camera.open_stream().expect("Stream should start");

    loop {
        if let Ok(buffer) = camera.frame() {
            let img = buffer.decode_image::<RgbFormat>().expect("decoding image to buffer should work");

            yolo_tx.send(yolo::detect(&mut model, img));
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