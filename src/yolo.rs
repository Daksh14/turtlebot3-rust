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
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ModelConfig {
    /// ONNX model absolute path
    pub model_path: String,
    /// array of class names
    pub class_names: Vec<String>,
    /// model input image size
    pub input_size: i32,
    /// swarm addresses
    pub addr: Vec<String>,
}

#[derive(Debug)]
pub struct Model {
    pub model: YOLO,
}

/// load ModelConfig json config file
pub fn load_model_file() -> Result<ModelConfig, Error> {
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
pub fn load_model(model_config: ModelConfig) -> Result<Model, Error> {
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
pub fn detect(model: &mut YOLO, img: &[DynamicImage]) -> Option<XyXy> {
    let popped = model.run(img).ok()?.pop()?;

    // convert option to error, inference failed error
    popped
        .bboxes()
        .and_then(|x| x.first())
        .filter(|&bbox| bbox.confidence() >= 0.9)
        .map(usls::Bbox::xyxy)
}
