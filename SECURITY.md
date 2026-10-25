# Security policy

Report vulnerabilities privately through [GitHub Security Advisories](https://github.com/aleksUIX/vastlint-tools/security/advisories/new). Do not include private ad tags, credentials or personal data in public issues. The maintainer aims to acknowledge reports within 48 hours and credits reporters unless they request anonymity.

## The applications and engine have separate responsibilities.

This repository contains the Chrome and IDE extensions, MCP server, frontend runtime, React integration and agent manifests. Applications use the published engine pinned in `core-version.txt`. Report engine parsing and validation vulnerabilities to [the engine project](https://github.com/aleksUIX/vastlint/security/advisories/new). Coordinate affected application updates when the engine pin changes.

Local IDE validation and Chrome page inspection run on the user's machine. The Chrome popup and hosted MCP integration can open or call hosted services. See [PRIVACY.md](PRIVACY.md) and [the hosted privacy policy](https://vastlint.org/privacy/) for those paths. The local MCP process can fetch URLs when a caller selects URL inspection tools.

## Checks and artifact provenance remain public.

CI runs Rust formatting, Clippy, tests and RustSec checks; complete npm audits; application and example builds; MCP protocol checks; and the MCP container build. CodeQL scans Rust and JavaScript. Engine fuzzing and its public interfaces remain in the engine repository.

The release workflow tests and builds application artifacts before its release approval gate. GitHub releases attach provenance attestations to the MCP archives and extension packages. Preserve the existing publisher IDs and use the exact tested artifacts for marketplace publication. The setup required after this repository split is recorded in [RELEASE.md](RELEASE.md).

Latest application versions receive security fixes. Published engine updates are reviewed and adopted explicitly, independently of the tools version.
