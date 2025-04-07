use apriltag::{Detector, Family, Image};
use apriltag_image::prelude::*;
use std::path::Path;
use dirs;

//Neville Pochara's Apriltag detector.
//This program will simply open up a test apriltag .png file, and give out readings based on it.
//This is simply a test example to demonstrate that Apriltags can be detected by Rust.
//Will be integrated in our main code later!
//Most of this code was taken from: https://docs.rs/apriltag-image/latest/apriltag_image/index.html
//Likely will only compile on Linux, as that is the environment I used.

fn main() -> anyhow::Result<()> {
    let path = "image/apriltag.png";

    //Check if the file exists, but first we print the CWD.
     println!("Current working directory: {:?}", std::env::current_dir()?);
    if !Path::new(path).exists() {
        return Err(anyhow::anyhow!("Image file '{}' not found.", path));
    }

    //Proceed with reading the image, and output information based on it (default should return ID 135).
    let reader = image::io::Reader::open(path)?;
    let image_buf = reader.decode()?.to_luma8(); //Luma8 converts the image to grayscale, which is what the Apriltags detector needs.
    let image = Image::from_image_buffer(&image_buf);
    let mut detector = Detector::builder()
        .add_family_bits(Family::tag_36h11(), 1) //36h11 is the Tag Family we are using.
        .build()?;
    let detections = detector.detect(&image);
    println!("Detections: {:?}", detections);
    Ok(())
}

