# Contributing

Thank you for improving the Rust implementation of ZLID.

## Development

Use Rust 1.88 or newer with zrail 0.0.2 and zcheck 0.0.2 on `PATH`. Run the
canonical gate before opening a pull request:

```sh
zcheck
```

Production Rust files have a 300-line ceiling, tests live in separate files,
and `unsafe`, `todo!`, and `unimplemented!` are rejected mechanically.

## Conformance changes

The fixture under `conformance/` represents the public ZLID specification. Do
not change expected wire behavior in this repository alone. Coordinate the
specification change first, update the fixture and checksum together, and add a
focused regression case.

## Pull requests

Keep changes narrow, use Conventional Commit subjects, and explain compatibility
or wire-format consequences explicitly. Every pull request must pass the MSRV,
stable, macOS, Windows, package, and downstream-consumer checks.
