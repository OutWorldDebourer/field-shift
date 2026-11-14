# 3. Architecture

Decided with the team owner on 2026-11-14. Data access behind each source was tested live: see `research/01_api_tests.md`.

## One product, one world

- **Index (landing):** a live Three.js globe at sunrise with real NASA imagery (Blue Marble, VIIRS night lights). Scrolling dives from orbit to the valley. A frosted-glass phone in front shows the app in use. Approved reference: `design-directions/approved-index-hero.webp`.
- **App:** the same world continues. The background becomes the farmer's own field from above; the interface is frosted glass floating over it.
- **Same features on phone and desktop:** location (GPS, map point or drawn outline, place search), up to 10 photos per message (camera or drag and drop), chat with the consultant, result sections, dashboard, export.

## Components

| Layer | Choice | Why |
|-------|--------|-----|
| Frontend | TypeScript, HTML, CSS; Three.js for the globe; a web map for the field | Owner's choice; one responsive web app (installable PWA) for phone and desktop |
| Backend | Rust | Owner's standing preference; fast, single binary |
| Database | MariaDB 11.4 LTS | Owner's choice; open source (GPL); spatial types for field outlines; good Rust support (`sqlx`) |
| Files | Server disk or Google Cloud Storage | Photos and exported PDFs stay out of the database |
| AI consultant | Claude through the Anthropic API | Two stages: an analyst stage builds a fixed evidence package from NASA and catalog data; the consultant presents it and may only cite its numbers |
| Hosting | One Google Compute Engine e2-micro VM (always-free tier) running the Rust API, MariaDB and an HTTPS web server | Free for the whole judging period (through January 2027); one public URL on the GoDaddy domain |

## What MariaDB stores

- Analyses: field outline, location mode (GPS or example site), crop history, priorities.
- Conversation turns and the state of each result section.
- References to photos (the files live outside the database).
- Generated reports, for the share link.
- A cache of NASA responses, so the demo stays fast and keeps working if a service is slow during judging.

## Data sources by access

| Access | Sources |
|--------|---------|
| No account | NASA POWER; Planetary Computer (Sentinel-2, Landsat, NASADEM, MODIS, NEX-GDDP-CMIP6, ERA5); NASA GIBS map layers; NASA CMR search |
| Earthdata Login token (owner's, stored in `env.txt`) | SMAP L4 soil moisture (NSIDC), IMERG Late daily rainfall (GES DISC), GRACE/GRACE-FO mascons (PO.DAAC) |

## Secrets

`env.txt` (API keys) and `infra/.env` (database passwords) are git-ignored and never printed in logs or chat.

## Privacy

Locations and crop photos are personal data. Analyses are deleted after a retention period, except the example sites, and the app says so.

## Status (2026-11-14, end of day one)

- Built: Rust backend (one crate per service, single binary that also serves the app), MariaDB schema, evidence package, crop knowledge, Claude consultant with demo fallback, technical sheet, CSV and share links, the index with the live globe and the app. See `06-how-it-works.md`.
- NEX-GDDP-CMIP6 licences confirmed (CC BY-SA 4.0 per v2.0 file); GRACE and the projection are precomputed for the Huaral example site (`backend/data/`).
- Not yet: deployment to the e2-micro VM and the domain (owner's step).
