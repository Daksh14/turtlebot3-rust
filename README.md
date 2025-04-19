# Turtle bot 3 simulation 

Simulation software for the turtleobot3 https://emanual.robotis.com/docs/en/platform/turtlebot3/overview/ (BURGER)

The script requires the bringup script to be executed 
```
ros2 launch turtlebot3_bringup robot.launch.py
```

```
ros2 topic list
```

Make sure you see the `/cmd_vel` and `/scan` topic

If your turtlebot system has cargo and rust installed then 
```
git clone https://github.com/Daksh14/turtlebot3-rust.git
cd turtlebot3-rust

cargo b --release && ./target/release/ros2_cmd_vel_publisher
```
If your turtlebot doesnt have rust installed you will have to cross compile the binary

# Cross Compiling the binary

## Install rust https://www.rust-lang.org/tools/install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Cross compiling for aarch64-unknown-linux-gnu
To setup on your system, please follow the contents of `Dockerfile.system`

# Cross compiling opencv and statically linking the libraries

The `Dockerfile.system` has steps to setup the system which can compile this rust project, please follow the commands in the dockerfile to achieve the configuration, you can also use the docker file and not install anything on your host system

# Scripts
- `build-project.sh` Source ros jazzy, static library and cross compile build.
- `copy-result.sh` Copy binary file from docker container to your host system to push to github or use in turtlebot. Requires `./build-image.sh` to be run before (with docker desktop https://www.docker.com/products/docker-desktop/)
- `docker-compile.sh` Update the codebase from the Host to the docker container so the docker container can compile the latest code (This allows editing on your host machine and copying your host machine project into the docker container project which can compile the binary)
- `build-image.sh` Build the docker system image required for compilng the project binar and run the docker container 

# Result
```diff
-The resultin binary is around 1.5 megabytes and multithreaded, we can further try to reduce size by enabling fat LTO and by asking LLVM to prioritise small binary size over optimizations
+the new binary is around 25.0 megabytes possiblity because of statically linking the onnx runtime.
```
