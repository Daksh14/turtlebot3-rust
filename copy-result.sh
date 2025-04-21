#!/bin/sh

container_id=$(docker ps -q --filter "ancestor=my-system")

rm -rf ./output-binary/
mkdir -p ./output-binary

# Copy the project files to the container
docker cp $container_id:/rust-example/target/aarch64-unknown-linux-gnu/release/ros2_cmd_vel_publisher ./output-binary/

# Copy into docker container
scp output-binary/ros2_cmd_vel_publisher Yohanse@10.170.8.255:~/rust-example/output-binary/