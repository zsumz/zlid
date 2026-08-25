# Security

## Reporting

Please report suspected vulnerabilities privately through GitHub's
**Security > Report a vulnerability** flow for `zsumz/zlid`. Do not open a
public issue for an undisclosed vulnerability.

## Supported versions

The newest `0.0.1-rc.x` release receives security fixes while the crate is a
release candidate. Older release candidates may be asked to upgrade.

## Boundaries

- ZLID-A is reversible obfuscation, not encryption or authentication.
- ZLID-A keys must be high-entropy secrets loaded from secret storage. Decoding
  with the wrong key is not detected; applications own key versioning and
  rotation.
- An omitted partition key is the public all-zero key. Partition values are
  domain labels, not a security boundary.
- Ordered IDs are deterministically unique per coordinated generator, profile,
  and partition stream—not across arbitrary uncoordinated writers.
- ZLID-R is collision-resistant, not a mathematical global-uniqueness proof.
- IDs are identifiers, not secrets, capabilities, or authorization tokens.
