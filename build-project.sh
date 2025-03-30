#!/bin/bash

# setup rust-build by running the build docker file which compiles the project
docker build -t rust-build -f Dockerfile.build .

# run the container and then copy the target
docker run --name rust-container -d rust-build && docker wait rust-container && docker cp rust-container:rust-example/target ./output
