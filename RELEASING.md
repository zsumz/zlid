# Releasing

ZLID releases are built from a clean, signed commit and published from the
matching signed tag.

## Candidate

1. Confirm `Cargo.toml`, the conformance fixture release, README examples, and
   `CHANGELOG.md` agree on the version.
2. Run `scripts/check-release` from a clean checkout.
3. Run `cargo audit --deny warnings` with a freshly updated RustSec database.
4. Run `cargo publish --locked --dry-run` with registry access.
5. Review the exact file list and the generated `.crate` under
   `target/package/`.

## Publish

1. Create and verify the signed tag `v<version>`.
2. Push the commit and tag; wait for required CI to pass.
3. Run `cargo publish --locked` only after explicit release approval.
4. Add a second crates.io owner or organization team after the first publish.

## Public proof

After crates.io indexes the release:

1. Download the registry archive and compare its checksum with crates.io.
2. Build a fresh consumer using `zlid = "=<version>"` with no path or Git patch.
3. Confirm the crates.io package page, repository link, README, and license.
4. Confirm `https://docs.rs/zlid/<version>/zlid/` returns the expected API docs.
5. Create the GitHub release from the exact signed tag.

The first release is manual. Configure crates.io trusted publishing only after
the crate exists and the GitHub environment has been reviewed.
