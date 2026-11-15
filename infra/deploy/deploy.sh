#!/usr/bin/env bash
# Installs one Field Shift release with no downtime worth noticing and rolls back by itself if it is unhealthy.
#
#   sudo /opt/fieldshift/bin/deploy.sh <release.tar.gz> [env.txt to install]
#
# A release is: fieldshift (Linux binary), frontend/dist, backend/data, VERSION. It is unpacked into
# releases/<version>, linked to the shared folders (secrets, database settings, uploads, caches), then "current" is
# switched atomically and the service restarted. /api/health must answer within 30 s, otherwise "current" goes
# back to the previous release. The last 3 releases are kept for instant rollback (deploy.sh --rollback).
set -euo pipefail
BASE=/opt/fieldshift
cd "$BASE"

health() { for _ in $(seq 1 30); do curl -fsS -m 2 http://127.0.0.1:8787/api/health >/dev/null 2>&1 && return 0; sleep 1; done; return 1; }
switch() { ln -sfn "$1" "$BASE/current.new" && mv -T "$BASE/current.new" "$BASE/current"; systemctl restart fieldshift; }

if [ "${1:-}" = "--rollback" ]; then
  # The release installed just before the current one (releases are ordered by install time).
  cur="$(basename "$(readlink -f "$BASE/current")")"
  prev="$(ls -1t releases | awk -v c="$cur" 'f { print; exit } $0 == c { f = 1 }')"
  [ -n "$prev" ] || { echo "no previous release"; exit 1; }
  switch "$BASE/releases/$prev"; health && echo "rolled back to $prev" || { echo "rollback unhealthy"; exit 1; }
  exit 0
fi

TGZ="${1:?usage: deploy.sh <release.tar.gz> [env.txt] | --rollback}"

# Secrets: env.txt arrives from the workflow (GitHub secret) and is installed readable only by root and the app.
if [ -n "${2:-}" ] && [ -s "$2" ]; then
  install -m 640 -o root -g fieldshift "$2" "$BASE/shared/env.txt"; rm -f "$2"
fi

STAGE="$(mktemp -d)"; trap 'rm -rf "$STAGE"' EXIT
tar -xzf "$TGZ" -C "$STAGE"
VER="$(cat "$STAGE/VERSION")"
DEST="$BASE/releases/$VER"
rm -rf "$DEST"; mv "$STAGE" "$DEST"; trap - EXIT
chmod 755 "$DEST" "$DEST/fieldshift"
# The app reads everything relative to its root: point the mutable parts at the shared folder.
ln -sfn "$BASE/shared/env.txt" "$DEST/env.txt"
ln -sfn "$BASE/shared/infra" "$DEST/infra"
ln -sfn "$BASE/shared/uploads" "$DEST/backend/uploads"
ln -sfn "$BASE/shared/cache" "$DEST/backend/cache"
chown -R root:root "$DEST"

# The release running now (none on the very first deploy).
PREV=""; [ -L "$BASE/current" ] && [ -d "$(readlink -f "$BASE/current")" ] && PREV="$(readlink -f "$BASE/current")"
switch "$DEST"
if health; then
  echo "deployed $VER"
  rm -f "$TGZ"
  ls -1dt releases/*/ | tail -n +4 | xargs -r rm -rf
else
  echo "release $VER is unhealthy; last log lines:"; journalctl -u fieldshift -n 40 --no-pager || true
  if [ -n "$PREV" ] && [ "$PREV" != "$DEST" ]; then switch "$PREV"; health && echo "rolled back to $(basename "$PREV")"
  else systemctl stop fieldshift; rm -f "$BASE/current"; echo "first release failed: nothing to roll back to"; fi
  rm -rf "$DEST"   # a failed release is not kept (it must never be a rollback target)
  exit 1
fi
