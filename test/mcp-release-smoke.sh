#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
export MCP_BINARY="${CARGO_TARGET_DIR:-target}/release/vastlint-mcp"

# Build MCP server (release)
cargo build --release -p vastlint-mcp
# MCP smoke : initialize + tools/list returns 7 tools
RESPONSE=$(
  (
    echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"ci","version":"1.0"}}}'
    sleep 0.2
    echo '{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}'
    sleep 0.2
    echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
    sleep 1
  ) | "$MCP_BINARY" 2>/dev/null
)

echo "$RESPONSE" | grep -q '"tools"' || {
  echo "FAIL: initialize response missing tools capability"
  echo "$RESPONSE"
  exit 1
}
echo "capabilities.tools present ✓"

TOOL_COUNT=$(echo "$RESPONSE" | grep '"tools":\[' | grep -o '"name":"' | wc -l | tr -d ' ')
if [ "$TOOL_COUNT" -ne 7 ]; then
  echo "FAIL: expected 7 tools, got $TOOL_COUNT"
  echo "$RESPONSE"
  exit 1
fi
echo "tools/list returned $TOOL_COUNT tools ✓"

for TOOL in validate_vast validate_vast_url fix_vast list_rules explain_rule inspect_vast; do
  echo "$RESPONSE" | grep -q "\"$TOOL\"" || {
    echo "FAIL: tool '$TOOL' missing from tools/list"
    echo "$RESPONSE"
    exit 1
  }
  echo "  $TOOL ✓"
done

# MCP smoke : validate_vast (valid + invalid) and fix_vast tool calls
python3 - <<'PYEOF'
import os
import subprocess, json, sys, time

VALID_XML   = open('test/fixtures/valid_4.2.xml').read()
INVALID_XML = open('test/fixtures/err_inline_missing_impression.xml').read()
HTTP_XML    = ('<VAST version="2.0"><Ad id="1"><InLine>'
               '<AdSystem>T</AdSystem><AdTitle>T</AdTitle>'
               '<Impression>http://t.example.com/imp</Impression>'
               '<Creatives><Creative><Linear><Duration>00:00:30</Duration>'
               '<MediaFiles><MediaFile delivery="progressive" type="video/mp4"'
               ' width="640" height="480">http://cdn.example.com/ad.mp4'
               '</MediaFile></MediaFiles></Linear></Creative></Creatives>'
               '</InLine></Ad></VAST>')

proc = subprocess.Popen(
    [os.environ['MCP_BINARY']],
    stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL
)

_id = [0]
def send(msg):
    _id[0] += 1
    if 'id' not in msg:
        msg = dict(msg, id=_id[0])
    proc.stdin.write((json.dumps(msg) + '\n').encode())
    proc.stdin.flush()
    return msg.get('id')

def read_until(target_id, timeout=10):
    import select
    deadline = time.monotonic() + timeout
    buf = b''
    while time.monotonic() < deadline:
        r, _, _ = select.select([proc.stdout], [], [], 0.5)
        if r:
            chunk = proc.stdout.read1(4096)
            if chunk:
                buf += chunk
                while b'\n' in buf:
                    line, buf = buf.split(b'\n', 1)
                    line = line.strip()
                    if not line:
                        continue
                    try:
                        d = json.loads(line)
                    except Exception:
                        continue
                    if d.get('id') == target_id:
                        return d
    raise TimeoutError(f'no response for id={target_id}')

# Initialize
init_id = send({'jsonrpc':'2.0','method':'initialize','params':{
    'protocolVersion':'2024-11-05','capabilities':{},'clientInfo':{'name':'ci','version':'1.0'}}})
read_until(init_id)
proc.stdin.write((json.dumps({'jsonrpc':'2.0','method':'notifications/initialized','params':{}}) + '\n').encode())
proc.stdin.flush()

# validate_vast — valid fixture
vid = send({'jsonrpc':'2.0','method':'tools/call','params':{'name':'validate_vast','arguments':{'xml': VALID_XML}}})
r = read_until(vid)
obj = json.loads(r['result']['content'][0]['text'])
assert obj['valid'] is True,  f"FAIL: MCP validate_vast valid: expected valid=true, got {obj}"
assert obj['summary']['errors'] == 0, f"FAIL: MCP validate_vast valid: errors={obj['summary']['errors']}"
assert obj['version'] == '4.2', f"FAIL: MCP validate_vast valid: version={obj['version']}"
print("PASS: MCP validate_vast valid fixture ✓")

# validate_vast — invalid fixture with specific rule ID
iid = send({'jsonrpc':'2.0','method':'tools/call','params':{'name':'validate_vast','arguments':{'xml': INVALID_XML}}})
r = read_until(iid)
obj = json.loads(r['result']['content'][0]['text'])
assert obj['valid'] is False, f"FAIL: MCP validate_vast invalid: expected valid=false, got {obj}"
assert obj['summary']['errors'] > 0, f"FAIL: MCP validate_vast invalid: errors=0"
ids = [i['id'] for i in obj['issues']]
assert 'VAST-2.0-inline-impression' in ids, f"FAIL: MCP validate_vast invalid: rule ID not found in {ids}"
print(f"PASS: MCP validate_vast invalid fixture, VAST-2.0-inline-impression fires ✓")

