# Repository instructions

- Use the ZDEV volume for all work and build output.
- Commit as `zsumz` with a PGP signature.
- Use a bodyless Conventional Commit subject and never add a coauthor.
- Keep production Rust files at or below 300 lines.
- Keep tests in separate files.
- Treat `scripts/check` as the canonical development gate and
  `scripts/check-release` as the publication gate.
