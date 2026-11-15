#!/usr/bin/env bash
# Creates the Field Shift server on Google Cloud. The OWNER runs it once in Cloud Shell
# (console.cloud.google.com → the ">_" button); it creates resources, so automation never runs it.
#
#   bash gcp-create.sh <project-id> "<contents of fieldshift_deploy.pub>"
#
# What it creates (inside the Compute Engine "Always Free" allowance when kept as is):
#   - one e2-micro VM, Debian 13, 30 GB standard persistent disk, zone us-east1-b (closest free region to Peru);
#   - an EPHEMERAL external IP on the Standard network tier (a reserved static IP is billed, so none is reserved);
#   - a firewall rule for HTTP/HTTPS only (SSH stays on Google's default rule, key-based);
#   - the deploy user's public SSH key, so GitHub Actions can configure and update the server.
# It does not touch billing. Add a budget alert yourself (infra/deploy/README.md, step 1).
set -euo pipefail

PROJECT="${1:?usage: bash gcp-create.sh <project-id> \"<deploy public key>\"}"
PUBKEY="${2:?usage: bash gcp-create.sh <project-id> \"<deploy public key>\"}"
ZONE="${ZONE:-us-east1-b}"
NAME="${NAME:-fieldshift}"

gcloud config set project "$PROJECT"
gcloud services enable compute.googleapis.com

gcloud compute firewall-rules describe fieldshift-web >/dev/null 2>&1 || \
  gcloud compute firewall-rules create fieldshift-web --network=default --direction=INGRESS \
    --action=ALLOW --rules=tcp:80,tcp:443 --target-tags=fieldshift-web --description="Field Shift: HTTP and HTTPS"

gcloud compute instances describe "$NAME" --zone="$ZONE" >/dev/null 2>&1 || \
  gcloud compute instances create "$NAME" --zone="$ZONE" \
    --machine-type=e2-micro \
    --image-family=debian-13 --image-project=debian-cloud \
    --boot-disk-size=30GB --boot-disk-type=pd-standard \
    --network-tier=STANDARD \
    --tags=fieldshift-web \
    --metadata="ssh-keys=deploy:${PUBKEY}"

IP="$(gcloud compute instances describe "$NAME" --zone="$ZONE" --format='get(networkInterfaces[0].accessConfigs[0].natIP)')"
echo
echo "Server created: $NAME in $ZONE"
echo "External IP:    $IP"
echo "Temporary HTTPS address until the domain exists: ${IP//./-}.sslip.io"
echo "Copy the IP back to Claude / to infra/deploy/github-setup.ps1."
