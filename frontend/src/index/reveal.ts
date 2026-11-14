// Scroll reveal for the index: text types itself out letter by letter and figures count up as each block comes
// into view. Screen readers get the full text at once (a visually hidden copy); the letters are aria-hidden.
// With "reduce motion" nothing is split or animated. A language switch replaces the text, which then shows whole.

const TYPE = [
  '.how__eyebrow [data-i18n]', '.sec-title [data-i18n]', '.sec-lede [data-i18n]', '.how__hook-t [data-i18n]',
  '.step__t [data-i18n]', '.step__b [data-i18n]', '.step__c [data-i18n]', '.stat__k [data-i18n]',
  '.ds__what [data-i18n]', '.get__item h3 [data-i18n]', '.get__item p [data-i18n]', '.get__note [data-i18n]',
  '.cta__title [data-i18n]',
].join(', ');

const reduced = () => matchMedia('(prefers-reduced-motion: reduce)').matches;

/** Split an element's text into words of letters, hidden until the element is revealed. Returns its duration (ms). */
function split(el: HTMLElement): number {
  const text = el.textContent ?? '';
  if (!text.trim()) return 0;
  const n = text.length;
  const step = Math.max(5, Math.min(26, 1100 / n));
  el.style.setProperty('--tw-step', `${step.toFixed(1)}ms`);
  const sr = document.createElement('span');
  sr.className = 'sr-only';
  sr.textContent = text;
  const vis = document.createElement('span');
  vis.setAttribute('aria-hidden', 'true');
  let i = 0;
  for (const part of text.split(/(\s+)/)) {
    if (!part) continue;
    if (/^\s+$/.test(part)) { vis.append(document.createTextNode(part)); i += part.length; continue; }
    const w = document.createElement('span');
    w.className = 'tw-w';
    for (const ch of part) {
      const c = document.createElement('span');
      c.className = 'tw-c';
      c.style.setProperty('--i', String(i++));
      c.textContent = ch;
      w.append(c);
    }
    vis.append(w);
  }
  el.replaceChildren(sr, vis);
  el.classList.add('tw');
  return n * step;
}

/** The figure in a stat or the hook: the last text node holding a number. */
function numberNode(el: HTMLElement): { node: Text; to: number; pre: string; post: string } | null {
  const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT);
  let found: { node: Text; to: number; pre: string; post: string } | null = null;
  for (let n = walker.nextNode() as Text | null; n; n = walker.nextNode() as Text | null) {
    const m = /^(\D*?)(\d+)(.*)$/s.exec(n.data);
    if (m && !n.parentElement?.closest('small')) found = { node: n, pre: m[1], to: Number(m[2]), post: m[3] };
  }
  return found;
}

type Fig = { node: Text; to: number; pre: string; post: string };
function count(f: Fig, delay: number): void {
  // "0 readings without a source" counts down from 100; every other figure counts up from 0.
  const from = f.to === 0 ? 100 : 0;
  const dur = 1400;
  f.node.data = `${f.pre}${from}${f.post}`;
  const t0 = performance.now() + delay;
  const tick = (now: number) => {
    const p = Math.max(0, Math.min(1, (now - t0) / dur));
    const e = 1 - Math.pow(1 - p, 3);
    f.node.data = `${f.pre}${Math.round(from + (f.to - from) * e)}${f.post}`;
    if (p < 1) requestAnimationFrame(tick);
  };
  requestAnimationFrame(tick);
}

export function mountReveal(root: HTMLElement): void {
  if (reduced()) return;
  const dur = new Map<Element, number>();
  root.querySelectorAll<HTMLElement>(TYPE).forEach((el) => {
    if (el.closest('.hero, .preview')) return;
    dur.set(el, split(el));
  });
  // Read each target figure before showing its starting value.
  const figs = new Map<Element, Fig>();
  root.querySelectorAll<HTMLElement>('.stat__v, .how__hook-v').forEach((el) => {
    const f = numberNode(el);
    if (!f) return;
    figs.set(el, f);
    f.node.data = `${f.pre}${f.to === 0 ? 100 : 0}${f.post}`;
  });

  // Pending blocks in reading order. Checked on scroll (not an IntersectionObserver), so blocks jumped over by an
  // anchor link are still revealed — at once, since they are already above the screen.
  let pending = [...dur.keys(), ...figs.keys()].sort((a, b) => (a.compareDocumentPosition(b) & Node.DOCUMENT_POSITION_FOLLOWING ? -1 : 1)) as HTMLElement[];
  const check = () => {
    if (!pending.length) return;
    const H = innerHeight;
    let at = 0;
    pending = pending.filter((el) => {
      const r = el.getBoundingClientRect();
      if (r.top > H * 0.88) return true;
      const passed = r.bottom < 0;
      if (dur.has(el)) {
        // A language switch since the split leaves plain text: nothing to type.
        if (el.querySelector('.tw-c')) {
          if (passed) el.style.setProperty('--tw-step', '0ms');
          el.style.setProperty('--tw-delay', `${passed ? 0 : at}ms`);
          el.classList.add('is-on');
          if (!passed) at = Math.min(1200, at + Math.min(320, (dur.get(el) ?? 0) * 0.3));
        }
      } else {
        const f = figs.get(el)!;
        if (passed) f.node.data = `${f.pre}${f.to}${f.post}`; else count(f, Math.min(at, 300));
      }
      return false;
    });
    if (!pending.length) removeEventListener('scroll', onScroll);
  };
  let ticking = false;
  const onScroll = () => { if (!ticking) { ticking = true; requestAnimationFrame(() => { ticking = false; check(); }); } };
  addEventListener('scroll', onScroll, { passive: true });
  addEventListener('resize', onScroll);
  requestAnimationFrame(check);
}
