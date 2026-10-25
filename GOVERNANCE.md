# Governance

VASTlint tools were created by Aleksander Sekowski ([@aleksUIX](https://github.com/aleksUIX)), the current maintainer.

## The maintainer stewards the applications.

This public repository contains the Chrome and IDE extensions, MCP product, frontend runtime, React integration, examples and agent integrations. The maintainer reviews contributions, sets application direction, coordinates security fixes and publishes releases. Community feedback is welcome through issues and pull requests. Contributors follow [CONTRIBUTING.md](CONTRIBUTING.md) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

The engine, language bindings and reference transports have separate repositories and release responsibilities. A proposal to contribute the engine does not itself transfer the applications, marketplace listings or hosted operations. Any future contribution of these tools requires its own agreed scope.

## Publishing uses the existing accounts.

Aleksander Sekowski retains the existing Chrome Web Store, VS Code Marketplace and Open VSX publisher accounts and extension identities. Hosted endpoints retain their addresses. Client and React packages and the MCP registry listing retain their package names. Credentials and npm trusted-publisher settings must be configured for this repository as described in [RELEASE.md](RELEASE.md).

Initial, minor and major tools releases use the production approval environment. Patch releases retain the patch environment. The engine version is pinned separately; application releases do not publish engine packages or binding artifacts.

## Builds and source history remain available.

The source, tests and release workflows are public under Apache 2.0. Their [history](HISTORY.md), [license](LICENSE) and [NOTICE](NOTICE) preserve authorship. Builds use published engine packages and do not require a sibling engine checkout. Publishing requires access to the corresponding account credentials.

The project currently has one maintainer. Co-maintainer access and shared credential custody require an explicit agreement with the maintainer.
