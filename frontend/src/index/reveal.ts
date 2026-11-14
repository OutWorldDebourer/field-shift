// Scroll reveal for the index: text types itself out with a caret and figures count up as each block comes into view.
// Built to stay cheap: one text node per block is extended once per frame (no element per letter, no blur or
// transforms), the final size is reserved by an invisible copy so nothing below moves, and one
// IntersectionObserver detects blocks. Screen readers get the full text at once. Off under reduced motion.

const TYPE = [
  '.how__eyebrow [data-i18n]', '.sec-title [data-i18n]', '.sec-lede [data-i18n]', '.how__hook-t [data-i18n]',
  '.step__t [data-i18n]', '.step__b [data-i18n]', '.step__c [data-i18n]', '.stat__k [data-i18n]',
  '.ds__what [data-i18n]', '.get__item h3 [data-i18n]', '.get__item p [data-i18n]', '.get__note [data-i18n]',
  '.cta__title [data-i18n]',
].join(', ');

type Typed = { el: HTMLElement; text: string; out: Text; start: number; step: number; shown: number };
type Fig = { node: Text; to: number; pre: string; post: string; start: number };

/** Time per letter: short texts type at a readable pace, long ones never take more than ~0.75 s. */
const stepFor = (n: number) => Math.max(3, Math.min(16, 750 / n));

/** Reserve the final size and leave an empty text node to type into. */
function prepare(el: HTMLElement): Typed | null {
  const text = el.textContent ?? '';
  if (!text.trim()) return null;
  const sr = document.createElement('span'); sr.className = 'sr-only'; sr.textContent = text;
  const ghost = document.createElement('span'); ghost.className = 'tw-ghost'; ghost.setAttribute('aria-hidden', 'true'); ghost.textContent = text;
  const type = document.createElement('span'); type.className = 'tw-type'; type.setAttribute('aria-hidden', 'true');
  const out = document.createTextNode('');
  type.append(out);
  el.replaceChildren(sr, ghost, type);
  el.classList.add('tw');
  return { el, text, out, start: 0, step: stepFor(text.length), shown: 0 };
}

function numberNode(el: HTMLElement): Omit<Fig, 'start'> | null {
  const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT);
  let found: Omit<Fig, 'start'> | null = null;
  for (let n = walker.nextNode() as Text | null; n; n = walker.nextNode() as Text | null) {
    const m = /^(\D*?)(\d+)(.*)$/s.exec(n.data);
    if (m && !n.parentElement?.closest('small')) found = { node: n, pre: m[1], to: Number(m[2]), post: m[3] };
  }
  return found;
}

export function mountReveal(root: HTMLElement): void {
  if (matchMedia('(prefers-reduced-motion: reduce)').matches || typeof IntersectionObserver === 'undefined') return;
  const typed = new Map<Element, Typed>();
  root.querySelectorAll<HTMLElement>(TYPE).forEach((el) => {
    if (el.closest('.hero, .preview')) return;
    const t = prepare(el);
    if (t) typed.set(el, t);
  });
  // "0 readings without a source" counts down from 100; every other figure counts up from 0.
  const figs = new Map<Element, Fig>();
  root.querySelectorAll<HTMLElement>('.stat__v, .how__hook-v').forEach((el) => {
    const f = numberNode(el);
    if (!f) return;
    figs.set(el, { ...f, start: 0 });
    f.node.data = `${f.pre}${f.to === 0 ? 100 : 0}${f.post}`;
  });

  // One animation loop for everything that is currently typing or counting.
  const active = new Set<Typed | Fig>();
  let raf = 0;
  const finish = (t: Typed) => { if (t.el.isConnected && t.el.classList.contains('tw')) { t.el.textContent = t.text; t.el.classList.remove('tw', 'is-on'); } };
  const frame = (now: number) => {
    for (const a of active) {
      if ('text' in a) {
        // A language switch replaced the text meanwhile: stop.
        if (!a.el.contains(a.out)) { active.delete(a); a.el.classList.remove('tw', 'is-on'); continue; }
        if (now < a.start) continue;
        a.el.classList.add('is-on');
        const n = Math.min(a.text.length, Math.floor((now - a.start) / a.step) + 1);
        if (n !== a.shown) { a.shown = n; a.out.data = a.text.slice(0, n); }
        if (n >= a.text.length) { active.delete(a); finish(a); }
      } else {
        if (now < a.start) continue;
        const p = Math.min(1, (now - a.start) / 1100);
        const from = a.to === 0 ? 100 : 0;
        a.node.data = `${a.pre}${Math.round(from + (a.to - from) * (1 - Math.pow(1 - p, 3)))}${a.post}`;
        if (p >= 1) active.delete(a);
      }
    }
    raf = active.size ? requestAnimationFrame(frame) : 0;
  };
  const run = () => { if (!raf && active.size) raf = requestAnimationFrame(frame); };

  // The root is stretched far upwards, so a block jumped over by an anchor link also reports in: it is shown at once.
  const io = new IntersectionObserver((entries) => {
    const now = performance.now();
    let at = 0;
    const hits = entries.filter((e) => e.isIntersecting)
      .sort((a, b) => (a.target.compareDocumentPosition(b.target) & Node.DOCUMENT_POSITION_FOLLOWING ? -1 : 1));
    for (const e of hits) {
      io.unobserve(e.target);
      const passed = e.boundingClientRect.bottom < 0;
      const t = typed.get(e.target);
      if (t) {
        if (passed) { finish(t); continue; }
        t.start = now + at;
        at = Math.min(900, at + Math.min(260, t.text.length * t.step * 0.4));
        active.add(t);
      } else {
        const f = figs.get(e.target);
        if (!f) continue;
        if (passed) { f.node.data = `${f.pre}${f.to}${f.post}`; continue; }
        f.start = now + Math.min(at, 250);
        active.add(f);
      }
    }
    run();
  }, { rootMargin: '100000px 0px -10% 0px' });
  typed.forEach((_, el) => io.observe(el));
  figs.forEach((_, el) => io.observe(el));
}
