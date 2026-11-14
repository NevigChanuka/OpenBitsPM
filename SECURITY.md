# Security

Report suspected package impersonation, compromised links, or validator vulnerabilities privately through the repository's GitHub security reporting channel once it is enabled. Provide the manifest path, affected source, and how maintainers can confirm the issue. Keep credentials and private exploit details out of public issues.

## Trust and validation

- Treat manifests and external responses as untrusted data. Never execute commands from metadata or run installers during review, validation, or synchronization.
- Maintainers review upstream publisher and provider namespace ownership before approving a package. Review changes to source identity, repository transfers, provider package IDs, and direct installer origins.
- Any component that fetches metadata or installer bytes must enforce HTTPS, reject credentials, and validate each redirect target against reviewed origins. Apply timeouts and size limits. Keep API tokens scoped to the intended host and out of logs.
- Verify downloaded installer bytes against the recorded SHA-256 before use. A matching hash proves the bytes agree with the recorded digest. Publisher authenticity comes from verified provenance, such as a trusted publisher signature or an independently authenticated release channel.
- The validator checks data and URL structure offline. Evidence links require maintainer review for authenticity, freshness, and resolver behavior.
- Before publishing a catalog, define how the desktop client authenticates the catalog issuer using a pinned verification identity. The digest check protects integrity, and authentication must cover the digest or payload through a trusted signature or attestation.
