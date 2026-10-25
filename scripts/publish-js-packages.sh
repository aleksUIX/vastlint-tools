#!/usr/bin/env bash
# Publish the staged application packages without changing engine pins.
set -euo pipefail

STAGE="$(cd "${1:?Usage: bash scripts/publish-js-packages.sh <stage-directory>}" && pwd)"
trap 'rm -f "$STAGE/.npmrc"' EXIT

if [ -n "${NODE_AUTH_TOKEN:-}" ]; then
  printf '%s\n' "//registry.npmjs.org/:_authToken=${NODE_AUTH_TOKEN}" > "$STAGE/.npmrc"
else
  unset NODE_AUTH_TOKEN NPM_CONFIG_USERCONFIG
fi

publish_package() {
  local name="$1" version latest higher dist_tag=latest
  version="$(node -p 'require("./package.json").version')"
  if npm view "${name}@${version}" version >/dev/null 2>&1; then
    echo "${name}@${version} is already published"
    return 0
  fi
  latest="$(npm view "$name" version 2>/dev/null || true)"
  if [ -n "$latest" ] && [ "$version" != "$latest" ]; then
    higher="$(printf '%s\n%s\n' "$version" "$latest" | sort -V | tail -n1)"
    if [ "$higher" = "$latest" ]; then
      dist_tag="archive-${version}"
    fi
  fi
  if npm publish --provenance --tag "$dist_tag"; then
    return 0
  fi
  if [ -z "${NODE_AUTH_TOKEN:-}" ]; then
    return 1
  fi
  echo "Retrying ${name} with npm trusted publishing"
  rm -f "$STAGE/.npmrc"
  unset NODE_AUTH_TOKEN NPM_CONFIG_USERCONFIG
  npm publish --provenance --tag "$dist_tag"
}

for name in vastlint-client vastlint-react; do
  (cd "$STAGE/$name" && publish_package "$name")
done
