// yolo.rs
use ndarray::{s, Axis};
use ort::{
    session::{builder::SessionBuilder, Session},
    value::{TensorValueType, ValueRef},
};
use serde::{Deserialize, Serialize};

use std::{error::Error, fs::File, io::BufReader};

pub type Frame<'a> = ValueRef<'a, TensorValueType<f32>>;

const YOLOV8_CLASS_LABELS: [&str; 10] = [
    "blue cone",
    "cow",
    "football",
    "green cone",
    "mouse",
    "orange cone",
    "picFrame",
    "purple cone",
    "robot",
    "yellow cone",
];

#[derive(Debug, Serialize, Deserialize)]
pub struct BoxDetection {
    pub xmin: i32,  // bounding box left-top x
    pub ymin: i32,  // bounding box left-top y
    pub xmax: i32,  // bounding box right-bottom x
    pub ymax: i32,  // bounding box right-bottom y
    pub class: i32, // class index
    pub conf: f32,  // confidence score
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Detections {
    pub detections: Vec<BoxDetection>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ModelConfig {
    // refer to the `data/config.json`
    pub model_path: String,       // ONNX model absolute path
    pub class_names: Vec<String>, // array of class names
    pub input_size: i32,          // model input image size
}

pub struct Model {
    pub model: Session,
}

pub fn load_model() -> Result<Model, Box<dyn Error>> {
    let model_config = load_model_from_config().unwrap();
    println!("test");
    let model = SessionBuilder::new()?.commit_from_file(&model_config.model_path)?;
    println!("test");
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

// yolo.rs
pub fn detect(model_data: &mut Model, img: Frame) -> Result<(), Box<dyn std::error::Error>> {
    let model = &mut model_data.model;
    let model_inputs = [ort::session::SessionInputValue::from(img)];
    let outputs = model.run(model_inputs).expect("inference should work");

    let output = outputs["output0"]
        .try_extract_tensor::<f32>()?
        .t()
        .into_owned();

    let output = output.slice(s![.., .., 0]);
    let img_width = 640;
    let img_height = 640;
    let mut boxes = Vec::new();

    println!("{:?}", output);

    for row in output.axis_iter(Axis(0)) {
        let row: Vec<_> = row.iter().copied().collect();
        let (class_id, prob) = row
            .iter()
            // skip bounding box coordinates
            .skip(4)
            .enumerate()
            .map(|(index, value)| (index, *value))
            .reduce(|accum, row| if row.1 > accum.1 { row } else { accum })
            .unwrap();

        if prob < 0.5 {
            continue;
        }

        let label = YOLOV8_CLASS_LABELS[class_id];

        let xc = row[0] / 640. * (img_width as f32);
        let yc = row[1] / 640. * (img_height as f32);
        let w = row[2] / 640. * (img_width as f32);
        let h = row[3] / 640. * (img_height as f32);

        boxes.push((
            BoundingBox {
                x1: xc - w / 2.,
                y1: yc - h / 2.,
                x2: xc + w / 2.,
                y2: yc + h / 2.,
            },
            label,
            prob,
        ));
    }

    boxes.sort_by(|box1, box2| box2.2.total_cmp(&box1.2));
    let mut result = Vec::new();

    while !boxes.is_empty() {
        result.push(boxes[0]);
        boxes = boxes
            .iter()
            .filter(|box1| intersection(&boxes[0].0, &box1.0) / union(&boxes[0].0, &box1.0) < 0.7)
            .copied()
            .collect();
    }

    println!("result: {:?}", result);

    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct BoundingBox {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
}

fn intersection(box1: &BoundingBox, box2: &BoundingBox) -> f32 {
    (box1.x2.min(box2.x2) - box1.x1.max(box2.x1)) * (box1.y2.min(box2.y2) - box1.y1.max(box2.y1))
}

fn union(box1: &BoundingBox, box2: &BoundingBox) -> f32 {
    ((box1.x2 - box1.x1) * (box1.y2 - box1.y1)) + ((box2.x2 - box2.x1) * (box2.y2 - box2.y1))
        - intersection(box1, box2)
}
