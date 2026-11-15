---
description: Implement the Field Shift iteration 4 plan (ITERATION4_PLAN.md) phase by phase
argument-hint: "[0|A|B|C|D|E|all]  (empty = next unchecked phase)"
---

# /goal — Field Shift iteration 4

Goal: implement `ITERATION4_PLAN.md` at the repository root, phase by phase, until every phase is ticked.

Phase requested: `$ARGUMENTS` (empty → the first phase whose Status row is `[ ]`; `all` → every remaining phase in order, one after another).

## Before starting
1. Read `ITERATION4_PLAN.md` in full, then the "Standing rules" of `BUILD_PLAN.md`. Those rules bind every step (EN default + ES strings, modular files and folder READMEs, secrets never printed or committed, every source cited with its licence, pre-diagnosis wording, design audit gate for UI surfaces, `AI_USE.md` kept current).
2. Talk to the owner in Spanish. Keep code, UI text, docs and commits in English (plus the ES strings).
3. Check `git status`; do not discard anyone's uncommitted work.

## For each phase
1. Work through its tasks in order. Read the files the plan names before changing them; keep changes small and modular.
2. Build and verify for real: `cargo test` and `cargo build --release -p api` in `backend/`, `npx tsc --noEmit -p .` and `npx vite build` in `frontend/`; restart the `fieldshift` preview server (stop it before rebuilding the binary on Windows) and check the feature in the browser pane at 1440 px and 390 px, in EN and ES, with no console errors.
3. Run every "Acceptance" check of the phase. UI surfaces go through the design audit gate (fresh auditor agent, scores in `design-directions/audit-log.md`, at most 3 rounds).
4. Tick the finished tasks and the phase's Status row in `ITERATION4_PLAN.md`; add one line to the Decision log in `BUILD_PLAN.md` (what changed, numbers measured, anything left open).
5. Commit with a clear message ending `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` and push to `origin`. Never force-push.
6. Report to the owner in Spanish: what is now true, what was verified, what is still open.

## Stop and ask the owner first when
- a step would spend API credit beyond what the plan allows (Phase D: only the five Huaral crops without asking);
- a step would create accounts, enter payment data, make anything public or send messages;
- a phase's acceptance check fails after a real attempt to fix it (say what failed and why).

## Cost discipline while building
Use the scripted demo mode (`?demo=1`) to test chat flows that do not need the live model; make live consultant calls only to verify a change that depends on the model, and say so when you do.
