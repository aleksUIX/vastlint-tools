import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const version = fs.readFileSync(path.join(root, 'core-version.txt'), 'utf8').trim();
assert.match(version, /^\d+\.\d+\.\d+$/);
const manifest = fs.readFileSync(path.join(root, 'crates/vastlint-mcp/Cargo.toml'), 'utf8');
assert.match(manifest, new RegExp(`^vastlint-core = "=${version.replaceAll('.', '\\.')}"$`, 'm'));
for (const directory of ['chrome', 'packages/vastlint-client',
  'examples/vastlint-client-browser', 'examples/vastlint-client-react-scratch', 'examples/vastlint-react-demo']) {
  const data = JSON.parse(fs.readFileSync(path.join(root, directory, 'package.json'), 'utf8'));
  assert.equal(data.dependencies.vastlint, version, `${directory} must use the pinned published core`);
}
const vscode = JSON.parse(fs.readFileSync(path.join(root, 'vscode/package.json'), 'utf8'));
assert.equal(vscode.dependencies['vastlint-engine'], `npm:vastlint@${version}`);
console.log(`All applications use published VASTlint ${version}.`);
