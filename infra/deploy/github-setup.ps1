# Gives the "Deploy" workflow what it needs, as encrypted GitHub repository secrets. The OWNER runs it once from
# the project folder (PowerShell), after creating the server; values are read from local files and never printed.
#
#   .\infra\deploy\github-setup.ps1 -Ip 34.x.x.x [-Domain fieldshift.example]
#
# Sets: DEPLOY_HOST (server IP), DEPLOY_SSH_KEY (~\.ssh\fieldshift_deploy, the deploy user's private key),
# FIELDSHIFT_ENV_TXT (env.txt, the API keys) and, when given, the FIELDSHIFT_DOMAIN variable. Re-run it with
# -Domain once the domain points at the server; without it the site uses <ip-with-dashes>.sslip.io.
param(
  [Parameter(Mandatory = $true)][string]$Ip,
  [string]$Domain = ''
)
$ErrorActionPreference = 'Stop'
$key = Join-Path $HOME '.ssh\fieldshift_deploy'
if (-not (Test-Path $key)) { throw "Deploy key not found: $key" }
if (-not (Test-Path 'env.txt')) { throw 'Run this from the project folder (env.txt not found).' }

gh secret set DEPLOY_HOST --body $Ip
Get-Content $key -Raw | gh secret set DEPLOY_SSH_KEY
Get-Content 'env.txt' -Raw | gh secret set FIELDSHIFT_ENV_TXT
if ($Domain) { gh variable set FIELDSHIFT_DOMAIN --body $Domain }
Write-Host 'GitHub is ready. Start a deploy with:  gh workflow run Deploy   (or Actions → Deploy → Run workflow)'
