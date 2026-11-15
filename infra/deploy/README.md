# infra/deploy — putting Field Shift on the internet

One small server runs everything: a Google Compute Engine **e2-micro** (Always Free) with Debian 13, where
**Caddy** serves HTTPS, **MariaDB** keeps the data and the **`fieldshift`** binary serves the app and the API.
GitHub Actions builds each release and installs it; the server's whole setup lives in this folder.

```
visitor ──HTTPS──▶ Caddy :443 ──▶ fieldshift 127.0.0.1:8787 ──▶ MariaDB 127.0.0.1:3306
                   (Let's Encrypt)   (systemd, /opt/fieldshift/current)
```

## Files

| File | What it does | Who runs it |
|---|---|---|
| `gcp-create.sh` | Creates the VM, the HTTP/HTTPS firewall rule and the deploy user's SSH key on Google Cloud | The owner, once, in Cloud Shell |
| `github-setup.ps1` | Stores the server IP, the deploy key and `env.txt` as encrypted GitHub secrets | The owner, once, from this folder's repo root |
| `bootstrap.sh` | Idempotent server setup: swap, MariaDB (small, local), Caddy, user, folders, database + generated password, systemd unit, Caddy site, nightly backup, unattended security upgrades | The Deploy workflow, every deploy |
| `deploy.sh` | Unpacks a release, links shared data, switches `current` atomically, restarts, checks `/api/health`, rolls back by itself if unhealthy; keeps 3 releases; `--rollback` for a manual step back | The Deploy workflow; by hand over SSH |
| `fieldshift.service` | systemd unit (hardened, memory-capped, restarts on failure) | installed by `bootstrap.sh` |
| `Caddyfile.tmpl` | HTTPS site, compression, security headers, no buffering for streamed answers | installed by `bootstrap.sh` |
| `mariadb.cnf` | MariaDB sized for 1 GB RAM, bound to 127.0.0.1 | installed by `bootstrap.sh` |
| `backup.sh` | Nightly `mariadb-dump`, last 7 kept in `/opt/fieldshift/backups` | cron |

Workflows: `.github/workflows/ci.yml` (tests + build on every push to `main`, never deploys) and
`.github/workflows/deploy.yml` (publishes; only on "Run workflow" or a pushed `v*` tag).

## Owner steps (once, about 20 minutes)

These create accounts, resources or public things, so they are the owner's.

1. **Google Cloud project and a budget alert.** In console.cloud.google.com create a project (e.g. `field-shift`)
   linked to your billing account. The Always Free e2-micro applies to paid accounts too; the $300 trial is not
   needed. Then *Billing → Budgets & alerts → Create budget*: US$ 5 per month, alerts at 50 / 90 / 100 %.
2. **Create the server.** Open Cloud Shell (the `>_` button), upload `infra/deploy/gcp-create.sh` (⋮ → Upload) and run:
   ```bash
   bash gcp-create.sh <project-id> "<one line from %USERPROFILE%\.ssh\fieldshift_deploy.pub>"
   ```
   It prints the server IP and a temporary HTTPS address `<ip-with-dashes>.sslip.io`.
3. **Give GitHub what the deploy needs** (PowerShell, repository root):
   ```powershell
   .\infra\deploy\github-setup.ps1 -Ip <server IP>
   ```
4. **First deploy:** `gh workflow run Deploy` (or GitHub → Actions → Deploy → Run workflow). About 8 minutes the
   first time. The site answers at `https://<ip-with-dashes>.sslip.io`.
5. **Domain.** Claim the free domain from the Space Apps Global Offers (GoDaddy Registry → Porkbun; non-premium
   names only). In Porkbun → DNS add an `A` record for the bare domain (and `www` if wanted) pointing at the server
   IP. Then `.\infra\deploy\github-setup.ps1 -Ip <server IP> -Domain <your domain>` and run Deploy again: Caddy
   obtains the certificate by itself.
6. **Before submitting:** submit the domain for classification at FortiGuard (fortiguard.com → Web Filter
   Lookup → "Request a review") so school and company networks do not block a brand-new domain; make the
   repository public if the submission requires it.

## Everyday changes

- Work locally as usual. Every push to `main` runs **CI** (tests, type check, build); nothing goes live.
- To publish: `gh workflow run Deploy`, the "Run workflow" button, or `git tag v1.2 && git push --tags`.
  The workflow builds, uploads, runs `bootstrap.sh` (applies any server change made in this folder), runs
  `deploy.sh` and checks the public `/api/health`. A release that does not answer within 30 s is rolled back
  automatically and removed.
- Manual step back: `ssh deploy@<ip> sudo /opt/fieldshift/bin/deploy.sh --rollback`.
- Database changes ship as migrations in `backend/migrations`; the app applies them when it starts.
- Logs: `ssh deploy@<ip> journalctl -u fieldshift -n 100` (app) and `journalctl -u caddy` (web).
- New API keys: edit `env.txt`, re-run `github-setup.ps1`, deploy.

## Costs and limits

- e2-micro, 30 GB standard disk and the ephemeral external IP: inside the Compute Engine Always Free allowance in
  `us-east1` / `us-central1` / `us-west1` (a reserved static IP would be billed, so none is reserved; the IP only
  changes if the VM is deleted and recreated). Standard-tier egress keeps traffic costs at zero for this scale.
- The consultant's live model turns are capped (30 per visitor per hour, 400 per day; past that it answers in
  demo mode). Also set a monthly spend limit in the Anthropic console.
- Measured in a Debian 13 test VM: the app uses ~20 MB of RAM, MariaDB ~145 MB, Caddy ~50 MB; a 2 GB swap file
  covers peaks.

## Tested

The kit was run end to end in a Debian 13 systemd container (bootstrap twice, first deploy, a second release,
a broken release rolled back automatically and removed, manual rollback, MariaDB restart, nightly backup,
permissions of the secrets), with the Linux binary built exactly as the workflow builds it. Workflows pass
`actionlint`; scripts pass `shellcheck`.
