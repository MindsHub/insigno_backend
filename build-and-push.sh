#!/bin/bash
podman build . --platform=linux/amd64 -t mindshubalessio/insigno
podman push mindshubalessio/insigno
