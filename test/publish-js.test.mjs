import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const script = fileURLToPath(new URL('../scripts/publish-js-packages.sh', import.meta.url));

function runPublication({ latest = '0.16.0', existing = false, token = '', rejectToken = false, fail = false } = {}) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'vastlint-tools-publish-'));
  const stage = path.join(directory, 'stage');
  const bin = path.join(directory, 'bin');
  fs.mkdirSync(bin);
  for (const name of ['vastlint-client', 'vastlint-react']) {
    const destination = path.join(stage, name);
    fs.mkdirSync(destination, { recursive: true });
    fs.writeFileSync(path.join(destination, 'package.json'), JSON.stringify({ name, version: '0.16.1' }));
  }
  const log = path.join(directory, 'calls.jsonl');
  fs.writeFileSync(path.join(bin, 'npm'), `#!${process.execPath}
const fs = require('node:fs');
const args = process.argv.slice(2);
if (args[0] === 'view') {
  if (args[1].includes('@')) process.exit(process.env.FAKE_EXISTING === 'true' ? 0 : 1);
  console.log(process.env.FAKE_LATEST);
  process.exit(0);
}
if (args[0] !== 'publish') process.exit(99);
fs.appendFileSync(process.env.FAKE_LOG, JSON.stringify({name: require(process.cwd() + '/package.json').name,args,token: Boolean(process.env.NODE_AUTH_TOKEN)}) + '\\n');
if (process.env.FAKE_FAIL === 'true' || (process.env.FAKE_REJECT_TOKEN === 'true' && process.env.NODE_AUTH_TOKEN)) process.exit(1);
`, { mode: 0o755 });
  try {
    const result = spawnSync('bash', [script, stage], {
      encoding: 'utf8',
      env: { ...process.env, PATH: `${bin}${path.delimiter}${process.env.PATH}`, NODE_AUTH_TOKEN: token,
        FAKE_LATEST: latest, FAKE_EXISTING: String(existing), FAKE_REJECT_TOKEN: String(rejectToken), FAKE_FAIL: String(fail), FAKE_LOG: log },
    });
    assert.equal(fs.existsSync(path.join(stage, '.npmrc')), false, 'temporary registry token must be removed');
    return { status: result.status, calls: fs.existsSync(log) ? fs.readFileSync(log, 'utf8').trim().split('\n').map(JSON.parse) : [] };
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

test('publishes both new packages with provenance', () => {
  const result = runPublication();
  assert.equal(result.status, 0);
  assert.deepEqual(result.calls.map(call => call.name), ['vastlint-client', 'vastlint-react']);
  for (const call of result.calls) assert.deepEqual(call.args, ['publish', '--provenance', '--tag', 'latest']);
});

test('already-published versions are idempotent', () => {
  assert.deepEqual(runPublication({ existing: true }), { status: 0, calls: [] });
});

test('recovery of an older version preserves the latest dist-tag', () => {
  const result = runPublication({ latest: '0.17.0' });
  assert.equal(result.status, 0);
  for (const call of result.calls) assert.deepEqual(call.args, ['publish', '--provenance', '--tag', 'archive-0.16.1']);
});

test('a rejected token retries each package with OIDC', () => {
  const result = runPublication({ token: 'fixture-token', rejectToken: true });
  assert.equal(result.status, 0);
  assert.deepEqual(result.calls.map(call => [call.name, call.token]), [
    ['vastlint-client', true], ['vastlint-client', false], ['vastlint-react', true], ['vastlint-react', false],
  ]);
});

test('publication failures stop the release before the dependent package', () => {
  const result = runPublication({ fail: true });
  assert.notEqual(result.status, 0);
  assert.deepEqual(result.calls.map(call => call.name), ['vastlint-client']);
});
