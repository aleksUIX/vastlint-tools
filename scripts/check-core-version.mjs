import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const version = fs.readFileSync(path.join(root, 'core-version.txt'), 'utf8').trim();
assert.match(version, /^\d+\.\d+\.\d+$/);
const manifest = fs.readFileSync(path.join(root, 'crates/vastlint-mcp/Cargo.toml'), 'utf8');
assert.match(manifest, new RegExp(`^vastlint-core = "=${version.replaceAll('.', '\\.')}"$`, 'm'));
const cargoLock = fs.readFileSync(path.join(root, 'Cargo.lock'), 'utf8');
const corePackage = cargoLock.split('[[package]]').find(entry => /^name = "vastlint-core"$/m.test(entry));
assert.ok(corePackage, 'Cargo.lock must include the published engine');
assert.match(corePackage, new RegExp(`^version = "${version.replaceAll('.', '\\.')}"$`, 'm'));
assert.match(corePackage, /^source = "registry\+https:\/\/github\.com\/rust-lang\/crates\.io-index"$/m);

function checkLock(directory, dependency = 'vastlint') {
  const file = path.join(root, directory, 'package-lock.json');
  const lock = JSON.parse(fs.readFileSync(file, 'utf8'));
  const engine = lock.packages[`node_modules/${dependency}`];
  assert.ok(engine, `${directory} lockfile must include the engine`);
  assert.equal(engine.version, version, `${directory} lockfile engine version must match`);
  assert.equal(engine.link, undefined, `${directory} cannot link to a sibling engine checkout`);
  assert.equal(engine.resolved, `https://registry.npmjs.org/vastlint/-/vastlint-${version}.tgz`);
}

checkLock('.');
for (const directory of ['chrome', 'packages/vastlint-client',
  'examples/vastlint-client-browser', 'examples/vastlint-client-react-scratch', 'examples/vastlint-react-demo']) {
  const data = JSON.parse(fs.readFileSync(path.join(root, directory, 'package.json'), 'utf8'));
  assert.equal(data.dependencies.vastlint, version, `${directory} must use the pinned published core`);
  if (!directory.startsWith('packages/')) checkLock(directory);
}
const vscode = JSON.parse(fs.readFileSync(path.join(root, 'vscode/package.json'), 'utf8'));
assert.equal(vscode.dependencies['vastlint-engine'], `npm:vastlint@${version}`);
checkLock('vscode', 'vastlint-engine');
console.log(`All applications use published VASTlint ${version}.`);
