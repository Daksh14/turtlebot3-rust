use apriltag::{Detector, Family, Image};
use apriltag_image::prelude::*;
use std::path::Path;
use dirs;

//Neville Pochara's Apriltag detector.
//This program is designed to receive a camera Frame, which will be converted to a usable file for our detector.
//Will be integrated in our main code later!
//Most of this code was taken from: https://docs.rs/apriltag-image/latest/apriltag_image/index.html
//Likely will only compile on Linux, as that is the environment I used.
//NOTE TO SELF: Add into camera.rs. To send the frame to this file.

fn main(img: Frame) -> anyhow::Result<()> {

    let gray_image = img.decode_image::<Luma8Format>(); //Luma8 converts the image to grayscale, which is what the Apriltags detector needs.
    let mut detector = Detector::builder()
        .add_family_bits(Family::tag_36h11(), 1) //36h11 is the Tag Family we are using.
        .build()?;
    let detections = detector.detect(&gray_image);
    println!("Detections: {:?}", detections);
    Ok(())
}
