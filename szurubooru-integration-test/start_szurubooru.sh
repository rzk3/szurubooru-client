#!/bin/bash

#export MOUNT_DATA=szurubooru/data
export SERVER_MOUNT=./szurubooru/server
#export MOUNT_SQL=szurubooru/pgsql
export BASE_URL=http://localhost:9801
export PORT=9801

docker compose down || true

docker compose up -d
