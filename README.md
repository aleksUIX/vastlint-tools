# VASTlint tools

Created by [Aleksander Sekowski](https://github.com/aleksUIX).

OpenAdTech applications and integrations built on the public [VASTlint engine](https://github.com/aleksUIX/vastlint). This repository owns the application source, tests and marketplace release pipelines. The engine and language bindings have their own releases.

Generate a standalone, annotated VAST report with `vastlint explain tag.xml -o report.html`. [Walk through a pause ad report](https://vastlint.org/docs/explain-vast/)

## What the CLI shares

When CLI sharing is enabled, `vastlint check` sends usage statistics and the full XML of checked tags to vastlint.org. We use those tags only for research and to improve VASTlint. At setup, press Enter to enable sharing or type N to opt out. To turn it off later, run `vastlint telemetry off`. [CLI data handling](https://vastlint.org/privacy/#cli-telemetry)

## The applications use a published engine release.

`core-version.txt` records the engine version used by the CLI, MCP server, browser extension, IDE extension and frontend runtime. Their manifests pin published packages rather than a sibling source checkout.

- [Chrome extension](chrome/README.md): page inspection and browser workflows.
- [VS Code extension](vscode/README.md): inline editor validation, also published to Open VSX.
- [CLI](crates/vastlint-cli/README.md): the `vastlint` binary.
- [MCP server](crates/vastlint-mcp/README.md): validation tools for AI agents.
- [VAST client runtime](packages/vastlint-client/README.md): sessions, playback and tracking.
- [React bindings](packages/vastlint-react/README.md): hooks for the client runtime.
- [Browser and React examples](examples): application integration examples.

The `vastlint` CLI lives in this repository and links the published engine. The Rust engine, FFI, WASM binding, gRPC reference server and language bindings remain in the [engine repository](https://github.com/aleksUIX/vastlint) and its binding repositories. Java continues to use the public gRPC server. Erlang's OTP path continues to use the public CLI daemon.

## Build and test the applications.

Use Node.js 22 or later and a Rust toolchain that supports the declared MSRV.

```sh
npm ci
npm run check:core
npm run typecheck
npm test
cargo test --workspace --locked
bash test/mcp-release-smoke.sh
```

The IDE manifest keeps its existing `vastlint` name. Its npm dependency uses the `vastlint-engine` alias to avoid a self-dependency while preserving the marketplace identity.

Each extension has its own lockfile:

```sh
npm --prefix chrome ci
npm --prefix chrome run build
npm --prefix chrome test
npm --prefix vscode ci
npm --prefix vscode test
npm --prefix vscode run bundle
```

Build the MCP container from this repository:

```sh
docker build -f Dockerfile.mcp -t vastlint-mcp .
```

Agent integrations are provided by [plugin.json](plugin.json), [.claude-plugin](.claude-plugin), [gemini-extension.json](gemini-extension.json), [skills](skills) and [agents](agents). Hosted endpoints retain their existing addresses.

## Product releases are independent of engine releases.

Application tags publish the extensions, MCP server and frontend packages. Tags of the form `cli-vX.Y.Z` publish `vastlint-cli`, its binaries, the CLI container image and the Homebrew formula. The engine crate, engine npm package, gRPC server and language bindings stay in the engine repository.

Marketplace publisher accounts and existing extension IDs stay with Aleksander Sekowski. See [RELEASE.md](RELEASE.md) for publishing configuration, versioning and the migration of credentials and trusted publishers.

## Project history and attribution are preserved.

The application history was extracted from the original VASTlint repository. Original authors and commit dates are preserved. [HISTORY.md](HISTORY.md) records the source commit and commit mapping.

Licensed under [Apache 2.0](LICENSE). Preserve the applicable [NOTICE](NOTICE) attribution when redistributing. See [PRIVACY.md](PRIVACY.md) for data handling and [SECURITY.md](SECURITY.md) for security reports.
