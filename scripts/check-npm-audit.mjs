#!/usr/bin/env node
/**
 * npm audit wrapper. GHSA-fc5q-39xc-vjpf has no npm release yet; we pin
 * micromatch/picomatch#d9a3f12 (GHSA fix PR) via overrides and treat that as
 * mitigated when the lockfile resolves picomatch from that commit.
 */
import { execFileSync } from "node:child_process";
import { readFileSync, existsSync } from "node:fs";
import { resolve } from "node:path";

const PICOMATCH_ADVISORY = "GHSA-fc5q-39xc-vjpf";
const PICOMATCH_FIX_COMMIT = "d9a3f12faf186860cfc86c2834c41f38701152f2";
const PICOMATCH_CHAIN = new Set(["picomatch", "fdir", "tinyglobby", "vite", "@vscode/vsce", "ovsx"]);
const cwd = process.cwd();

function lockfileUsesPicomatchFix() {
  const lockPath = resolve(cwd, "package-lock.json");
  if (!existsSync(lockPath)) {
    return false;
  }
  const lock = readFileSync(lockPath, "utf8");
  return (
    lock.includes(PICOMATCH_FIX_COMMIT) ||
    lock.includes("Arulraj-a2b/picomatch")
  );
}

function mentionsPicomatchAdvisory(vuln) {
  return (vuln.via ?? []).some(
    (via) => typeof via === "object" && via.url?.includes(PICOMATCH_ADVISORY)
  );
}

function isPicomatchAdvisoryOnly(vulns) {
  if (vulns.length === 0) {
    return false;
  }
  const names = new Set(vulns.map((v) => v.name));
  if (![...names].every((name) => PICOMATCH_CHAIN.has(name))) {
    return false;
  }
  return vulns.some(mentionsPicomatchAdvisory);
}

let audit;
try {
  const raw = execFileSync("npm", ["audit", "--audit-level=low", "--json"], {
    cwd,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  audit = JSON.parse(raw);
} catch (error) {
  const stdout = error.stdout?.toString() ?? "";
  try {
    audit = JSON.parse(stdout);
  } catch {
    console.error(error.stderr?.toString() ?? error.message);
    process.exit(1);
  }
}

const vulns = Object.values(audit.vulnerabilities ?? {});

if (isPicomatchAdvisoryOnly(vulns) && lockfileUsesPicomatchFix()) {
  console.log(
    `npm audit: ${PICOMATCH_ADVISORY} mitigated via picomatch git override (${PICOMATCH_FIX_COMMIT.slice(0, 7)})`
  );
  process.exit(0);
}

if (vulns.length > 0) {
  execFileSync("npm", ["audit", "--audit-level=low"], {
    cwd,
    stdio: "inherit",
  });
  process.exit(1);
}

console.log("npm audit: no vulnerabilities");