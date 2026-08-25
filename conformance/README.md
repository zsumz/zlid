# Conformance snapshot

This directory vendors the ZLID v0.1 golden dataset used by the Rust SDK. The
fixture is part of the published crate so its tests remain self-contained after
download from crates.io.

- Specification: <https://github.com/zlid-io/spec>
- Fixture release: `0.1.0-beta.2`
- Extraction source: ZLID monorepo commit `4f90f76a183d2c9d508badfd99ec465758f777da`

The handwritten validators under `tests/conformance/schema*.rs` deliberately
lock this exact snapshot. They mirror
`conformance/schema/zlid-v0.1-golden.schema.json` from that monorepo commit;
the canonical schema blob is `fdec380a0dd653e28198841050026805ce69891f`.
This validator/parser targets the canonical frozen fixture encoding; it is not a
general-purpose JSON Schema draft-07 engine.

`zlid-v0.1-golden.sha256` records the fixture digest, and a crate unit test pins
the same digest in Rust so changing the fixture and sidecar together is still
detected. A fixture change must be coordinated with the public specification,
update both pins, and pass the complete conformance suite.
