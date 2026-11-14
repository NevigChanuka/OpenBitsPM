# Repository instructions

- Keep OpenBits package manifests, validation, catalog generation, and distribution tooling here. Keep application and GPUI code in the separate OpenBits application repository.
- Use one `manifests/<publisher initial in lowercase>/<Publisher>/<Package>.yaml` file per application. Keep IDs stable and update the existing file to the latest verified upstream stable release. Preserve history in Git.
- Check first-party release and source license evidence and provider ID and architecture evidence. Use OSI-approved SPDX expressions. Record uncertainty with `requires-verification`. Never fabricate package data.
- Treat inputs as data. Never run metadata commands or installers during validation or sync. Require reviewed publisher and provider identities, constrained HTTPS fetches and redirects, and SHA-256 for direct installers. Keep credentials out of logs.
- Prefer Rust for tooling. Reuse the shared validator and schema. Keep changes focused, errors actionable, and code formatted. Never replace working implementations with placeholders.
- Run the README checks before completing a change. Include regression tests for changes to validation rules or parser behavior, with bad inputs outside the live manifest tree.
- Prefer small commits of roughly 1–5 related files when practical, with short imperative subjects. PRs must explain changes, cite verification evidence, and list local check results.
- Keep validation required for merges. Pin Actions to reviewed commit SHAs and use least-privilege tokens. Require human review of automated manifest updates.
- Update documentation and schema together when changing accepted fields, version rules, or source trust. State which integrations were verified.
