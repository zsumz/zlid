# zlid

[![Crates.io](https://img.shields.io/crates/v/zlid.svg)](https://crates.io/crates/zlid)
[![docs.rs](https://docs.rs/zlid/badge.svg)](https://docs.rs/zlid)
[![CI](https://github.com/zlid-io/rust/actions/workflows/ci.yml/badge.svg)](https://github.com/zlid-io/rust/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://github.com/zlid-io/rust/blob/main/LICENSE)

Canonical, sortable, collision-resistant identifiers with deterministic
per-stream uniqueness.

`zlid` is the Rust implementation of the
[ZLID v0.1 specification](https://github.com/zlid-io/spec). Every identifier is
a 16-byte value with a canonical 26-character text form.

This crate is a beta. Its wire format is stable within ZLID v0.1, while the
Rust API may still be refined before 1.0.

## Install

```toml
[dependencies]
zlid = "0.1.0-beta.2"
```

## Quick start

```rust
use zlid::{Inspection, Profile, Zlid};

fn main() -> zlid::Result<()> {
    let ordered = Zlid::next_with_partition(42)?;
    println!("{ordered}");

    let mut generator = Zlid::generator(Profile::HighThroughput, 42);
    let next = generator.next()?;

    if let Inspection::Ordered { sequence, .. } = next.inspect() {
        println!("sequence: {sequence}");
    }

    let parsed = Zlid::parse("01k2r7-kfwe58 07000000000001")?;
    assert_eq!(parsed.text(), "01K2R7KFWE5807000000000001");

    Ok(())
}
```

The executable version lives in [`examples/quickstart.rs`](examples/quickstart.rs):

```sh
cargo run --example quickstart
```

## Identifier families

| Family | Use |
| --- | --- |
| ZLID | Time-sortable IDs with deterministic uniqueness per generator, profile, and partition stream |
| ZLID-R | Random IDs backed by operating-system entropy |
| ZLID-A | Keyed, reversible aliases for ordered ZLIDs |

ZLID-A is deterministic obfuscation. It is not encryption, authentication, or
a bearer token.

## Core operations

```rust
use zlid::Zlid;

let id = Zlid::parse("01K2R7KFWE5807000000000001")?;
let bytes = id.bytes();
assert_eq!(Zlid::from_bytes(&bytes)?, id);

let key = [0, 1, 2, 3];
let alias = id.alias_str(&key, "users|prod")?;
assert_eq!(alias.unalias_str(&key, "users|prod")?, id);

let partition = Zlid::partition_str("tenant:acme", None)?;
assert_eq!(partition, 23);

# Ok::<(), zlid::Error>(())
```

The shared `Zlid::next()` generator is synchronized within one process.
Explicit generators are independent streams; uncoordinated writers do not gain
a global deterministic uniqueness guarantee.

## Platforms

The crate supports Rust 1.88 and newer on the native platforms supported by
[`getrandom`](https://docs.rs/getrandom). For browser-style
`wasm32-unknown-unknown` targets, enable the `wasm-js` feature:

```toml
zlid = { version = "0.1.0-beta.2", features = ["wasm-js"] }
```

The crate itself contains no `unsafe` Rust.

## Conformance

The published crate contains its ZLID v0.1 fixture and coverage manifest. The
same tests therefore run from a checkout, an unpacked `.crate`, and docs.rs
source archives.

```sh
scripts/check
```

`scripts/check-release` adds a clean-tree requirement, tests the exact packaged
archive, and runs an isolated downstream consumer.

## License

Apache License 2.0. See the repository's
[LICENSE](https://github.com/zlid-io/rust/blob/main/LICENSE).
