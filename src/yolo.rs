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
    let file = File::open("../data/config.json"); // change the path if needed
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

fn post_process(
    outs: &Vector<Mat>,
    mat_info: &MatInfo,
    conf_thresh: f32,
    nms_thresh: f32,
) -> opencv::Result<Detections> {
    // outs: tensor float32[1, M, 8400]  M = 4 + the number of classes， 8400 anchors
    let dets = outs.get(0).unwrap(); // remove the outermost dimension
                                     // dets: 1xMx8400   1 x [x_center, y_center, width, height, class_0_conf, class_1_conf, ...] x 8400
    let rows = *dets.mat_size().get(2).unwrap(); // 8400
    let cols = *dets.mat_size().get(1).unwrap(); // M

    let mut boxes: Vector<Rect> = Vector::default();
    let mut scores: Vector<f32> = Vector::default();
    let mut indices: Vector<i32> = Vector::default();
    let mut class_index_list: Vector<i32> = Vector::default();
    let x_scale = mat_info.width / mat_info.scaled_size;
    let y_scale = mat_info.height / mat_info.scaled_size;

    // Iterate over all detections/anchors and get the maximum class confidence score and its index
    // To understand it better, I iterate over all anchors using the for loop.
    // In practice, it's recommended to use the function `opencv::core::min_max_loc()` to get the maximum score and its index. easy to use.
    for row in 0..rows {
        // 8400 anchors
        let mut vec = Vec::new();
        let mut max_score = 0f32;
        let mut max_index = 0;
        for col in 0..cols {
            // [x_center, y_center, width, height, class_0_conf, class_1_conf, ...]
            // first 4 values are x_center, y_center, width, height
            let value: f32 = *dets.at_3d::<f32>(0, col, row)?; // (1 x M x 8400)
            if col > 3 {
                // the rest (after 4th) values are class scores
                if value > max_score {
                    max_score = value;
                    max_index = col - 4;
                }
            }
            vec.push(value);
        }
        // thresholding by score
        if max_score > 0.25 {
            scores.push(max_score);
            class_index_list.push(max_index as i32);
            let cx = vec[0];
            let cy = vec[1];
            let w = vec[2];
            let h = vec[3];
            boxes.push(Rect {
                x: (((cx) - (w) / 2.0) * x_scale).round() as i32,
                y: (((cy) - (h) / 2.0) * y_scale).round() as i32,
                width: (w * x_scale).round() as i32,
                height: (h * y_scale).round() as i32,
            });
            indices.push(row as i32);
        }
    }
    // do NMS
    dnn::nms_boxes(
        &boxes,
        &scores,
        conf_thresh,
        nms_thresh,
        &mut indices,
        1.0,
        0,
    )?;

    let mut final_boxes: Vec<BoxDetection> = Vec::default();

    for i in &indices {
        let class = class_index_list.get(i as usize)?;
        let rect = boxes.get(i as usize)?;

        let bbox = BoxDetection {
            xmin: rect.x,
            ymin: rect.y,
            xmax: rect.x + rect.width,
            ymax: rect.y + rect.height,
            conf: scores.get(i as usize)?,
            class: class,
        };

        final_boxes.push(bbox);
    }

    Ok(Detections {
        detections: final_boxes,
    })
}

pub fn draw_predictions(img: &mut Mat, detections: &Detections, model_config: &ModelConfig) {
    let boxes = &detections.detections;

    for i in 0..boxes.len() {
        let bbox = &boxes[i];
        let rect = Rect::new(
            bbox.xmin,
            bbox.ymin,
            bbox.xmax - bbox.xmin,
            bbox.ymax - bbox.ymin,
        );
        let label = model_config.class_names.get(bbox.class as usize).unwrap();

        // change according to your needs
        if label == "person" {
            let box_color = core::Scalar::new(0.0, 255.0, 0.0, 0.0); // green color
            opencv::imgproc::rectangle(img, rect, box_color, 2, opencv::imgproc::LINE_8, 0)
                .unwrap();
        } else if label == "bicycle" {
            let box_color = core::Scalar::new(0.0, 165.0, 255.0, 0.0); // orange color
            opencv::imgproc::rectangle(img, rect, box_color, 2, opencv::imgproc::LINE_8, 0)
                .unwrap();
        }
        // ... other classes
    }
}
