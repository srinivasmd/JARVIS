#!/usr/bin/env bash
set -euo pipefail

cargo run -- publish-plugin examples/plugin-manifest.json
