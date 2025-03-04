#!/bin/bash
# File for building and running the official onlinedi.vision
# server. This script is expected to be ran as @root on the server.
# Please be aware of this.

# Installing / Building / Running the landing page.
cd landing
npm install
npm build 
serve -s build > ~/od-logs/ws.log &

# Build / Run the CDN server
cd ../cdn
cargo build --release
./target/cdn > ~/od-logs/cdn.log &


