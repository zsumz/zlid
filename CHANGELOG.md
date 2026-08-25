# Changelog

All notable changes to this project are documented here.

## 0.0.1-rc.4 - 2026-08-25

- Finalized `ZLID` as the sole value type and reorganized low-level wire and
  injectable generator seams under `wire` and `advanced` modules.
- Added semantic non-exhaustive errors, explicit representation constants,
  strict canonical parsing, and allocation-free family classification.
- Added scheduled fuzzing and same-runner release benchmark evidence while
  preserving every ZLID v0.1 wire byte and generator-state transition.
- Documented deterministic alias linkability, visible alias tag metadata, and
  the current Node.js WebAssembly runtime evidence boundary.

## 0.0.1-rc.3 - 2026-08-25

- Replaced allocation-heavy alias hashing with streaming RustCrypto
  HMAC-SHA256 while preserving published outputs and ZLID v0.1 vectors.
- Removed steady-state heap allocation from valid parsing, ordered random
  tails, aliasing, and formatting through `Display` and `Debug`.
- Reduced shared-generator contention by drawing system entropy before locking
  stream state while keeping clock, sequence, packing, and commit atomic.
- Added byte-exact legacy crypto oracles, allocation budgets, and a packaged
  hot-path benchmark.

## 0.0.1-rc.2 - 2026-08-25

- Made `ZLID` the primary Rust type while retaining `Zlid` as a compatibility
  alias.
- Added a JavaScript-backed system clock and executable Node.js qualification
  for `wasm32-unknown-unknown`.
- Added independent HMAC-SHA256 and SipHash-2-4 reference vectors plus
  deterministic wire, alias, state-machine, and concurrency properties.
- Pinned the exact normative ZLID v0.1 specification commit and blob.

## 0.0.1-rc.1 - 2026-08-25

- Added the complete Rust SDK for the ZLID v0.1 ordered, random, alias,
  partition, comparison, sentinel, opaque, and inspection surfaces.
- Added shared conformance coverage for parsing and generator behavior.
- Added cross-platform operating-system entropy through `getrandom`.
- Declared Rust 1.88 as the minimum supported Rust version.
- Made the published crate self-contained, including its license, conformance
  fixture, coverage manifest, executable example, and archive-level tests.
- Added strict local, CI, packaging, and release verification gates.
