#!/bin/bash


# Step 1: Build the initial Docker image
echo "Building the initial Docker image..."
docker build -t my-system -f Dockerfile.system .

# Step 2: Run the container
echo "Running the container..."
container_id=$(docker run -dit my-system)

# Copying project files to the container
docker cp ./ $container_id:/rust-example/
