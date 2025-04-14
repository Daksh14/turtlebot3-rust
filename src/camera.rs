use image::DynamicImage;
use nokhwa::{
    Camera,
    pixel_format::RgbFormat,
    utils::{
        ApiBackend, CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType,
        Resolution,
    },
};
use tokio::sync::mpsc::Sender;

use crate::{
    XyXy,
    error::Error,
    yolo::{self},
};

pub fn cam_plus_yolo_detect(yolo_tx: Sender<XyXy>) -> Result<(), Error> {
    let mut model = yolo::load_model()?.model;

    let res = Resolution {
        width_x: 640,
        height_y: 480,
    };

    let frame_format = FrameFormat::MJPEG;
    let fps = 30;
    let req_format_type = RequestedFormatType::Exact(CameraFormat::new(res, frame_format, fps));
    let format = RequestedFormat::new::<RgbFormat>(req_format_type);

    let mut camera: Camera =
        Camera::with_backend(CameraIndex::Index(0), format, ApiBackend::Video4Linux)?;

    camera.open_stream()?;

    loop {
        let buffer = camera.frame()?;

        if let Ok(img) = buffer.decode_image::<RgbFormat>() {
            match yolo::detect(&mut model, &[DynamicImage::ImageRgb8(img)]) {
                Ok(bbox) => {
                    let _ = yolo_tx.blocking_send(bbox);
                }
                Err(e) => {
                    // stop loop if inference failed for some reason
                    if let Error::InferenceFailed = e {
                        break;
                    }
                }
            }
        }
    }

    println!("Camera stream closed");

    Ok(())
}

#[allow(dead_code)]
pub async fn yolo_detect_test() -> Result<(), Error> {
    let mut model = yolo::load_model()?.model;

    // load the yolo model
    let img_path = "../data/IMG_8405.JPG"; // change the path if needed
    let img = image::ImageReader::open(img_path)
        .unwrap()
        .decode()
        .unwrap();

    println!("yolo detect test {:?}", yolo::detect(&mut model, &[img])?);

    Ok(())
}
