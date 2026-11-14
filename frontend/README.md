# frontend

Field Shift web app: Vite 7 multi-page, TypeScript strict. `npm run dev` (port 5173, `/api` → 127.0.0.1:8787), `npm run build`.

- `index.html` — landing page (orbit → valley → daylight), entry `src/index/main.ts`.
- `app.html` — the app (field selection, workspace, dashboard, rotation), entry `src/app/main.ts`. Query: `?site=huaral` (preselect example), `&go=1` (open it), `?a=<id>`, `?a=demo-huaral` (offline demo), `?share=<token>` (read-only), `&tab=dashboard|rotation`, `?lang=es`.
- `globe.html` — globe test page (owned by the globe module).
- `src/` — source; `public/` — static files; `screens/` — audit screenshots.
