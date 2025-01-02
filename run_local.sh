#!/bin/bash

export RUST_LOG=debug
export CROSS_CONTAINER_OPTS=--add-host=host.docker.internal:host-gateway

cd client
cross run -- host.docker.internal --port 5901 "$@"
