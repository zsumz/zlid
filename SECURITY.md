# Security

## Reporting

Please report suspected vulnerabilities privately through GitHub's
**Security > Report a vulnerability** flow for `zlid-io/rust`. Do not open a
public issue for an undisclosed vulnerability.

## Supported versions

The newest `0.1.0-beta.x` release receives security fixes while the crate is in
beta. Older beta releases may be asked to upgrade.

## Boundaries

- ZLID-A is reversible obfuscation, not encryption or authentication.
- Ordered IDs are deterministically unique per coordinated generator, profile,
  and partition stream—not across arbitrary uncoordinated writers.
- ZLID-R is collision-resistant, not a mathematical global-uniqueness proof.
- IDs are identifiers, not secrets, capabilities, or authorization tokens.
