// Pointer and keyboard read-out for the hand-written SVG charts. A chart lists its data points as invisible markers
// (hitsHtml); one delegated listener (installChartHover) snaps the pointer to the nearest point in x, draws a guide
// line and a ring on the value, and shows a small label with the date and the values. Keyboard: the chart is
// focusable, ←/→ move between points, Home/End jump, Esc hides; keyboard moves are also announced to screen readers.
import { esc } from './dom';

export interface Hit { x: number; y: number; k: string; v: string[] }

/** Invisible markers carrying each point's position and read-out (one per x position). */
export function hitsHtml(hits: Hit[]): string {
  return `<g class="hits" aria-hidden="true">${hits.map((h) =>
    `<circle class="hit" r="0" cx="${h.x.toFixed(1)}" cy="${h.y.toFixed(1)}" data-k="${esc(h.k)}" data-v="${esc(h.v.join('\n'))}"/>`).join('')}</g>`;
}

const SVG = 'http://www.w3.org/2000/svg';
let installed = false;

export function installChartHover(): void {
  if (installed) return;
  installed = true;
  document.addEventListener('pointermove', onMove, { passive: true });
  document.addEventListener('pointerdown', onMove, { passive: true });
  document.addEventListener('pointerout', (e) => {
    const svg = (e.target as Element).closest?.('svg[data-hover]') as SVGSVGElement | null;
    if (svg && !svg.contains(e.relatedTarget as Node)) hide(svg);
  });
  document.addEventListener('keydown', onKey);
  document.addEventListener('focusout', (e) => { const svg = (e.target as Element).closest?.('svg[data-hover]') as SVGSVGElement | null; if (svg) hide(svg); });
}

const hitsOf = (svg: SVGSVGElement) => [...svg.querySelectorAll<SVGCircleElement>('.hit')];

function onMove(e: PointerEvent): void {
  const svg = (e.target as Element).closest?.('svg[data-hover]') as SVGSVGElement | null;
  if (!svg) return;
  const ctm = svg.getScreenCTM();
  if (!ctm) return;
  const p = new DOMPoint(e.clientX, e.clientY).matrixTransform(ctm.inverse());
  const hits = hitsOf(svg);
  let best = -1, bd = Infinity;
  hits.forEach((h, i) => { const d = Math.abs(+h.getAttribute('cx')! - p.x); if (d < bd) { bd = d; best = i; } });
  if (best < 0 || bd > 40) { hide(svg); return; }
  show(svg, best, false);
}

function onKey(e: KeyboardEvent): void {
  const svg = (e.target as Element).closest?.('svg[data-hover]') as SVGSVGElement | null;
  if (!svg) return;
  const n = hitsOf(svg).length;
  if (!n) return;
  const cur = Number(svg.dataset.at ?? -1);
  let next = cur;
  if (e.key === 'ArrowRight') next = cur < 0 ? 0 : Math.min(n - 1, cur + 1);
  else if (e.key === 'ArrowLeft') next = cur < 0 ? n - 1 : Math.max(0, cur - 1);
  else if (e.key === 'Home') next = 0;
  else if (e.key === 'End') next = n - 1;
  else if (e.key === 'Escape') { hide(svg); return; }
  else return;
  e.preventDefault();
  show(svg, next, true);
}

function layer(svg: SVGSVGElement): { guide: SVGLineElement; ring: SVGCircleElement } {
  let g = svg.querySelector<SVGGElement>('.hov');
  if (!g) {
    g = document.createElementNS(SVG, 'g'); g.setAttribute('class', 'hov'); g.setAttribute('aria-hidden', 'true');
    const guide = document.createElementNS(SVG, 'line'); guide.setAttribute('class', 'hov__guide');
    const ring = document.createElementNS(SVG, 'circle'); ring.setAttribute('class', 'hov__ring'); ring.setAttribute('r', '5.5');
    g.append(guide, ring); svg.append(g);
  }
  return { guide: g.querySelector('line')!, ring: g.querySelector('circle')! };
}

function tipOf(svg: SVGSVGElement): { tip: HTMLDivElement; live: HTMLParagraphElement } {
  const host = svg.parentElement!;
  if (getComputedStyle(host).position === 'static') host.style.position = 'relative';
  let tip = host.querySelector<HTMLDivElement>(':scope > .chart-tip');
  if (!tip) { tip = document.createElement('div'); tip.className = 'chart-tip'; tip.hidden = true; host.append(tip); }
  let live = host.querySelector<HTMLParagraphElement>(':scope > .chart-live');
  if (!live) { live = document.createElement('p'); live.className = 'chart-live sr-only'; live.setAttribute('aria-live', 'polite'); host.append(live); }
  return { tip, live };
}

function show(svg: SVGSVGElement, i: number, announce: boolean): void {
  const h = hitsOf(svg)[i];
  if (!h) return;
  svg.dataset.at = String(i);
  const cx = +h.getAttribute('cx')!, cy = +h.getAttribute('cy')!;
  const vb = svg.viewBox.baseVal;
  const { guide, ring } = layer(svg);
  guide.setAttribute('x1', String(cx)); guide.setAttribute('x2', String(cx));
  guide.setAttribute('y1', String(vb.y + 12)); guide.setAttribute('y2', String(vb.y + vb.height - 30));
  ring.setAttribute('cx', String(cx)); ring.setAttribute('cy', String(cy));
  svg.classList.add('is-hovering');

  const { tip, live } = tipOf(svg);
  const k = h.dataset.k ?? '', v = (h.dataset.v ?? '').split('\n').filter(Boolean);
  tip.innerHTML = `<b class="chart-tip__k">${esc(k)}</b>${v.map((l) => `<span>${esc(l)}</span>`).join('')}`;
  tip.hidden = false;
  // place above the point, inside the chart's box (flip below near the top, clamp at the sides)
  const host = svg.parentElement!, hr = host.getBoundingClientRect();
  const pt = new DOMPoint(cx, cy).matrixTransform(svg.getScreenCTM()!);
  const px = pt.x - hr.left + host.scrollLeft, py = pt.y - hr.top + host.scrollTop;
  const tw = tip.offsetWidth, th = tip.offsetHeight;
  const minX = host.scrollLeft + 4, maxX = host.scrollLeft + host.clientWidth - tw - 4;
  const left = Math.max(minX, Math.min(maxX, px - tw / 2));
  const top = py - th - 12 < 0 ? py + 14 : py - th - 12;
  tip.style.left = `${left}px`; tip.style.top = `${top}px`;
  if (announce) live.textContent = `${k}: ${v.join('. ')}`;
}

function hide(svg: SVGSVGElement): void {
  delete svg.dataset.at;
  svg.classList.remove('is-hovering');
  const tip = svg.parentElement?.querySelector<HTMLDivElement>(':scope > .chart-tip');
  if (tip) tip.hidden = true;
}
