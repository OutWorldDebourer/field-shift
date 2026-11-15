#!/usr/bin/env bash
# Deploys from this computer, without GitHub Actions (same release, same server scripts as deploy.yml).
# Needs Docker (builds the Linux binary in rust:1-bookworm), Node for the frontend, and the deploy key.
#
#   bash infra/deploy/local-deploy.sh <server IP> [site address]
#
# The site address defaults to <ip-with-dashes>.sslip.io (HTTPS without a domain). env.txt is uploaded and
# installed on the server readable only by root and the app; it is never printed. FS_DRY_RUN=1 builds only.
set -euo pipefail
HOST="${1:?usage: local-deploy.sh <server IP> [site address]}"
DOMAIN="${2:-${HOST//./-}.sslip.io}"
KEY="${FS_DEPLOY_KEY:-$HOME/.ssh/fieldshift_deploy}"
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
# Docker Desktop on Windows needs Windows paths for bind mounts (Git Bash gives /c/... or /tmp/...).
host_path() { if command -v cygpath >/dev/null 2>&1; then cygpath -w "$1"; else printf '%s' "$1"; fi; }
[ -f env.txt ] || { echo "env.txt not found in $ROOT"; exit 1; }
[ -f "$KEY" ] || { echo "deploy key not found: $KEY"; exit 1; }

echo "== backend (Linux release, Docker)"
OUT="$(mktemp -d)"; trap 'rm -rf "$OUT"' EXIT
MSYS_NO_PATHCONV=1 docker run --rm -v "$(host_path "$ROOT/backend"):/src:ro" -v fs-target:/target -v fs-cargo:/usr/local/cargo/registry \
  -e CARGO_TARGET_DIR=/target -w /src rust:1-bookworm cargo build --release --locked -p api --bin fieldshift
mkdir -p "$OUT/rel/frontend" "$OUT/rel/backend"
MSYS_NO_PATHCONV=1 docker run --rm -v fs-target:/target -v "$(host_path "$OUT/rel"):/out" rust:1-bookworm cp /target/release/fieldshift /out/

echo "== frontend"
(cd frontend && npx tsc --noEmit -p . && npx vite build >/dev/null)
cp -r frontend/dist "$OUT/rel/frontend/dist"
cp -r backend/data "$OUT/rel/backend/data"
echo "$(date -u +%Y%m%d-%H%M%S)-$(git rev-parse --short HEAD)" > "$OUT/rel/VERSION"
(cd "$OUT" && tar -czf release.tar.gz -C rel .)
echo "release $(cat "$OUT/rel/VERSION"), $(du -h "$OUT/release.tar.gz" | cut -f1)"
if [ -n "${FS_DRY_RUN:-}" ]; then tar -tzf "$OUT/release.tar.gz" | head -8; echo "dry run: built, not uploaded"; exit 0; fi

echo "== upload and install on $HOST ($DOMAIN)"
SSH=(ssh -i "$KEY" -o StrictHostKeyChecking=accept-new)
D=/tmp/fieldshift-deploy
"${SSH[@]}" deploy@"$HOST" "rm -rf $D && mkdir -p $D"
scp -q -i "$KEY" -o StrictHostKeyChecking=accept-new infra/deploy/* "$OUT/release.tar.gz" env.txt deploy@"$HOST":$D/
"${SSH[@]}" deploy@"$HOST" "sudo FS_DOMAIN='$DOMAIN' bash $D/bootstrap.sh $D && sudo /opt/fieldshift/bin/deploy.sh $D/release.tar.gz $D/env.txt; s=\$?; rm -rf $D; exit \$s"

echo "== public check"
for _ in $(seq 1 20); do
  if curl -fsS -m 10 "https://$DOMAIN/api/health" >/dev/null; then echo "live: https://$DOMAIN"; exit 0; fi
  sleep 6
done
echo "https://$DOMAIN/api/health did not answer yet (DNS or certificate still settling)"; exit 1
