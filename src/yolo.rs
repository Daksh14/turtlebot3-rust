// yolo.rs
use image::DynamicImage;
use serde::Deserialize;
use usls::{Device, Nms, Options, Vision, YOLOTask, YOLOVersion, models::YOLO};

use std::path::Path;
use std::{fs::File, io::BufReader};

use crate::{XyXy, error::Error};

const YOLOV8_CLASS_LABELS: [&str; 5] = [
    "football",
    "green cone",
    "purple cone",
    "red cone",
    "yellow cone",
];

/// Config json file data structure
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ModelConfig {
    /// ONNX model absolute path
    pub model_path: String,
    /// array of class names
    pub class_names: Vec<String>,
    /// model input image size
    pub input_size: i32,
}

#[derive(Debug)]
pub struct Model {
    pub model: YOLO,
}

/// load ModelConfig json config file
fn load_model_file() -> Result<ModelConfig, Error> {
    // change the path if needed
    let file = File::open("../data/config.json")?;
    let reader = BufReader::new(file);
    let model_config: ModelConfig = serde_json::from_reader(reader)?;

    if !Path::new(&model_config.model_path).exists() {
        return Err(Error::OnnxModelFileNotFound);
    }

    Ok(model_config)
}

/// Load the model to put in the
pub fn load_model() -> Result<Model, Error> {
    let model_config = load_model_file()?;

    let options = Options::new()
        .with_model(&model_config.model_path)
        .expect("model should load")
        .with_yolo_version(YOLOVersion::V8)
        .with_yolo_task(YOLOTask::Detect)
        .with_device(Device::Cpu(0))
        .with_ixx(0, 0, (1, 1, 4).into())
        .with_ixx(0, 2, (0, 480, 480).into())
        .with_ixx(0, 3, (0, 480, 480).into())
        .with_confs(&[0.25])
        .with_names(&YOLOV8_CLASS_LABELS);

    let model = YOLO::new(options)?;

    println!("Yolo ONNX model loaded.");

    Ok(Model { model })
}

/// Yolo inference on pre allocated res_box,
pub fn detect(model: &mut YOLO, img: &[DynamicImage]) -> Result<XyXy, Error> {
    let mut result = model.run(img)?;

    // convert option to error, inference failed error
    let res = result.pop().ok_or(Error::NoDetection)?;
    let boxes = res.bboxes().ok_or(Error::NoDetection)?;

    let bbox = boxes.get(0).ok_or(Error::InferenceFailed)?;

    if bbox.confidence() >= 0.9 {
        return Ok(bbox.xyxy());
    }

    Err(Error::NoDetection)
}
