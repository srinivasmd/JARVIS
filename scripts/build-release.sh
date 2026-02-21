#!/usr/bin/env bash
set -euo pipefail

cargo build --release
printf 'release binary: target/release/jarvis\n'
