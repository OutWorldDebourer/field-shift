# shared

- `tokens.css` — fonts (@font-face), colour/type tokens, base elements, focus, selection, reduced motion.
- `components.css` — glass, buttons, EN|ES switch, instruments, reticle, badges, data row, chat bubbles, photo tiles, backdrop.
- `charts.css` — season chart, NDVI chart, calendar strip, dot scores.
- `contract.ts` — HTTP contract with the backend (two optional frontend additions are marked).
- `strings.ts` — every UI string in EN and ES.
- `i18n.ts` — language state (`?lang=`, localStorage), `t()`, `pick()`, number/date formatting.
- `langswitch.ts` — EN | ES switch markup and wiring.
- `api.ts` — typed client with timeouts and the offline demo fallback.
- `datarow.ts` — the data row (icon, label, value, verdict, source · date, badge, sparkline).
- `charts.ts` — SVG charts and the 12-month strip.
- `backdrop.ts` — field-from-above plate (backend Sentinel-2 snapshot, else EOX tiles) with outline and reticle.
- `phone.ts` — the hero's glass phone.
- `geo.ts` — area in ha, centroid, bbox helpers.
- `icons.ts` — line icon set.
- `dom.ts` — escaping, element helpers, toast.
- `demo/` — bundled Huaral demo dataset.
