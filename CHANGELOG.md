# Changelog

## CLI 0.19.0 (2026-11-26)

- Follow links beside annotated VAST declarations to relevant reading on vastlint.org. The links contain only public topic names.
- Review the full XML or scroll individual source lines, preview ad behavior in the QA player, and compare declared CTV placement and position with the booking.

## CLI 0.18.0 (2026-11-26)

- Setup asks whether to send usage statistics and full tag XML. Press Enter to enable both or type N to opt out. Tags are used only for research and to improve VASTlint. [CLI data handling](https://vastlint.org/privacy/#cli-telemetry)
- Change the saved choice with `vastlint telemetry off` or `vastlint telemetry on`. Existing usage-only settings continue to cover statistics only.

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
