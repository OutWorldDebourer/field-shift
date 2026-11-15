// "Ready" light memory (per analysis, in this browser): which views the farmer has already opened and whether the
// sheet/package was exported. The neon call to action (complete.css) goes out for what is already done, so it never
// keeps shouting. Storage can be blocked: then the light simply shows again on the next visit.
const key = (id: string, k: string) => `fs-light:${k}:${id}`;

export function seenViews(id: string): Set<string> {
  try { return new Set((localStorage.getItem(key(id, 'seen')) ?? '').split(',').filter(Boolean)); } catch { return new Set(); }
}

export function markSeen(id: string, view: string): void {
  const s = seenViews(id);
  if (s.has(view)) return;
  s.add(view);
  try { localStorage.setItem(key(id, 'seen'), [...s].join(',')); } catch { /* blocked */ }
}

export function exported(id: string): boolean {
  try { return localStorage.getItem(key(id, 'export')) === '1'; } catch { return false; }
}

export function markExported(id: string): void {
  try { localStorage.setItem(key(id, 'export'), '1'); } catch { /* blocked */ }
}
