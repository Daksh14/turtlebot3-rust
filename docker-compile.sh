#!/bin/sh

container_id=$(docker ps -q --filter "ancestor=my-system")

rm -rf ./target

# Copy the project files to the container
docker cp ./ $container_id:/rust-example/
