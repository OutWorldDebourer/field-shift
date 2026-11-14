# Design audit log

Owner's gate (BUILD_PLAN.md): every UI surface is scored by a fresh Opus auditor that never saw the build conversation, from real 1440 px and 390 px screenshots, the two approved images and the surface's contract. Pass = 8.5/10 or more with no criterion below half its points. At most 3 rounds per surface.

Criteria (points): fidelity to the approved direction 2 · clarity for the farmer 2 · visible working functionality 1.5 · phone and desktop 1.5 · accessibility and outdoor legibility 1 · data honesty 1 · polish and performance 1.

Screenshots of every round: `design-directions/audit-screens/`.

| Date | Surface | Round | Score | Pass | Main findings |
|------|---------|-------|-------|------|---------------|
| 2026-11-14 | Technical sheet | 1 | 5.7 | no | Phone table overflows and hides sources; advice text calls regional POWER signals 'field signal'; pest lists contradict; source column cluttered; citations unusable on paper |
| 2026-11-14 | Technical sheet | 2 | 6.9 | no | Pest guidance contradicts the reference-pest table; repeated gaps/evidence; Spanish unit leaks; print tile/footer; mobile capture broken past 8192 px |
| 2026-11-14 | Field selection | 1 | 5.9 | no | Phone sticky button covers search; blank map tile; no corner instruments; imagery date only in small print; low outdoor contrast |
| 2026-11-14 | Technical sheet | 3 | 8.0 | no (round limit reached) | Value/unit collision in Spanish; '+5.0 °C' not labelled as vs normal; reference vs crop pests confusing; amber used on notice label; number-unit line breaks. Post-audit fixes applied, not re-scored |
| 2026-11-14 | Analysis workspace | 1 | 7.45 | no | Phone crop input overflows; compact rows lack date and real badges; imagery credit hidden on phone; 10 px source text; header too tall on phone; ring states indistinct |
| 2026-11-14 | Index with globe | 1 | 6.35 | no | Hard-edged clouds and pale sun vs approved flare; leader lines outside their section; no night header backdrop; HUD over content; mobile hero contrast; hero taller than 900 px |
| 2026-11-14 | Dashboard + rotation | 1 | 6.75 | no | Soil/nutrition rows show disclaimers; no recommended strategy or confidence; tiles too tall (alerts/calendar below fold); mobile chrome; flat lime map fill; risk dots inverted wording |
| 2026-11-14 | Index with globe | 2 | 7.35 | no | Corner HUD still covers section titles; mobile reticle labels over copy; leader lines through label text; mobile credit cut; small hero phone text; fog gap before dashboard preview; cloud ellipses |
| 2026-11-14 | Field selection | 2 | 7.25 | no | Image date shown for fields outside the dated snapshot; seam between 2026 snapshot and 2016 base; soft upsampled imagery; phone tools below the fold; unreadable scale bar |
| 2026-11-14 | Dashboard + rotation | 2 | 6.7 (provisional) | no | Full-page captures missed content scrolled inside .view (tool fixed); chart below the fold; projection not labelled 'not a forecast'; mobile layer panel covers map; plan layer not visible; small 11 px labels |
| 2026-11-14 | Analysis workspace | 2 | 7.4 | no | Compact rows show raw numbers without verdicts; demo consultant repeated its summary (test data); phone chat too short; credit and resolution truncated; meta text ~11 px |
