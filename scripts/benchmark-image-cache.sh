#!/usr/bin/env bash
set -euo pipefail
out=/tmp/wealthfolio-benchmark
mkdir -p "$out"
printf 'variant,elapsed_seconds\n' > "$out/timings.csv"
for variant in unchanged frontend rust; do
  context="$(mktemp -d)"
  git archive HEAD | tar -x -C "$context"
  if [[ "$variant" == frontend ]]; then
    printf '\n/* CI cache probe */\n' >> "$context/apps/frontend/src/globals.css"
  elif [[ "$variant" == rust ]]; then
    printf '\n// CI cache probe\n' >> "$context/apps/server/src/main.rs"
  fi
  start=$SECONDS
  docker buildx build --platform linux/amd64 --progress plain \
    --cache-from type=local,src=/tmp/wealthfolio-build-cache \
    --output type=cacheonly "$context" 2>&1 | tee "$out/$variant.log"
  printf '%s,%s\n' "$variant" "$((SECONDS-start))" >> "$out/timings.csv"
  rm -rf "$context"
done
printf 'Commit: %s\nCompiler: Rust 1.95; workspace opt 0; dependency opt 1; jobs 2\n' "$GITHUB_SHA" > "$out/context.txt"
cat "$out/timings.csv" >> "$GITHUB_STEP_SUMMARY"
