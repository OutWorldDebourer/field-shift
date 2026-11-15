#!/usr/bin/env bash
# Server configuration for Field Shift on Debian 12/13. Idempotent: the deploy workflow runs it on every deploy,
# so the server's setup lives in this repository and changes here reach the VM with the next deploy.
#
#   sudo FS_DOMAIN=<site address> [FS_REDIRECTS="<other hosts>"] bash bootstrap.sh <dir with the infra/deploy files>
#
# Installs MariaDB (local only, small), Caddy (HTTPS), a 2 GB swap file, unattended security upgrades; creates the
# fieldshift user, /opt/fieldshift/{releases,shared,backups}, the database and its password (generated here, never
# leaves the server), the systemd unit, the Caddy site and a nightly backup.
set -euo pipefail
SRC="${1:?usage: FS_DOMAIN=<site> bash bootstrap.sh <deploy files dir>}"
DOMAIN="${FS_DOMAIN:?set FS_DOMAIN (the domain, or <ip-with-dashes>.sslip.io)}"
BASE=/opt/fieldshift
export DEBIAN_FRONTEND=noninteractive
changed() { ! cmp -s "$1" "$2"; }   # true when $2 is missing or differs from $1

# --- swap: 1 GB of RAM is tight for apt, MariaDB and the app together
if ! swapon --show 2>/dev/null | grep -q /swapfile; then
  if { [ -f /swapfile ] || fallocate -l 2G /swapfile; } && chmod 600 /swapfile && mkswap /swapfile >/dev/null && swapon /swapfile 2>/dev/null; then
    grep -q '^/swapfile' /etc/fstab || echo '/swapfile none swap sw 0 0' >> /etc/fstab
    echo 'vm.swappiness=10' > /etc/sysctl.d/90-fieldshift.conf; sysctl -q vm.swappiness=10 || true
  else
    echo "swap: not available here (skipped)"; rm -f /swapfile
  fi
fi

# --- packages
need=()
for p in mariadb-server curl gpg debian-keyring debian-archive-keyring apt-transport-https unattended-upgrades; do
  dpkg -s "$p" >/dev/null 2>&1 || need+=("$p")
done
if ((${#need[@]})); then apt-get update -qq && apt-get install -y -qq "${need[@]}"; fi
if ! command -v caddy >/dev/null; then
  # Official Caddy repository (https://caddyserver.com/docs/install#debian-ubuntu-raspbian)
  curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
  curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' > /etc/apt/sources.list.d/caddy-stable.list
  apt-get update -qq && apt-get install -y -qq caddy
fi
dpkg-reconfigure -f noninteractive unattended-upgrades >/dev/null 2>&1 || true

# --- MariaDB: small and local
if changed "$SRC/mariadb.cnf" /etc/mysql/mariadb.conf.d/90-fieldshift.cnf; then
  install -m 644 "$SRC/mariadb.cnf" /etc/mysql/mariadb.conf.d/90-fieldshift.cnf
  systemctl restart mariadb
fi
systemctl enable --now mariadb >/dev/null

# --- user and folders
id fieldshift >/dev/null 2>&1 || useradd --system --home "$BASE" --shell /usr/sbin/nologin fieldshift
install -d -m 755 "$BASE" "$BASE/releases" "$BASE/bin"
install -d -m 750 -o fieldshift -g fieldshift "$BASE/shared" "$BASE/shared/uploads" "$BASE/shared/cache" "$BASE/shared/infra"
install -d -m 700 "$BASE/backups"

# --- database and its password (generated once, kept only in shared/infra/.env)
ENVF="$BASE/shared/infra/.env"
if [ ! -s "$ENVF" ]; then
  PW="$(head -c 32 /dev/urandom | base64 | tr -dc 'A-Za-z0-9' | head -c 32)"
  printf 'DB_NAME=fieldshift\nDB_USER=fieldshift\nDB_PASSWORD=%s\n' "$PW" > "$ENVF"
  chown root:fieldshift "$ENVF"; chmod 640 "$ENVF"
fi
set -a
# shellcheck source=/dev/null
. "$ENVF"
set +a
mariadb -e "CREATE DATABASE IF NOT EXISTS \`$DB_NAME\` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
  CREATE USER IF NOT EXISTS '$DB_USER'@'localhost' IDENTIFIED BY '$DB_PASSWORD';
  ALTER USER '$DB_USER'@'localhost' IDENTIFIED BY '$DB_PASSWORD';
  GRANT ALL PRIVILEGES ON \`$DB_NAME\`.* TO '$DB_USER'@'localhost';
  CREATE USER IF NOT EXISTS '$DB_USER'@'127.0.0.1' IDENTIFIED BY '$DB_PASSWORD';
  ALTER USER '$DB_USER'@'127.0.0.1' IDENTIFIED BY '$DB_PASSWORD';
  GRANT ALL PRIVILEGES ON \`$DB_NAME\`.* TO '$DB_USER'@'127.0.0.1'; FLUSH PRIVILEGES;"

# --- deploy helpers
install -m 755 "$SRC/deploy.sh" "$BASE/bin/deploy.sh"
install -m 755 "$SRC/backup.sh" "$BASE/bin/backup.sh"
echo "30 3 * * * root $BASE/bin/backup.sh" > /etc/cron.d/fieldshift-backup

# --- systemd unit
if changed "$SRC/fieldshift.service" /etc/systemd/system/fieldshift.service; then
  install -m 644 "$SRC/fieldshift.service" /etc/systemd/system/fieldshift.service
  systemctl daemon-reload
fi
systemctl enable fieldshift >/dev/null 2>&1 || true

# --- Caddy site
TMP="$(mktemp)"
sed "s/__DOMAIN__/$DOMAIN/g" "$SRC/Caddyfile.tmpl" > "$TMP"
# Other names that should land on the site: www.<domain> for a real domain, plus FS_REDIRECTS (e.g. the old
# <ip>.sslip.io address), each a permanent redirect to the main address with its own certificate.
REDIRECTS="${FS_REDIRECTS:-}"
case "$DOMAIN" in :*|*.sslip.io) ;; *) REDIRECTS="www.$DOMAIN $REDIRECTS" ;; esac
for host in $REDIRECTS; do
  [ "$host" = "$DOMAIN" ] && continue
  printf '\n%s {\n\tredir https://%s{uri} permanent\n}\n' "$host" "$DOMAIN" >> "$TMP"
done
if changed "$TMP" /etc/caddy/Caddyfile; then
  caddy validate --config "$TMP" --adapter caddyfile >/dev/null
  install -m 644 "$TMP" /etc/caddy/Caddyfile
  systemctl reload caddy || systemctl restart caddy
fi
rm -f "$TMP"
systemctl enable --now caddy >/dev/null

echo "bootstrap: ok (site $DOMAIN)"
