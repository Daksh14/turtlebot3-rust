#!/bin/sh

ORT_LIB_LOCATION=/onnxruntime/build/Linux/Release \
. /opt/ros/jazzy/setup.sh && cargo build --target=aarch64-unknown-linux-gnu --release
