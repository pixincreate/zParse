#!/bin/bash

 TAG="${1:-v1.0.0}"

 git tag "${TAG}" -m "zParse ${TAG}"
 git push origin "${TAG}"
