# The tools retain their original development history.

VASTlint and its companion tools were created by Aleksander Sekowski (GitHub: [aleksUIX](https://github.com/aleksUIX)).

The tools repository was extracted from [aleksUIX/vastlint](https://github.com/aleksUIX/vastlint) at source commit [ce64a093a44420b131d309739916321cd6314bd2](https://github.com/aleksUIX/vastlint/commit/ce64a093a44420b131d309739916321cd6314bd2).

Git fast-export and fast-import retained the selected paths, their authors and their commit dates. The resulting commit hashes differ because each commit contains a smaller tree. The source repository's existing history was not rewritten.

The [history map](docs/history-map.json) relates imported commit hashes to source commit hashes. The migration commit applies the current attribution edits, replaces sibling source dependencies with published SDK pins and separates the release workflows.

The moved application source includes Chrome and VS Code extensions, the MCP server, frontend runtime packages, agent integration manifests and examples. Core implementation and reference transports remain in the engine repository.

Historical source links and prior engine releases remain useful provenance for versions released before the split. The existing publisher accounts, extension IDs, package names and MCP Registry name are preserved.

The migration also preserves the application version and publisher dependency updates from source commit `0add5d6` (the concurrent engine v0.16.0 change). Applications keep the working published 0.15.3 engine pin until an explicit SDK update.
