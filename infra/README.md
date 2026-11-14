# infra

Local development services (Docker) and, later, server deployment files.

| File | What it is |
|------|------------|
| `compose.yaml` | MariaDB 11.4 LTS bound to 127.0.0.1:3306 with a named volume |
| `.env.example` | Variable names for `.env` (DB name, user, passwords) |
| `.env` | Real local values, git-ignored, generated locally |

Start: `docker compose -f infra/compose.yaml --env-file infra/.env up -d`
