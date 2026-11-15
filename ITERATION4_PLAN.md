# Iteration 4 plan: honest pests, lean consultant, compact chat, prevention and organic

Run it with the `/goal` command (`.claude/commands/goal.md`): `/goal` does the next unchecked phase, `/goal 0` (or `A`…`E`) a given one, `/goal all` every remaining phase in order. Owner requests of 2026-11-14 (Spanish, in the session) are the source of this plan.

The standing rules of `BUILD_PLAN.md` apply to every step (EN default + ES, modular files with folder READMEs, secrets never printed, cite every source, pre-diagnosis wording, design audit gate for UI surfaces, commit + push after each phase, never force-push, never create accounts or spend beyond what this plan allows).

## Status

| Phase | Title | State |
|---|---|---|
| 0 | Honest pests, clean start, one intake, lean API calls | [x] |
| A | Compact chat and floating chat bubble | [x] |
| B | The consultant knows every view | [x] |
| C | Organic by categories; "What if I grow…?" | [x] |
| D | Knowledge research (pest conditions, micronutrients, biostimulants); Prevention tab | [x] |
| E | Technical sheet and consultant with everything above | [ ] |

Tick a phase only when every acceptance check below it passed in the running app (desktop 1440 px and phone 390 px, EN and ES) and the commit is pushed. Add one line per finished phase to the Decision log in `BUILD_PLAN.md`.

---

## Phase 0 · Honest pests, clean start, one intake, lean API calls (urgent)

**Why.** Audit of 2026-11-14:
- Every field gets the same three "reference" pests (green peach aphid *Myzus persicae*, whitefly *Bemisia tabaci* MEAM1, fall armyworm *Spodoptera frugiperda*), hard-coded in `backend/crates/power/src/pests.rs`, whatever the crop. They flow into the "pest generations" tile (`evidence/src/from_power.rs`), alerts, pest-care advice (`crops/src/advice/care.rs` "green peach aphid reference model"), rotation signals (`crops/src/advice/rotation.rs`), `evidence/src/advise.rs`, the organic timing, and every `get_evidence` the consultant reads. A vineyard or an avocado orchard gets aphid alerts.
- The consultant never sees other analyses (history is per analysis), but the field selection restores the last marked field from `localStorage` (`fieldshift.lastField` in `frontend/src/app/field/select.ts`), so a new analysis looks pre-filled.
- Cost: each turn runs a tool loop (up to 8 rounds) that re-sends everything. `get_advice` returns the full bilingual sections (~44 000 characters), `get_evidence` rebuilds the package and returns ~34 000 characters (11 000 of them the NDVI series). Only the system prompt is cached.
- The guided topics ask one topic per message: many paid turns.

**Tasks**
- [x] 0.1 Pests of the real crop. Evidence `pests`, the pest tile, alerts, care advice, rotation signals and organic timing use the current crop's own pests (`evidence/src/crop_pests.rs::generations` with the crop record thresholds), plus crops named later in the chat. The three reference pests are used only while no crop is known, labelled "general reference, not your crop" (EN/ES). The tile title names the crop's leading pest ("Spider mite generations"). Remove the "green peach aphid reference model" wording. Keep citations.
- [x] 0.2 Clean start. "Start analysis" opens an empty map; a visible "Use my last field" button restores the saved one only when pressed. Add a backend test asserting that a turn's request body contains only this analysis's messages, context note and tool results.
- [x] 0.3 One intake, zero model calls. After the field is chosen, open the intake form as the first step (one screen: crop now and before, irrigation, soil, organic matter and residues, last fertilizer, soil test, pests seen, next crop, harvest use, optional photos). Submitting runs the existing `POST /api/analyses/{id}/intake` path (no Claude call except AI research of an unknown crop). "Skip, I'd rather chat" stays available. When information is missing, the consultant asks for all of it in one checklist message instead of one topic per message (prompt rule in `consultant/src/prompt.rs`).
- [x] 0.4 Lean, measured calls. Tool outputs carry only what the model needs, in the conversation language: `get_evidence` returns the stored package compacted (readings, forecast summary, pests; no full NDVI series) without rebuilding it; `get_advice` returns section summaries and item titles, not the full bilingual content. Add `cache_control` on the last message block so tool-loop rounds reuse the cache. Log `usage` (input, output, cache read, cache write tokens) per call and per turn (tracing + a `usage` table or log line), and report before/after numbers for one Huaral conversation in the Decision log.

**Acceptance**
- A new analysis for a vineyard or avocado never shows aphid, whitefly or armyworm unless they are that crop's recorded pests; the Huaral pea analysis shows pea pests.
- "Start analysis" shows an empty map; the last field appears only after "Use my last field".
- Completing the intake fills all five topics, the sections and the sheet with zero Claude calls (check the usage log).
- Tokens per consultant turn drop measurably (numbers in the Decision log).
- `cargo test` passes, including the new isolation test.

---

## Phase A · Compact chat and floating bubble

