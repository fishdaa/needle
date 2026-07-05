#!/usr/bin/env bash

set -euo pipefail

channel="${1:-stable}"

extra_args=(--locked)
if [[ "$channel" == "beta" ]]; then
  extra_args+=(--allow-dirty)
fi

cargo publish -p toge-core "${extra_args[@]}"
sleep 10
cargo publish -p toge "${extra_args[@]}"
sleep 10
cargo publish -p toged "${extra_args[@]}"
