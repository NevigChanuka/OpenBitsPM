# OpenBits package repository

This repository maintains the latest verified stable release of each open-source desktop application in the OpenBits catalog. The Rust and GPUI desktop application lives in the separate `openbits` repository.

## Validate locally

Install Rust through rustup, then run these commands from the repository root. The toolchain and crate dependencies are pinned.

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo run --locked -p openbits-validate
```

To validate a different manifest tree, pass its path after `--`. The validator reads local files and reports the first invalid file or catalog conflict with a nonzero exit code. It returns verified records sorted by package ID and reports packages requiring review.

## Manifest format

The [v1 JSON Schema](schemas/package.schema.json) defines the YAML fields. Each ID has two segments, `Publisher.Package`. Its manifest lives at `manifests/<lowercase publisher initial>/<Publisher>/<Package>.yaml`. Change the same file when a release is updated and use Git history for previous versions.

- `version` is a quoted upstream stable version. V1 accepts one to six dot-separated numeric components. Confirm release status with upstream metadata even when the version looks stable. A new versioning convention needs an explicit schema and policy change.
- `status: verified` requires release and license evidence. When the latest release cannot be confirmed, use `status: requires-verification` with `review_reason` and omit `version` and `verification`. A catalog consumer receives only the validator's verified records.
- Platform bindings identify install methods by OS, CPU architecture, provider, and package ID. V1 models WinGet, Scoop, Chocolatey, Flatpak, Homebrew, and GitHub Releases. The initial records use WinGet on Windows x64.
- `provider_source` links to evidence for a provider binding. An optional `provider_version` records an independently verified provider version. The desktop application should query the provider for current availability and installed versions.
- GitHub Releases bindings model OpenBits-managed downloads and require an installer URL and SHA-256. Delegated bindings identify a package by its provider ID.
- SPDX expressions must use OSI-approved license IDs in every branch. V1 reserves SPDX exceptions and custom license references for future policy.

## Milestones

Foundation and validation cover the manifests, schema, Rust validator, tests, and validation workflow. The next milestone adds a deterministic JSON and gzip generator with SHA-256 checksums and separately handled generation timestamps. Synchronization and daily update PRs follow, then release publishing and the desktop integration contract. Catalog files, sync and publish workflows, and a public endpoint will be introduced in those milestones.

See [CONTRIBUTING.md](CONTRIBUTING.md) for package review and [SECURITY.md](SECURITY.md) for trust requirements.
