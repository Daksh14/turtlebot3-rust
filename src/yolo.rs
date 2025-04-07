// yolo.rs
use serde::Deserialize;
use usls::{models::YOLO, Bbox, Device, Nms, Options, Vision, YOLOTask, YOLOVersion};

use image::{DynamicImage, RgbImage};
use std::{error::Error, fs::File, io::BufReader};

pub type Frame = RgbImage;

const YOLOV8_CLASS_LABELS: [&str; 8] = [
    "cow",
    "football",
    "green cone",
    "picFrame",
    "purple cone",
    "red cone",
    "robot drummer",
    "yellow cone",
];

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ModelConfig {
    // refer to the `data/config.json`
    pub model_path: String,       // ONNX model absolute path
    pub class_names: Vec<String>, // array of class names
    pub input_size: i32,          // model input image size
}

pub struct Model {
    pub model: YOLO,
}

pub fn load_model() -> Result<Model, Box<dyn Error>> {
    let model_config = load_model_from_config().unwrap();

    let options = Options::new()
        .with_model(&model_config.model_path)
        .expect("model should load")
        .with_yolo_version(YOLOVersion::V8)
        .with_yolo_task(YOLOTask::Detect)
        .with_device(Device::Cpu(0))
        .with_ixx(0, 0, (1, 1, 4).into())
        .with_ixx(0, 2, (0, 360, 360).into())
        .with_ixx(0, 3, (0, 360, 360).into())
        .with_confs(&[0.25])
        .with_names(&YOLOV8_CLASS_LABELS);

    let model = YOLO::new(options).expect("yolo model to load");

    println!("Yolo ONNX model loaded.");

    Ok(Model { model })
}

fn load_model_from_config() -> Result<ModelConfig, Box<dyn Error>> {
    let file = File::open("../data/config.json"); // change the path if needed
    let file = match file {
        Ok(file) => file,
        Err(e) => {
            println!("{:?}", e);
            std::process::exit(0)
        }
    };

    let reader = BufReader::new(file);
    let model_config: std::result::Result<ModelConfig, serde_json::Error> =
        serde_json::from_reader(reader);
    let model_config = match model_config {
        Ok(model_config) => model_config,
        Err(_) => {
            println!("Invalid config json.");
            std::process::exit(0)
        }
    };

    if !std::path::Path::new(&model_config.model_path).exists() {
        println!(
            "ONNX model in {model_path} does NOT exist.",
            model_path = model_config.model_path
        );
        std::process::exit(0)
    }

    Ok(model_config)
}

pub fn detect(model_data: &mut Model, img: Frame) -> Vec<Bbox> {
    let model = &mut model_data.model;

    let result = model.run(&[DynamicImage::ImageRgb8(img)]);

    let mut res_bbox = Vec::with_capacity(2);

    if let Ok(result) = result {
        for res in result {
            if let Some(bboxes) = res.bboxes() {
                for bbox in bboxes {
                    let conf = bbox.confidence();

                    if conf > 0.8 {
                        res_bbox.push(bbox.clone());
                    }
                }
            }
        }
    }

    res_bbox
}
