import json
import os
from pathlib import Path
import urllib.error
import urllib.request

endpoint = "https://api.smithery.ai/servers/aleksander/vastlint"
with urllib.request.urlopen(urllib.request.Request(endpoint, headers={"User-Agent": "VASTlint-Smithery-Repair/1.0"}), timeout=30) as response:
    existing = json.load(response)
assert existing["qualifiedName"] == "aleksander/vastlint"
key = os.environ.get("SMITHERY_API_KEY")
if not key:
    raise SystemExit("The repository Smithery API key is missing.")
metadata = json.loads(Path("docs/smithery-listing.json").read_text())
request = urllib.request.Request(
    endpoint,
    data=json.dumps(metadata).encode(),
    method="PATCH",
    headers={"Authorization": "Bearer " + key, "Content-Type": "application/json", "User-Agent": "VASTlint-Smithery-Repair/1.0"},
)
try:
    with urllib.request.urlopen(request, timeout=30) as response:
        result = json.load(response)
        print("Smithery metadata update HTTP status:", response.status)
        print("Smithery reported success:", result.get("success"))
except urllib.error.HTTPError as error:
    message = error.read(2000).decode("utf-8", errors="replace").replace(key, "[redacted]")
    raise SystemExit("Smithery rejected the update: HTTP " + str(error.code) + " " + message)
with urllib.request.urlopen(urllib.request.Request(endpoint, headers={"User-Agent": "VASTlint-Smithery-Repair/1.0"}), timeout=30) as response:
    updated = json.load(response)
assert updated["description"] == metadata["description"], "Smithery did not persist the repaired description."
print("Verified the repaired description on the existing public listing.")
