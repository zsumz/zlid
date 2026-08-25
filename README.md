# zlid

**Ordered ZLIDs, random ZLIDs, and reversible aliases for Rust.**

`zlid` implements the [ZLID v0.1 specification](https://github.com/zlid-io/spec/blob/22669bdac45248e77708a602c8510d2bee39697d/SPECIFICATION.md).
Every ID is 16 bytes with a canonical 26-character text form. The v0.1 wire
format is stable; the Rust API is prerelease and may change before 1.0.

Rust 1.88 or newer is required.

## Install

```toml
[dependencies]
zlid = "=0.0.1-rc.2"
```

## Start

```rust
use zlid::ZLID;

let ordered = ZLID::next_with_partition(42)?;
assert_eq!(ZLID::parse(&ordered.text())?, ordered);

let random = ZLID::random()?;
assert_ne!(random, ordered);

let key = [0x42; 32]; // Demo only; load a high-entropy key from secret storage.
let alias = ordered.alias_str(&key, "users|prod")?;
assert_eq!(alias.unalias_str(&key, "users|prod")?, ordered);

# Ok::<(), zlid::Error>(())
```

Run the complete example from a checkout:

```sh
cargo run --example quickstart
```

## Families

| Family | Use |
| --- | --- |
| ZLID | Time-sortable IDs with deterministic uniqueness per generator, profile, and partition stream |
| ZLID-R | Random IDs backed by operating-system entropy |
| ZLID-A | Keyed, reversible aliases for ordered ZLIDs |

## Contract

The shared `ZLID::next()` generator is synchronized within one process.
Explicit generators are independent; separate writers and processes do not
gain a global deterministic uniqueness guarantee.

ZLID-A is deterministic obfuscation. It is not encryption, authentication, or
a bearer token. Use a high-entropy secret key; decoding with the wrong key is
not detected. Key versioning and rotation belong to the application.

An omitted partition key means the public all-zero key. Partition values are
domain labels, not a security boundary.

## WebAssembly

For `wasm32-unknown-unknown`, enable `wasm-js`:

```toml
[dependencies]
zlid = { version = "=0.0.1-rc.2", features = ["wasm-js"] }
```

## Qualification

```sh
zcheck
scripts/check-release
```

`zcheck` is the canonical development gate. `scripts/check-release` requires a
clean worktree and verifies the packaged crate and an isolated consumer. The
published crate includes the pinned
[ZLID v0.1 conformance snapshot](https://github.com/zsumz/zlid/blob/main/conformance/README.md).

## License

Apache-2.0. See [LICENSE](https://github.com/zsumz/zlid/blob/main/LICENSE).
