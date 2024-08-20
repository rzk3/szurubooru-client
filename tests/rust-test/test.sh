#!/bin/bash

cargo check
docker compose down
docker compose up -d
cargo run && docker compose down
