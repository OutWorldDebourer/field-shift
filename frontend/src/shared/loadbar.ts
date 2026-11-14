// Loading bar for NASA data requests: a label, the source being queried (cycling through the sources the request
// reads) and a bar that moves toward ~92 % over the expected time, then fills and fades when the data arrives.
// The fill is a transform (cheap); the timer runs only while a bar is visible. Requests are single calls, so the
// bar shows elapsed time against the usual duration, not a byte count, and never claims to be done before it is.
import { esc } from './dom';
import { t } from './i18n';
import './loadbar.css';

export const NASA_SOURCES = ['NASA POWER', 'SMAP L4', 'GPM IMERG', 'Sentinel-2 L2A', 'Landsat C2', 'NASADEM', 'NASA GEOS-FP', 'NEX-GDDP-CMIP6'];

export function loadbarHtml(label: string, cls = ''): string {
  return `<div class="lb ${cls}" role="progressbar" aria-label="${esc(label)}" aria-valuemin="0" aria-valuemax="100" aria-valuenow="0">
    <p class="lb__top"><span class="lb__l">${esc(label)}</span><span class="lb__src mono" aria-hidden="true"></span></p>
    <span class="lb__track"><i class="lb__fill"></i></span></div>`;
}

export interface Loadbar { done(): void; fail(): void }

/** Animate a bar rendered with loadbarHtml. `expectMs`: usual duration; `sources`: names shown one after another. */
export function startLoadbar(el: HTMLElement, expectMs: number, sources: string[] = [], startedAt = performance.now()): Loadbar {
  const fill = el.querySelector<HTMLElement>('.lb__fill');
  const src = el.querySelector<HTMLElement>('.lb__src');
  const t0 = startedAt;
  let stopped = false;
  const tick = () => {
    if (stopped || !el.isConnected) { clearInterval(id); return; }
    const dt = performance.now() - t0;
    const p = 0.92 * (1 - Math.exp(-dt / (expectMs / 2.2)));
    if (fill) fill.style.transform = `scaleX(${p.toFixed(3)})`;
    el.setAttribute('aria-valuenow', String(Math.round(p * 100)));
    if (src && sources.length) {
      const s = sources[Math.floor(dt / 1100) % sources.length];
      if (src.dataset.s !== s) { src.dataset.s = s; src.textContent = t('lb_querying', { s }); }
    }
  };
  const id = setInterval(tick, 120);
  tick();
  const end = (ok: boolean) => {
    if (stopped) return;
    stopped = true; clearInterval(id);
    if (fill) fill.style.transform = `scaleX(${ok ? 1 : 0})`;
    el.setAttribute('aria-valuenow', ok ? '100' : '0');
    el.classList.add(ok ? 'is-done' : 'is-failed');
    if (src && ok) src.textContent = t('lb_done');
  };
  return { done: () => end(true), fail: () => end(false) };
}
