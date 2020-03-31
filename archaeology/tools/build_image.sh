#!/bin/bash -e
# Builds the docker image then attemps to upload it.
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null && pwd )"
cd $SCRIPT_DIR

IMAGE_TAG=glalonde/spout_sw_rendering
DOCKER_FILE=Dockerfile

docker build --tag=$IMAGE_TAG --file=$DOCKER_FILE .
docker login --username=glalonde
docker push $IMAGE_TAG \
    || echo "WARNING: Failed to upload docker image to cloud storage"
