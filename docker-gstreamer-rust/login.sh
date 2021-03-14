#!/usr/bin/env bash
echo $GITHUB_TOKEN | docker login https://ghcr.io -u $GITHUB_USERNAME --password-stdin