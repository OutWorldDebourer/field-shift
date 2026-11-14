// The guided topic sequence (field → soil → pests → nutrition → rotation → sheet). Uses the backend's `steps`
// when present; otherwise derives them from the section states, so the rail and the column always agree.
import type { ClientAnalysis } from '../../shared/api';
import type { Bi, Section, SectionKey, SectionState, Step } from '../../shared/contract';
import { STRINGS, type StrKey } from '../../shared/strings';

export const ORDER: SectionKey[] = ['field', 'soil', 'pests', 'nutrition', 'rotation', 'sheet'];
export const TITLE: Record<SectionKey, StrKey> = { field: 'sec_field', soil: 'sec_soil', pests: 'sec_pests', nutrition: 'sec_nutrition', rotation: 'sec_rotation', sheet: 'sec_sheet' };
export const SHORT: Record<SectionKey, StrKey> = { field: 'sec_field_short', soil: 'sec_soil', pests: 'sec_pests_short', nutrition: 'sec_nutrition', rotation: 'sec_rotation', sheet: 'sec_sheet_short' };

export function getSection(a: ClientAnalysis, k: SectionKey): Section {
  return a.sections.find((s) => s.key === k) ?? { key: k, state: k === 'sheet' ? 'locked' : 'empty' };
}

const isDone = (s: SectionState) => s === 'ready' || s === 'gaps';
const bi = (k: StrKey): Bi => ({ en: STRINGS[k].en, es: STRINGS[k].es });

/** Backend steps in canonical order, or steps derived from the sections. Exactly one step is active until all are done. */
export function stepsOf(a: ClientAnalysis): Step[] {
  const remote = Array.isArray(a.steps) ? a.steps.filter((s) => s && ORDER.includes(s.key)) : [];
  if (remote.length) {
    const sorted = ORDER.map((k) => remote.find((s) => s.key === k)).filter((s): s is Step => !!s)
      .map((s) => ({ ...s, title: s.title ?? bi(TITLE[s.key]), needs: Array.isArray(s.needs) ? s.needs : [] }));
    if (!sorted.some((s) => s.state === 'active')) {
      const next = sorted.find((s) => s.key === a.next_step) ?? sorted.find((s) => s.state !== 'done');
      if (next) next.state = 'active';
    }
    return sorted;
  }
  const secs = ORDER.map((k) => getSection(a, k));
  const fiveDone = secs.slice(0, 5).every((s) => isDone(s.state));
  const steps: Step[] = secs.map((s) => ({
    key: s.key,
    state: (s.key === 'sheet' ? fiveDone && s.state !== 'locked' : isDone(s.state)) ? 'done' : 'pending',
    title: bi(TITLE[s.key]),
    needs: !isDone(s.state) && s.note ? [s.note] : [],
    summary: s.content?.summary,
  }));
  const first = steps.find((s) => s.state !== 'done');
  if (first) first.state = 'active';
  return steps;
}

export const allDone = (steps: Step[]): boolean => steps.length > 0 && steps.every((s) => s.state === 'done');

/** Keys that are done now but were not before. */
export function newlyDone(prev: Step[], next: Step[]): SectionKey[] {
  return next.filter((s) => s.state === 'done' && prev.find((p) => p.key === s.key)?.state !== 'done').map((s) => s.key);
}

/** The state the section column shows: the step decides pending / active / done; the section adds "with gaps". */
export function displayState(a: ClientAnalysis, k: SectionKey, steps = stepsOf(a)): { state: SectionState; step?: Step } {
  const s = getSection(a, k);
  const st = steps.find((x) => x.key === k);
  if (!st || !Array.isArray(a.steps) || !a.steps.length) return { state: s.state, step: st };
  if (st.state === 'done') return { state: s.state === 'gaps' ? 'gaps' : 'ready', step: st };
  if (st.state === 'active') return { state: 'progress', step: st };
  return { state: k === 'sheet' ? 'locked' : s.state === 'progress' ? 'progress' : 'empty', step: st };
}
