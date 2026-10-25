#!/usr/bin/env bash
# Build and package the IDE extension using its pinned public engine.
set -euo pipefail
cd "$(dirname "$0")/.."
npm run check:core
npm --prefix vscode ci
npm --prefix vscode test
npm --prefix vscode run bundle
node - <<'JS'
const fs = require('node:fs');
const engine = require('./vscode/out/deps/vastlint/index.cjs');
const xml = fs.readFileSync('test/fixtures/warn_linear_no_quartile_tracking.xml', 'utf8');
const result = engine.validate(xml);
if (!result.issues.some(issue => issue.id === 'VAST-2.0-linear-tracking-quartiles')) {
  throw new Error('Expected quartile tracking warning in the bundled engine');
}
console.log('Bundled engine fixture check passed.');
JS
mkdir -p .release
(cd vscode && npx --no-install vsce package --no-update-package-json --out ../.release/vastlint-local.vsix)
printf '%s\n' 'Install .release/vastlint-local.vsix with Extensions: Install from VSIX.'
printf '%s\n' 'Open test/fixtures/warn_linear_no_quartile_tracking.xml and verify the warning.'
