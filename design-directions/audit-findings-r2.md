# Round 2 audit findings, for the last fix pass (round 3 is the final audit)

## Index with globe (7.35)
1. `.hud` fixed corner readouts still cover section titles ("Read from space," at y1100, "for the field." at y2200) and cards: fade the HUD out while a section title/card is in the bottom ~120 px (IntersectionObserver) or add bottom padding + dark band; on ≤600 px show only ALT.
2. Mobile: reticle labels (#lbl-valley, #lbl-field) still ghost behind "Try an example site" and over the lede at y2200; hide or move them when their box overlaps `.hero__text`, `.sec-lede` or a CTA.
3. `.sky__leaders path`: end each leader at the label box edge (not through "11.47°S 77.21°W" / "Field: 5.1 ha"); give the labels a dark halo/plate.
4. Mobile HUD credit cut ("Contains modified Copernicus Senti…"): let it wrap to two lines at ≤600 px — the full sentence is a licence requirement.
5. Hero phone: tags and `.phone__foot` ≥12 px, row labels ≥14 px; raise `.scroll-cue` contrast.
6. Dashboard preview cards: no truncation ("9 km EASE-Gri…" → "9 km grid", wrap source lines), short label "Soil moisture (area)", plain-language line before m³/m³ and degree-day "generations".
7. Dead fog gap (~300–400 px) before "Your field, season by season.": reduce that section's top padding.
8. (globe) cloud ellipses / snow speckle still read as artificial — handled separately if time allows.

## Field selection (7.25)
1. Image date honesty: only show "Image 13 Nov 2026" / the Sentinel-2 date when the dated snapshot actually covers the field (check bounds vs the overlay bounds); otherwise "Base image 2016, may be outdated" + EOX credit.
2. Seam between the 2026 snapshot and the 2016 base: request a snapshot large enough for the viewport at the example's zoom (~4–5 km; backend accepts size_m up to 10000) or clamp fitBounds inside it; draw a thin white hairline round the overlay edge.
3. Soft imagery: frame the example at ~ALT 2–3 km, cap zoom at 16.
4. Phone: put Drop a point / Draw outline and the area readout directly under the map (first screen); credit must not truncate (`white-space: normal`).
5. Scale bar legible: 13 px Plex Mono 500, dark text-shadow, 2 px white border.
6. Direction devices: thin white leader from the bracket to the area label; desktop ALT bottom-left of the map area; top bar as a hairline over imagery like the hero.
7. Move the OSM licence line into the licences popover; arrow-key/roving tabindex on the `.seg` radiogroup; show "5.1 ha" once in white on the map and once in amber in the readout (drop it from the example card meta).

## Dashboard + rotation (6.7 provisional)
Note: the screenshot tool now slices inner scroll containers: `node tools/screens.mjs <url> <prefix> 6000 0 0 ".view--dashboard"` (or `.view--rotation`) → `-s1.jpg`, `-s2.jpg`…
1. Move `section.dash__chart` ("This season vs 40-year normal") above `.dash__row--3` so it is in the first desktop viewport.
2. Projection card: add EN/ES "Projection, not a forecast"; show "13 CMIP6 models" with the list behind details/title; show the range (d_temp_range) and scenario.
3. Mobile: layer switch as one horizontally scrolling row above the map (not a 2×2 panel over it); merge the disclaimer pill and `.ws__credit` into one short line so the bottom chrome stops covering the first tile.
4. Rotation plan layer: draw the blue dashed plan visibly on the field backdrop, fix the stray blue arc between `.strat` columns; or remove the dashed key from `.rot__legend`.
5. Basemap honesty in `.dmap__legend`: "Base image 2016 · outline and values 13 Nov 2026" (or use the dated snapshot as basemap); never truncate "9 km EASE-Grid" (wrap).
6. Outdoor sizes: `.al small`, `.strat__more dt`, `.dmap__scale` ≥13 px; bigger score dots and "3/5" text beside each row.
7. `.strat__more summary` needs a visible disclosure marker (chevron/+).
8. Amber only for the key number: EN underline, Demo dot and :focus-visible outline in white; `.dash__card h3` in Source Serif italic; drop the repeated "(+5.0 °C)" in the temperature verdict where the value already shows it.

## Analysis workspace (7.4)
1. `dataRowCompact` in `.sec__peek`: show the plain verdict ("drier than normal", "high vigor", "much warmer than normal") with the number; temperature label says "vs normal".
2. (backend, done by the coordinator) the scripted demo no longer repeats its summary; a clean showcase analysis replaces the test one (new id in research/cache/last_id).
3. Phone `.ws__credit`: wrap to two lines or "Sentinel-2 · © Copernicus 2026 · 2026-11-13"; date and licence must stay visible.
4. `shortRes()`: match a leading `^~?[\d.]+\s*k?m` first ("9 km"), never cut to "9 km EASE-Gri…"; date and resolution in nowrap spans so the badge stays on the source line.
5. Phone chat ≥50 % of the screen: hide the AI avatar column on narrow screens, collapse crop line / sheet peek into the chip row, smaller section buttons (~56 px).
6. Meta text ≥0.8125–0.875rem: `.drow__src`, `.badge`, `.sec__sub`, `.crop label`, `.ai-tag`, `.sec__cites`, `.bar__chip` small text; solid badge border at 0.8 opacity.
7. `.peek__t` on one line ("Rotation · Ready"); ISO dates nowrap in bubbles and rows.
8. Temperature row: state the baseline ("vs 1985–2024 normal") in source/tooltip; rings: partial ring for in progress, amber dot for gaps, lock for Technical sheet; desktop section column shows a "more below" cue so Technical sheet is not hidden.
