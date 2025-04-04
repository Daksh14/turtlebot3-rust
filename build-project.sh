#!/bin/sh

OPENCV_LINK_LIBS="opencv_imgcodecs,opencv_imgproc,opencv_face,opencv_objdetect,opencv_dnn,opencv_dnn_objdetect,opencv_core,ippiw,ittnotify,ippicv,liblibprotobuf,z" \
OPENCV_LINK_PATHS=/root/opencv/lib,/root/opencv/lib/opencv4/3rdparty,/usr/lib/aarch64-linux-gnu \
OPENCV_INCLUDE_PATHS=/root/opencv/include/opencv4 \
ORT_LIB_LOCATION=/onnxruntime/build/Linux/Release \
. /opt/ros/jazzy/setup.sh && cargo run --target=aarch64-unknown-linux-gnu --release
