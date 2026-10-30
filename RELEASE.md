# Release the tools from their own repository.

Created by Aleksander Sekowski.

## Keep engine and application versions separate.

`core-version.txt` pins the published engine used by the applications. The same exact version appears in the Rust MCP dependency and the npm dependencies of the extensions, client runtime and examples. Run `npm run check:core` after updating these pins and regenerate the affected Cargo and npm lockfiles.

Tools tags use `vMAJOR.MINOR.PATCH`. The release workflow stamps the application and MCP package versions without changing their engine dependency. Client and React packages receive the tools version. React depends on the corresponding client release.

## Verify a release before tagging it.

Run the Rust tests, formatting and Clippy checks, the frontend typecheck and tests, publication recovery tests (`npm run test:release`), the MCP protocol smoke tests, both extension builds and tests, and all example builds. CI and release workflows retain these checks.

Release builds consume published VASTlint packages. They do not build an engine from a private or sibling checkout.

## Configure publishing in this repository.

The release environments are configured: `production` requires approval from `aleksUIX` and permits self-review; `patch` has no reviewer gate. Publisher credentials, switches and npm trusted publishing still need configuration before the first tools release. A workflow environment name alone does not configure an approval rule.

Repository secrets and npm trusted-publisher settings do not transfer automatically with source files. Configure the retained accounts for `aleksUIX/vastlint-tools` before enabling publication.

- `ENABLE_NPM_PUBLISH=true`: publish `vastlint-client` and `vastlint-react`. Configure npm trusted publishing for this repository and `release.yml`, or provide `NPM_TOKEN`.
- `ENABLE_VSCODE_PUBLISH=true`: configure `VSCE_PAT` and `OVSX_TOKEN` for the existing publisher and Open VSX account.
- `ENABLE_CHROME_PUBLISH=true`: configure `CHROME_EXTENSION_ID`, `CHROME_CLIENT_ID`, `CHROME_CLIENT_SECRET` and `CHROME_REFRESH_TOKEN`. Retain the existing extension ID.
- `ENABLE_MCP_REGISTRY_PUBLISH=true`: GitHub OIDC publishes under the existing `io.github.aleksUIX/vastlint` namespace. The owner namespace stays unchanged; new binary URLs point to this repository.
- `ENABLE_SMITHERY_PUBLISH=true`: maintain the existing `aleksander/vastlint` Smithery listing. The release job checks the public listing and refreshes it through the CLI before October 31, 2026, using `SMITHERY_API_KEY`. Smithery's name is separate from the MCP Registry name `io.github.aleksUIX/vastlint`.

Smithery [retires its proxy, API, API keys and CLI on October 31, 2026](https://smithery.ai/future), while keeping its public directory. The Smithery release job skips CLI publication from that date and continues checking the listing. Clients connect directly to `https://vastlint.org/mcp`. Keep the existing listing and its verification; do not create a second server. The current listing metadata is recorded in `docs/smithery-listing.json`.

The standalone Chrome workflow builds without publishing on pushes and pull requests. Its manual `publish` input remains available after credentials are configured.

Preserve the prior release approval policy: patch releases use the `patch` environment; initial, minor and major releases use `production`. Configure the appropriate environment protection before enabling publishing.

## Build artifacts before publishing them.

The release workflow runs MCP protocol checks, builds the four native MCP archives, tests and stages frontend packages, and builds the Chrome archive and VSIX. The release gate precedes publication. GitHub provenance covers the packaged application artifacts. Marketplace publication uses the built extension artifacts.

The engine npm package, crates.io engine and CLI crates, gRPC image, native FFI/NIF artifacts, binding synchronization, Homebrew synchronization and engine-to-infrastructure synchronization remain in the engine release workflow.

A tools contribution can be agreed separately in the future. Keeping this repository public does not transfer its publisher accounts or assign ownership of the tools to another organization.

Keep the versions in `.claude-plugin/plugin.json`, `plugin.json` and `gemini-extension.json` aligned with the intended tools version before tagging. Historical binaries remain available in the engine repository releases; the first tools release creates the new download location.
