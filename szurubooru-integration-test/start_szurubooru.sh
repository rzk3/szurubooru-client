#!/bin/bash

export MOUNT_DATA=szurubooru/data
export SERVER_MOUNT=szurubooru/server
export MOUNT_SQL=szurubooru/pgsql
export BASE_URL=http://localhost:5000
export PORT=5000

docker compose up -d
