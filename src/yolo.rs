// yolo.rs
use opencv::{
    core::{self, MatExprTraitConst, MatTraitConst},
    core::{Mat, Rect, Vector},
    dnn::{self, NetTrait, NetTraitConst},
};
use serde::{Deserialize, Serialize};
use std::{error::Error, fs::File, io::BufReader};

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
pub struct ModelConfig {
    // refer to the `data/config.json`
    pub model_path: String,       // ONNX model absolute path
    pub class_names: Vec<String>, // array of class names
    pub input_size: i32,          // model input image size
}

pub struct Model {
    pub model: dnn::Net, // we will use OpenCV dnn module to load the ONNX model
    pub model_config: ModelConfig,
}

#[derive(Debug)]
pub struct MatInfo {
    width: f32,       // original image width
    height: f32,      // original image height
    scaled_size: f32, // effective size fed into the model
}

pub fn load_model() -> Result<Model, Box<dyn Error>> {
    let model_config = load_model_from_config().unwrap();
    let model = dnn::read_net_from_onnx(&model_config.model_path);

    let mut model = match model {
        Ok(model) => model,
        Err(_) => {
            println!("Invalid ONNX model.");
            std::process::exit(0)
        }
    };
    model.set_preferable_backend(dnn::DNN_BACKEND_OPENCV)?;

    println!("Yolo ONNX model loaded.");

    Ok(Model {
        model,
        model_config,
    })
}

fn load_model_from_config() -> Result<ModelConfig, Box<dyn Error>> {
    let file = File::open("data/config.json"); // change the path if needed
    let file = match file {
        Ok(file) => file,
        Err(_) => {
            println!("data/config.json does NOT exist.");
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

fn pre_process(img: &Mat) -> opencv::Result<Mat> {
    let width = img.cols();
    let height = img.rows();

    let _max = std::cmp::max(width, height);
    // keep the original aspect ratio by adding black padding
    let mut result = Mat::zeros(_max, _max, core::CV_8UC3)
        .unwrap()
        .to_mat()
        .unwrap();
    img.copy_to(&mut result)?;

    Ok(result)
}

// yolo.rs
pub fn detect(
    model_data: &mut Model,
    img: &Mat,
    conf_thresh: f32,
    nms_thresh: f32,
) -> opencv::Result<Detections> {
    let model = &mut model_data.model;
    let model_config = &mut model_data.model_config;

    let mat_info = MatInfo {
        width: img.cols() as f32,
        height: img.rows() as f32,
        scaled_size: model_config.input_size as f32,
    };

    let padded_mat = pre_process(&img).unwrap();
    // convert the image to blob input with resizing
    let blob = dnn::blob_from_image(
        &padded_mat,
        1.0 / 255.0,
        core::Size_ {
            width: model_config.input_size,
            height: model_config.input_size,
        },
        core::Scalar::new(0f64, 0f64, 0f64, 0f64),
        true,
        false,
        core::CV_32F,
    )?;
    let out_layer_names = model.get_unconnected_out_layers_names()?;

    let mut outs: Vector<Mat> = Vector::default();
    model.set_input(&blob, "", 1.0, core::Scalar::default())?;
    model.forward(&mut outs, &out_layer_names)?;

    let detections = post_process(&outs, &mat_info, conf_thresh, nms_thresh)?;

    Ok(detections)
}
