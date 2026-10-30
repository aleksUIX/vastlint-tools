import json
import time
import urllib.request

headers = {"User-Agent": "VASTlint-Smithery-Check/1.0"}
request = urllib.request.Request(
    "https://vastlint.org/mcp",
    data=json.dumps({"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}}).encode(),
    headers={**headers, "Content-Type": "application/json", "Accept": "application/json, text/event-stream"},
)
with urllib.request.urlopen(request, timeout=30) as response:
    raw = response.read().decode()
    if raw.startswith("data:"):
        raw = next(line[5:].strip() for line in raw.splitlines() if line.startswith("data:"))
    expected = {tool["name"] for tool in json.loads(raw)["result"]["tools"]}
for attempt in range(6):
    request = urllib.request.Request("https://api.smithery.ai/servers/aleksander/vastlint", headers=headers)
    with urllib.request.urlopen(request, timeout=30) as response:
        server = json.load(response)
    actual = {tool["name"] for tool in server["tools"]}
    if actual == expected:
        print("Verified all", len(expected), "live MCP tools on the Smithery listing.")
        break
    if attempt == 5:
        raise SystemExit("Smithery catalog is stale. Missing: " + str(sorted(expected - actual)))
    time.sleep(5)
