// App state: the current analysis and view, with a tiny subscribe/notify store and URL sync.
import type { ClientAnalysis } from '../shared/api';
import type { SectionKey } from '../shared/contract';

export type View = 'chat' | 'dashboard' | 'rotation' | 'organic';

export interface AppState {
  analysis: ClientAnalysis | null;
  view: View;
  /** Section open in the phone bottom sheet / expanded on desktop. */
  open: SectionKey | 'organic' | null;
  busy: boolean;
  snapshotCredit: string;
}

type Fn = (s: AppState, changed: Partial<AppState>) => void;
const subs = new Set<Fn>();

export const state: AppState = { analysis: null, view: 'chat', open: null, busy: false, snapshotCredit: '' };

export function set(patch: Partial<AppState>): void {
  Object.assign(state, patch);
  subs.forEach((f) => f(state, patch));
}

export function subscribe(f: Fn): () => void { subs.add(f); return () => subs.delete(f); }

export function syncUrl(): void {
  try {
    const u = new URL(location.href);
    const a = state.analysis;
    ['site', 'go'].forEach((k) => u.searchParams.delete(k));
    if (a && !a.readOnly) u.searchParams.set('a', a.id);
    if (state.view !== 'chat') u.searchParams.set('tab', state.view); else u.searchParams.delete('tab');
    history.replaceState(null, '', u);
  } catch { /* ignore */ }
}

export const VIEWS: View[] = ['chat', 'dashboard', 'rotation', 'organic'];
export const isView = (v: string | null): v is View => !!v && (VIEWS as string[]).includes(v);
