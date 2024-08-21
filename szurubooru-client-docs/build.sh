#!/bin/bash

maturin develop -F python -m ../szurubooru-client/Cargo.toml && mkdocs build