**Tasks**
- [x] A.1 Compact header: one line "pea now · lettuce before · Edit" plus an icon button for "Load information".
- [x] A.2 Topic rail becomes a thin progress bar with six dots and the active topic name; expands on click.
- [x] A.3 Lighter messages: slightly smaller type, narrower bubbles, 28 px avatar, less empty space.
- [x] A.4 Quick replies: two or three suggestions under the last consultant message, from the active topic and the recommended topics.
- [x] A.5 One-row composer: camera, text, microphone; the Gemini/Browser voice choice moves into a microphone menu.
- [x] A.6 Narrower chat column so more of the map shows.
- [x] A.7 Floating bubble: leaving the Chat tab animates the chat into a bubble (bottom right; above the bottom bar on phone) with an unread dot and a hover preview of the last message. Clicking opens a mini chat (about 380 × 540 px; a bottom sheet on phone) over the current view, using the same chat instance moved in the DOM (draft text and pending photos survive). Header says "Asking about: Rotation". Esc closes; focus returns to the bubble; reduced motion shows it without animation.

**Acceptance:** chat audit round by a fresh auditor (BUILD_PLAN audit gate); bubble works on every tab, phone and desktop; no console errors.

---

## Phase B · The consultant knows every view

**Tasks**
- [x] B.1 New consultant tool `get_view(view)` (`dashboard`, `rotation`, `organic`, `prevention`, `sheet`) returning exactly the data each view renders (the same builders the frontend uses, compacted, in the conversation language).
- [x] B.2 The frontend sends the current view and focused item with each turn (e.g. `rotation · strategy 2`); the context note says "The farmer is looking at …".
- [x] B.3 "Ask about this" button on cards (a reading, a strategy, an agent, a pest) that opens the chat (or the bubble) with the question prefilled.

**Acceptance:** asking "why does this one have low risk?" from Rotation strategy 2 gets an answer about strategy 2 without naming it; the tool returns the same numbers the view shows.

---

## Phase C · Organic by categories; "What if I grow…?"

**Tasks**
- [x] C.1 Organic view redesign: a three-line "this week" summary, then category cards, each with an icon, one plain sentence and a count, and a disclosure with the technical core (scientific names, how it acts, citation, data source):
  natural enemies grouped by pest ("For the aphid: 3 allies"); beneficial fungi and bacteria (Trichoderma, Beauveria, Bacillus…); biostimulants and micronutrients (filled in Phase D); practices; when to act (NASA data); what to avoid.
- [x] C.2 "Evaluate your idea" in Rotation: a field to type any crop; a fourth column "Your proposal" scored with the same engine (soil health, water use against the NASA water balance, pest-cycle break by botanical family, income, risk) with why / evidence / gaps. Endpoint `POST /api/analyses/{id}/rotation/evaluate {crop}` (crop resolved from the knowledge base, AI research only for unknown crops).
- [x] C.3 Consultant tool `evaluate_crop` so "what do you think of growing avocado?" in the chat runs the same evaluation and adds the card.

**Acceptance:** Organic reads clearly at first glance, details one click away (audit round); a proposed crop appears as a fourth scored column, from the form and from the chat; perennials are evaluated as perennials (no "rotate the main crop").

---

## Phase D · Knowledge research; Prevention tab

**Cost rule:** research uses paid API calls. Start with the five Huaral demo crops (pea, lettuce, sweet potato, maize, potato). Researching the other 13 seed crops needs the owner's yes.

**Tasks**
- [x] D.1 Extend crop records (`backend/crates/crops/data/*.json`, `model.rs`) with, each value cited or declared a gap:
  per pest, the conditions that favour it (temperature range, dry or humid, season) — e.g. spider mite favoured by hot, dry weather;
  critical micronutrients for the crop (Zn, B, Fe, Mn, Mo…) with deficiency symptoms;
  biostimulant categories with evidence for the crop (amino acids under heat or salinity stress, seaweed extracts, humic and fulvic substances).
  No product names or doses (agronomist's call). Research through the existing pipeline (`crops/src/research.rs`, web search, cite or gap); review and store in the seed JSON; label anything unreviewed "AI-researched, verify".
- [x] D.2 Prevention tab (top bar: Chat, Dashboard, Rotation, Prevention, Organic): for every crop in the conversation (current, previous, evaluated), each pest gets a traffic light (low / medium / high) with a plain reason, from NASA degree-day generations vs normal, the GEOS-FP 10-day forecast and the favourable conditions of D.1 ("Spider mite: high risk. Hotter than normal (+5 °C) and dry days up to 30 °C ahead"). Perennials get a year-round monitoring calendar; annuals a calendar by stage (sowing to harvest). Actions: monitoring, cultural practices, when to release natural enemies (links to Organic). Same summary + disclosure pattern.
- [x] D.3 Organic "biostimulants and micronutrients" category filled from D.1, linked to signals (low vigor → check micronutrients; no soil test → gap).

**Acceptance:** the five Huaral crops carry cited D.1 data; Prevention shows correct, explained risks for pea in Huaral and updates when a crop is added in the chat; audit round passes or is logged.

---

## Phase E · Technical sheet and consultant with everything

**Tasks**
- [ ] E.1 Sheet sections for "Your proposal", Prevention and Biostimulants & micronutrients, same summary + detail format, also in the ZIP.
- [ ] E.2 Consultant tools for prevention (`get_view('prevention')`) and the final closing message pointing to them.
- [ ] E.3 Handbook (EN + ES): how it works, references (new sources), AI use. `AI_USE.md` updated. Rebuild with `node handbook/build.mjs`.

**Acceptance:** sheet and ZIP show the new sections with sources; handbook rebuilt; final audit of the sheet.
