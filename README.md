# Install rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Cross compiling for aarch64-unknown-linux-gnu
To setup on your system, please follow the contents of `Dockerfile.system`

# Cross compiling opencv and statically linking the libraries

The `Dockerfile.system` has steps to setup the system which can compile this rust project, please follow the commands in the dockerfile to achieve the configuration, you can also use the docker file and not install anything on your host system

# Scripts
- `build-project.sh` Build the project and cross compile binary which is ready to be run on the raspberry pi of the turtlebot3
- `copy-result.sh` Copy binary file from docker container to your host system to push to github or use in turtlebot
- `docker-compile.sh` Update the codebase from the Host to the docker container so the docker container can compile the latest code
- `build-image.sh` Build the system required for compilng the project and run the docker container

# Result
The resultin binary is around 1.5 megabytes and multithreaded, we can further try to reduce size by enabling fat LTO and by asking LLVM to prioritise small binary size over optimizations
