#!/bin/bash
set -e

pip uninstall -y szurubooru_client || true
pip install -r requirements.txt
maturin develop -F python -m ../../szurubooru-client/Cargo.toml
docker compose down
docker compose up -d
python test.py && docker compose down
