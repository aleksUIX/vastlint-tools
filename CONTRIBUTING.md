# Contributing to VASTlint tools

This repository contains the applications and integrations created by Aleksander Sekowski. Engine validation rules, public language bindings and reference transports are maintained in [vastlint](https://github.com/aleksUIX/vastlint) and its binding repositories.

## Set up the applications.

Use Node.js 22 or later and Rust 1.91 or later. Start with:

```sh
git clone https://github.com/aleksUIX/vastlint-tools
cd vastlint-tools
npm ci
npm run check:core
npm run typecheck
npm test
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --locked
bash test/mcp-release-smoke.sh
```

Each extension and example has its own lockfile. Run `npm ci` in the directory being changed. Build and test Chrome with `npm run build` and `npm test`; build and test the IDE extension with `npm test` and `npm run bundle`. Build each frontend example with `npm run build`.

Applications consume the engine version pinned in `core-version.txt`. Keep that pin and all consumer manifests and lockfiles coordinated. A tool release may use a different version from its engine dependency. See [RELEASE.md](RELEASE.md) for publishing.

## Submit changes with attribution and tests.

Every commit must include a `Signed-off-by` line certifying the [Developer Certificate of Origin](https://developercertificate.org/). Use `git commit -s`. There is no separate project CLA.

Include regression coverage for behavior changes. Keep application tests, fixture checks, security audits, marketplace identities and release destinations intact. Update the relevant README when behavior or installation changes. The IDE catalog documentation test compares its rule count and severity totals with the pinned public engine.

Report bugs with the application and engine versions, a minimal reproduction, and expected and actual results. Send engine rule changes to the engine repository. Report security issues privately as described in [SECURITY.md](SECURITY.md). Follow [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) and [GOVERNANCE.md](GOVERNANCE.md).
