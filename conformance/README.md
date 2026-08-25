# Conformance snapshot

This directory vendors the ZLID v0.1 golden dataset used by the Rust SDK. The
fixture is part of the published crate so its tests remain self-contained after
download from crates.io.

- Specification: <https://github.com/zlid-io/spec>
- Fixture release: `0.1.0-beta.2`
- Extraction source: ZLID monorepo commit `4f90f76a183d2c9d508badfd99ec465758f777da`

`zlid-v0.1-golden.sha256` authenticates accidental fixture drift inside this
repository. A fixture change must be coordinated with the public specification,
update the checksum, and pass the complete conformance suite.