# fix_vast — HTTP→HTTPS upgrade
fid = send({'jsonrpc':'2.0','method':'tools/call','params':{'name':'fix_vast','arguments':{'xml': HTTP_XML}}})
r = read_until(fid)
obj = json.loads(r['result']['content'][0]['text'])
assert obj['applied_count'] > 0, f"FAIL: MCP fix_vast: no fixes applied, got {obj}"
assert 'https://' in obj['xml'], f"FAIL: MCP fix_vast: no https:// in fixed XML"
assert 'http://' not in obj['xml'], f"FAIL: MCP fix_vast: http:// still in fixed XML"
print(f"PASS: MCP fix_vast applied {obj['applied_count']} fix(es), HTTP→HTTPS ✓")

proc.stdin.close()
proc.wait(timeout=5)
print("All MCP tool-call smoke tests passed.")
PYEOF

# MCP smoke : list_rules and explain_rule tool calls
python3 - <<'PYEOF'
import os
import subprocess, json, time

proc = subprocess.Popen(
    [os.environ['MCP_BINARY']],
    stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL
)

_id = [0]
def send(msg):
    _id[0] += 1
    if 'id' not in msg:
        msg = dict(msg, id=_id[0])
    proc.stdin.write((json.dumps(msg) + '\n').encode())
    proc.stdin.flush()
    return msg.get('id')

def read_until(target_id, timeout=10):
    import select
    deadline = time.monotonic() + timeout
    buf = b''
    while time.monotonic() < deadline:
        r, _, _ = select.select([proc.stdout], [], [], 0.5)
        if r:
            chunk = proc.stdout.read1(4096)
            if chunk:
                buf += chunk
                while b'\n' in buf:
                    line, buf = buf.split(b'\n', 1)
                    line = line.strip()
                    if not line:
                        continue
                    try:
                        d = json.loads(line)
                    except Exception:
                        continue
                    if d.get('id') == target_id:
                        return d
    raise TimeoutError(f'no response for id={target_id}')

init_id = send({'jsonrpc':'2.0','method':'initialize','params':{
    'protocolVersion':'2024-11-05','capabilities':{},'clientInfo':{'name':'ci','version':'1.0'}}})
read_until(init_id)
proc.stdin.write((json.dumps({'jsonrpc':'2.0','method':'notifications/initialized','params':{}}) + '\n').encode())
proc.stdin.flush()

# list_rules — must return a non-empty catalog
lid = send({'jsonrpc':'2.0','method':'tools/call','params':{'name':'list_rules','arguments':{}}})
r = read_until(lid)
obj = json.loads(r['result']['content'][0]['text'])
assert obj['count'] > 0, f"FAIL: list_rules returned count=0"
assert len(obj['rules']) == obj['count'], "FAIL: list_rules count mismatch"
ids = [rule['id'] for rule in obj['rules']]
for expected_id in ['VAST-2.0-inline-impression', 'VAST-4.1-adservingid-present', 'VAST-2.0-root-version']:
    assert expected_id in ids, f"FAIL: list_rules missing rule {expected_id}"
print(f"PASS: MCP list_rules returned {obj['count']} rules, key IDs present ✓")

# explain_rule — known rule returns description and hint
eid = send({'jsonrpc':'2.0','method':'tools/call','params':{
    'name':'explain_rule','arguments':{'rule_id':'VAST-2.0-inline-impression'}}})
r = read_until(eid)
obj = json.loads(r['result']['content'][0]['text'])
assert obj['id'] == 'VAST-2.0-inline-impression', f"FAIL: explain_rule wrong id: {obj['id']}"
assert len(obj.get('description','')) > 10, f"FAIL: explain_rule empty description"
assert obj.get('severity') == 'error', f"FAIL: explain_rule wrong severity: {obj.get('severity')}"
print(f"PASS: MCP explain_rule VAST-2.0-inline-impression: severity=error, description present ✓")

# explain_rule — unknown rule returns graceful not-found message
uid = send({'jsonrpc':'2.0','method':'tools/call','params':{
    'name':'explain_rule','arguments':{'rule_id':'VAST-9.9-nonexistent'}}})
r = read_until(uid)
obj = json.loads(r['result']['content'][0]['text'])
assert 'not found' in obj.get('description','').lower() or obj.get('severity') == 'unknown', \
    f"FAIL: explain_rule unknown ID did not return graceful response: {obj}"
print("PASS: MCP explain_rule unknown rule ID → graceful not-found ✓")

proc.stdin.close()
proc.wait(timeout=5)
print("All MCP list_rules/explain_rule smoke tests passed.")
PYEOF
