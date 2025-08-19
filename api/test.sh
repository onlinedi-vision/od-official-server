#!/usr/bin/env bash
#
# Pre-push hook for Online Division development.
# Component:
#
#   https://github.com/onlinedi-vision/od-official-server
#

if ! cd $(git rev-parse --show-toplevel)/api && cargo build; then
  echo "[X] pre-push build test FAILED" >&2
  echo "    make sure this component builds successfully before pushing!"
  exit 1
fi
    
