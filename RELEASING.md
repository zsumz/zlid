# Releasing

ZLID releases are built from a clean, signed commit and published from the
matching signed tag.

Publishing a stable version requires explicit maintainer confirmation of both
the version and the stable-release intent.

## Candidate

1. Confirm `Cargo.toml`, README examples, and `CHANGELOG.md` agree on the package
   version. Confirm the pinned conformance fixture source release and checksum.
2. Run `scripts/check-release` from a clean checkout.
3. Run `cargo audit --deny warnings` with a freshly updated RustSec database.
4. Run `cargo publish --locked --dry-run` with registry access.
5. Review the exact file list and the generated `.crate` under
   `target/package/`.

## Publish

1. Push the final commit and wait for required CI on that exact commit.
2. Dispatch the Fuzz and Benchmark workflows for that commit and review their
   retained evidence.
3. Create a signed annotated tag `v<version>`. Verify both the tag object's
   signature and the signature of the commit it resolves to.
4. Push the tag.
5. Run `cargo publish --locked` only after explicit release approval.

## Public proof

After crates.io indexes the release:

1. Download the registry archive and compare its checksum with crates.io.
2. Build a fresh consumer using `zlid = "=<version>"` with no path or Git patch.
3. Confirm the crates.io package page, repository link, README, and license.
4. Confirm `https://docs.rs/zlid/<version>/zlid/` returns the expected API docs.
5. Create the GitHub release from the exact signed tag.

Publication is intentionally manual. Move to crates.io trusted publishing only
with explicit maintainer approval after a protected GitHub release environment
has been configured and reviewed.
