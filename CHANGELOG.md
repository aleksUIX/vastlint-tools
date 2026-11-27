# Changelog

## CLI 0.17.0 (2026-11-26)

- Run `vastlint explain tag.xml -o report.html` to generate a standalone report with annotated XML, source-linked findings, a partner handoff, and a trafficking review for each ad.
- Preview declared images, video, and audio from their XML rows. Use `--embed-asset URL FILE` to include local media for offline review.

## [0.16.4]

- Move `vastlint-cli` here and pin it, with the other applications, to published VASTlint 0.16.7.
- Publish the CLI from `cli-vX.Y.Z` tags: the crate, release binaries, container image and Homebrew formula.

## [0.16.3]

- Extract the public application source and its development history from `aleksUIX/vastlint`.
- Keep engine, bindings, CLI daemon and gRPC releases in the engine repository.
- Pin tools to published VASTlint 0.16.6 and give application releases independent version tags.
- Preserve Aleksander Sekowski's creator attribution and the existing marketplace identities.
