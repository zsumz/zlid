#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/zlid-benchmark-policy.XXXXXX")"
trap 'rm -rf "$tmp_root"' EXIT

printf '%s\n' \
  'deterministic pack ordered 10.00 ns/op MAD 1.00' \
  'advisory ordered explicit 4t 1000 ops/s' \
  > "$tmp_root/baseline.txt"
printf '%s\n' \
  'deterministic pack ordered 13.00 ns/op MAD 1.00' \
  'advisory explicit distinct 4t 700 ops/s MAD 10' \
  > "$tmp_root/regressed.txt"
printf '%s\n' \
  'deterministic pack ordered 11.00 ns/op MAD 1.00' \
  'advisory explicit distinct 4t 900 ops/s MAD 10' \
  > "$tmp_root/acceptable.txt"

regressed="$(
  BENCHMARK_REGRESSION_RATIO=1.20 \
    bash "$ROOT/scripts/compare-benchmarks" \
      "$tmp_root/baseline.txt" "$tmp_root/regressed.txt"
)"
test "$(grep -c '^::warning ' <<<"$regressed")" -eq 2
grep -Fq 'Compared 2 equivalent lane(s); 2 exceeded' <<<"$regressed"

acceptable="$(
  BENCHMARK_REGRESSION_RATIO=1.20 \
    bash "$ROOT/scripts/compare-benchmarks" \
      "$tmp_root/baseline.txt" "$tmp_root/acceptable.txt"
)"
test "$(grep -c '^::warning ' <<<"$acceptable" || true)" -eq 0
grep -Fq 'Compared 2 equivalent lane(s); 0 exceeded' <<<"$acceptable"

echo "Benchmark comparison checks passed"
