#!/usr/bin/env bash
# Nightly database backup (cron, installed by bootstrap.sh): a compressed dump, the last 7 kept on the VM disk.
set -euo pipefail
DIR=/opt/fieldshift/backups
mkdir -p "$DIR"
# shellcheck disable=SC1091
set -a; . /opt/fieldshift/shared/infra/.env; set +a
# The password goes through a private option file, never on the command line (visible in ps).
CNF="$(mktemp)"; trap 'rm -f "$CNF"' EXIT
chmod 600 "$CNF"
printf '[client]\nuser=%s\npassword=%s\n' "$DB_USER" "$DB_PASSWORD" > "$CNF"
mariadb-dump --defaults-extra-file="$CNF" --single-transaction --quick --routines "$DB_NAME" | gzip -9 > "$DIR/fieldshift-$(date -u +%Y%m%d).sql.gz"
ls -1t "$DIR"/fieldshift-*.sql.gz | tail -n +8 | xargs -r rm -f
