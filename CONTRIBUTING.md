# Contributing

1. Search the manifest tree for the application, upstream source repository, and provider IDs. Keep an existing OpenBits ID stable even when its display name changes.
2. Confirm the upstream publisher's identity, latest stable release, corresponding source and license. Cite the evidence in the manifest and pull request. Prefer immutable release and source references.
3. Verify every provider ID and architecture against that provider's metadata and check that its package belongs to the upstream application. Record provider versions only when independently verified.
4. Add or edit one canonical YAML file. Use two-space indentation, LF line endings, UTF-8, and quotes around version strings. Record unresolved release verification using the review status described in the README.
5. Run the four validation commands in the README. In the pull request, explain the change and include the evidence and check results.

Maintainers must review new publishers and changes to source identity. Automations may propose changes for review. A maintainer must confirm evidence before merging.
