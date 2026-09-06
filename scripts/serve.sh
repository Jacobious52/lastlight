#!/bin/sh
set -eu
cd "$(dirname "$0")/../dist"
python3 -m http.server "${1:-8080}" --bind 127.0.0.1
