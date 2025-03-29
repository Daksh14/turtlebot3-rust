#!/bin/bash


# Step 1: Build the initial Docker image
echo "Building the initial Docker image..."
docker build -t my-system -f Dockerfile.system .

# Step 2: Run the container
echo "Running the container..."
container_id=$(docker run -dit my-system)

# Step 3: Commit the container state
echo "Committing the container state..."
docker commit "$container_id" my-system-image

# Saving container state as tar
docker save my-system-image > my-system-image.tar
