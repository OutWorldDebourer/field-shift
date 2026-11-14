# Round 1 audit findings (app surfaces), for the fix pass

Scores and summary lines are in `audit-log.md`. Fixes below are the auditors' lists, condensed, ordered by impact. Backend items are marked (backend) and are handled by the coordinator.

## Index with globe (6.35)
1. Globe: clouds are hard-edged white blotches over the Andes/Amazon and cover the valley; use a softer, thinner cloud layer (lower opacity, wider smoothstep, blurrier mip) and keep the coast clear. (globe)
2. Globe: sun is a pale grey disc; needs the saturated orange core with bloom of approved-orbit-shot1; clear blue atmosphere rim; lit relief feeling. (globe)
3. `.data__leaders`: draw a leader only when its card and the `.data` section are on screen; fade with visibility (lines currently cross How it works and run off the top).
4. `.top` header: add a night backdrop (rgba(3,6,12,.55) + blur) once scrolled so content never shows through "Field Shift".
5. `.hud` fixed corner instruments collide with section titles/cards: bottom scrim on desktop or hide bl/br while content overlaps the bottom 90 px.
6. Mobile: hide ground labels/reticle while hero CTAs or `.get` content overlap them.
7. Mobile hero: lede sits on the bright sunrise band; add a local dark scrim behind `.hero__text` or reframe.
8. Desktop 1440x900: hero taller than 900 px so "Scroll to descend" is cut; shrink the phone at max-height 950 px.
9. One amber number per screen: neutral DEMO DATA pill, white chart end label, neutral alert icons; crop sequence panel in cartographic blue and not mostly empty.
10. Farmer wording in alerts: "Aphid risk: higher than normal, about 3–4 extra generations", figures in the mono source line; remove duplicated NDVI card on mobile preview.

## Field selection (5.9)
1. Phone: sticky "Use this field" covers the search input; use a fixed opaque bottom action bar with safe-area padding and matching panel padding.
2. Blank EOX tile in the middle of the map: tileerror retry + errorTileUrl in a neutral imagery tone; container background not flat.
3. Corner instruments missing: mono ALT (from zoom), scale bar, imagery line "Sentinel-2 L2A · 2026-11-13"; attribution prefix off ("Leaflet (BSD-2)"), licence via an info button/short line.
4. Imagery date next to the field ("Image: 13 Nov 2026"; before snapshot: "Base image 2016, may be outdated").
5. Open on Example site by default (locked reticle + area visible); My GPS triggers location itself (no second Locate me); hide Clear until there is something to clear.
6. Outdoor contrast: panel more opaque (~0.86), lede/hints/status near-white ≥13 px, credit ≥12 px, disabled button opaque with a reason ("Draw or pick a field first").
7. Cap maxZoom ~17 or use the dated snapshot for the field view; feather the snapshot overlay edge.
8. Desktop panel void (~150 px) from `.fs__readout{margin-top:auto}`.
9. Phone credit collides with scale bar; one-line credit; full-screen map toggle while drawing on phone.
10. Example list: drop the "coming soon" line (only Huaral is real).

## Analysis workspace (7.45)
1. Phone: `.crops` grid → minmax(0,1fr) minmax(0,1fr); `.crop input{width:100%;min-width:0}` ("Grew before" overflows).
2. `dataRowCompact`: add the date to the source line and render a real badge for both field-level and regional.
3. Keep a one-line imagery credit with date on phone (currently `.ws__credit{display:none}`).
4. `.drow--compact .drow__src` ≥0.75rem near-white; non-breaking space in "10 m", "~50 km"; `.secs__count`, `.ws__credit` ≥0.8125rem.
5. Phone header in two rows (brand + Demo + EN|ES; field chip); consider collapsing the crop row once both crops are set.
6. Phone bottom-sheet peek strip with the most recently updated section.
7. Ring states distinguishable: thinner ring, dim track, check/fill when ready, amber dot + note with gaps.
8. Chat history language: scripted/demo bubbles stored as finished strings; render consistently (or at least do not mix languages).
9. Ground imagery smoothed (image-rendering:auto, more blur outside the outline, field interior sharp).
10. Demo flag not amber (white outline + amber dot); sparkline for water balance when series exists.

## Dashboard + rotation (6.75)
1. Rotation mini rows: Soil and Nutrition summaries are disclaimers; show first 2–3 item titles (e.g. "Green manure · Trichoderma compost · no residue piles"). (backend: summaries fixed at source too)
2. Recommended strategy + confidence: backend will set `best_for` on one strategy and a `prediagnosis` {headline, confidence}; render `.is-best` and the confidence bar. (backend)
3. Tiles too tall: short source line on tiles ("NASA POWER · 11 Oct–9 Nov · ~50 km"), full geometry in a title/tooltip; bring alerts + calendar + projection into the first desktop view; chart month axis visible.
4. Mobile: EN|ES in the header row; pre-diagnosis pill merged or shown once; `.dmap__layers` as a 2×2 grid; first tile badge not under the fixed bar.
5. Map field fill: lower saturation, ~0.25 opacity, white hairline, the corner reticle; label as field average.
6. Strategy columns aligned (subgrid or fixed row heights); chips not truncated ("Green man…"); drop the duplicate "Current ·"/"(current plan)".
7. Risk row renamed "Low risk" so more dots = better everywhere; remove the per-column note.
8. Legibility: `.drow__src`, `.dmap__src`, `.strat__tl`, `.strat__tag` ≥0.8125rem in --ink; one-line compact map credit.
9. Sparklines: backend will add series to rain and water balance readings (backend); Soil moisture layer aria-disabled with "not available for this field" when no reading.
10. Plan legend points to a blue plan that is hidden: draw it visibly or remove; notice once on Rotation desktop.
