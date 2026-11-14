# Post-audit polish (after the final round 3; not re-scored)

Round 3 scores: index 7.9, field selection 8.05, workspace 8.15, dashboard + rotation 7.5, technical sheet 8.0 (all short of 8.5; round limit reached). These are the highest-value remaining items from the final auditors, for one polish pass.

Backend (done): readings now carry `provisional: true` and `note: {en, es}` on the +5.0 °C temperature reading; rain verdict reads "light rain, as usual on the desert coast …"; aphid verdict reads "about 3.5 more generations than normal"; when the farmer cannot say the last fertilizer, Nutrition is set to state "gaps" with note "needs: last fertilizer used".

Frontend:
1. Data honesty: show a visible "provisional" tag (EN/ES) next to every display of a reading with `provisional: true` (hero phone, index NASA-data card, dashboard tile and its preview, section rows, rotation headline that relies on it) and on the latest point of the season chart; add the date range to the hero phone row.
2. Nominatim usage policy: no search-as-you-type. Search on Enter or a Search button only (keep ≤1 req/s, cache results); keep the attribution.
3. Section count honesty: "6 of 6 · 1 with gaps" style, not "6 of 6 ready" when any section has gaps.
4. Chart labels ≥12 px on phone and desktop (season chart, NDVI chart); one-line verdict above the NDVI chart.
5. Phone map reticle on the dashboard: recompute the corners from the outline bounds after the map resizes (currently ~45 px off on phone).
6. Phone first screen of field selection: GPS/Example switch (or an "Example site · Huaral valley, Peru · change" chip) and search visible above the draw tools; instruction line under the title, not cut by the action bar; area/date tag flips left near the right edge.
7. Index mobile hero: no reticle callout behind the CTAs; corner instruments not over the hero phone's photos; leader line ends below the coordinate text (hero and final CTA); label plates on bright desert.
8. Workspace: desktop backdrop graded darker rather than blurred so the terrain reads; top/bottom fade mask on the chat scroll; badges and small labels ≥13 px.
9. Remove the amber dot from the Pest care icon on the rotation view (amber = the one key number); attach the projection "+" to its summary text; shorten/fill the half-empty crop sequence card.
