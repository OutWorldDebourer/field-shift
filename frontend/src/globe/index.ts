// Field Shift globe: one continuous dive from an oblique orbit at sunrise to a locked nadir frame over
// the field, on real NASA and Copernicus imagery. `createGlobe(host, opts)` is the whole public surface.
import { kmPerPx, project } from './camera';
import { clamp, diveFrame, type DiveFrame } from './dive';
import { niceScale, Overlay } from './overlay';
import { creditText, GlobeRenderer } from './renderer';
import { StillView } from './still';
import type { GlobeCamera, GlobeHandle, GlobeOptions, GlobeTarget } from './types';
export type * from './types';

const LOCK_ON_MS = 1700;

export function createGlobe(host: HTMLElement, opts: GlobeOptions): GlobeHandle {
  const base = opts.assetBase ?? `${import.meta.env.BASE_URL}globe/`;
  if (getComputedStyle(host).position === 'static') host.style.position = 'relative';
  const root = document.createElement('div');
  Object.assign(root.style, { position: 'absolute', inset: '0', overflow: 'hidden', background: '#03060c' });
  host.appendChild(root);

  const reduced = typeof matchMedia === 'function' && matchMedia('(prefers-reduced-motion: reduce)').matches;
  let lang: 'en' | 'es' = opts.lang ?? 'en';
  let target: GlobeTarget = opts.target;
  let pTarget = 0;
  let still: StillView | null = null;
  let renderer: GlobeRenderer | null = null;
  let disposed = false;

  const toStill = () => {
    if (disposed || still) return;
    host.dataset.globe = 'still';
    still = new StillView(root, base, { ...opts, lang, target });
    still.setProgress(pTarget);
  };

  if (!opts.still && !reduced) {
    try { renderer = new GlobeRenderer(root, base); } catch { renderer = null; }
  }
  if (!renderer) {
    toStill();
    return {
      setProgress(p) { pTarget = clamp(p); still?.setProgress(pTarget); },
      lockOn(t) { target = t; still?.lockOn(t); },
      setLang(l) { lang = l; still?.setLang(l); },
      resize() { still?.resize(); },
      dispose() { disposed = true; still?.dispose(); root.remove(); delete host.dataset.globe; },
    };
  }

  // ---------------------------------------------------------------- live globe
  host.dataset.globe = 'live';
  const overlay = opts.overlay === false ? null : new Overlay(root);
  const anchor = opts.anchor ?? { x: 0.5, y: 0.5 };
  const creep = opts.creep !== false;
  let W = 1, H = 1, pr = 1, prScale = 1;
  let pShown = 0;
  let look = { lat: target.lat, lon: target.lon };
  let glide: { from: { lat: number; lon: number }; to: { lat: number; lon: number }; t0: number } | null = null;
  let raf = 0, last = 0, t0 = 0, started = false, onScreen = true, dirty = true, parity = false;
  let slow = 0, ema = 16;
  let lastCam = '';

  const basePr = () => opts.pixelRatio ?? Math.min(devicePixelRatio || 1, W * H > 1.1e6 ? 1.5 : 2);
  const size = () => {
    W = Math.max(1, root.clientWidth); H = Math.max(1, root.clientHeight);
    pr = basePr() * prScale;
    renderer?.setSize(W, H, pr);
    overlay?.resize(W, H, Math.min(devicePixelRatio || 1, 2));
    dirty = true;
  };

  const frame = (now: number) => {
    raf = 0;
    if (disposed || !renderer) return;
    if (!started) { started = true; t0 = now; last = now; overlay?.lock(now, 700); }
    const dt = Math.min(100, now - last);
    last = now;

    // ease the shown progress towards the scroll, glide the look-at point on lockOn
    const before = pShown;
    pShown += (pTarget - pShown) * (1 - Math.exp(-dt / 110));
    if (Math.abs(pTarget - pShown) < 2e-5) pShown = pTarget;
    let moving = pShown !== before;
    if (glide) {
      const k = clamp((now - glide.t0) / LOCK_ON_MS), e = k * k * (3 - 2 * k);
      look = { lat: glide.from.lat + (glide.to.lat - glide.from.lat) * e, lon: glide.from.lon + (glide.to.lon - glide.from.lon) * e };
      if (k >= 1) glide = null;
      moving = true;
    }
    const entering = now - t0 < 3200;                        // reticle snap, leader and label drawing on

    // when nothing but the idle creep is happening, draw every other frame
    parity = !parity;
    if (!moving && !entering && !dirty && parity) { schedule(); return; }

    const time = (now - t0) / 1000;
    const f: DiveFrame = diveFrame({ p: pShown, lat: look.lat, lon: look.lon, W, H, anchor, time }, creep);
    const ps = renderer.plateState(f, target, now);
    renderer.render(f, ps.alpha, target, time, now);
    dirty = ps.loading;

    const sc = niceScale(kmPerPx(f.cam), W);
    const credit = creditText(ps.credit, lang);
    overlay?.draw({
      cam: f.cam, target, lang, now, ring: f.ring, label: f.label, settled: false,
      instruments: opts.instruments !== false, altitudeKm: f.cam.alt, scaleBarMeters: sc.meters, scaleBarPx: sc.px, credit,
    });
    if (opts.onCamera) {
      const ts = project(f.cam, target.lat, target.lon);
      const c: GlobeCamera = {
        altitudeKm: f.cam.alt, scaleBarMeters: sc.meters, scaleBarPx: sc.px, imageryCredit: credit,
        targetScreen: ts.visible ? { x: ts.x, y: ts.y } : null, progress: pShown, sunElevationDeg: f.sunEl, mode: 'live',
      };
      // the creep changes the altitude a little every frame: report at the instrument's resolution
      const key = `${c.altitudeKm.toPrecision(3)}|${c.scaleBarMeters}|${Math.round(c.scaleBarPx)}|${credit}|${ts.visible ? Math.round(ts.x) + ',' + Math.round(ts.y) : '-'}|${pShown.toFixed(4)}`;
      if (key !== lastCam) { lastCam = key; opts.onCamera(c); }
    }

    // low power: if frames stay slow while the dive is moving, render fewer pixels
    if (opts.pixelRatio === undefined && moving) {
      ema += (dt - ema) * 0.1;
      slow = ema > 26 ? slow + 1 : 0;
      if (slow > 24 && prScale > 0.55) { prScale = Math.max(0.55, prScale * 0.8); slow = 0; ema = 16; size(); }
    }
    schedule();
  };

  const schedule = () => { if (!raf && !disposed && onScreen && !document.hidden) raf = requestAnimationFrame(frame); };
  const wake = () => { last = performance.now(); schedule(); };

  const ro = typeof ResizeObserver === 'function' ? new ResizeObserver(() => { size(); wake(); }) : null;
  ro?.observe(root);
  const io = typeof IntersectionObserver === 'function'
    ? new IntersectionObserver((es) => { onScreen = es[es.length - 1].isIntersecting; if (onScreen) wake(); else if (raf) { cancelAnimationFrame(raf); raf = 0; } })
    : null;
  io?.observe(root);
  const onVis = () => { if (document.hidden) { if (raf) { cancelAnimationFrame(raf); raf = 0; } } else wake(); };
  document.addEventListener('visibilitychange', onVis);

  renderer.onTexture = () => { dirty = true; };
  renderer.onContextLost = () => { teardownLive(); toStill(); };
  size();
  void renderer.loadAll(() => {
    if (disposed) return;
    schedule();
    requestAnimationFrame(() => { if (!disposed) opts.onReady?.(); });
  });
  if (document.fonts) document.fonts.load('500 16px "IBM Plex Mono"').then(() => { dirty = true; }, () => undefined);

  function teardownLive() {
    if (raf) cancelAnimationFrame(raf);
    raf = 0;
    ro?.disconnect(); io?.disconnect();
    document.removeEventListener('visibilitychange', onVis);
    overlay?.dispose();
    renderer?.dispose();
    renderer = null;
  }

  return {
    setProgress(p) {
      pTarget = clamp(p);
      if (still) still.setProgress(pTarget); else schedule();
    },
    lockOn(t) {
      if (still) { target = t; still.lockOn(t); return; }
      glide = { from: { ...look }, to: { lat: t.lat, lon: t.lon }, t0: performance.now() };
      target = t;
      overlay?.lock(performance.now(), LOCK_ON_MS * 0.8);
      schedule();
    },
    setLang(l) { lang = l; dirty = true; lastCam = ''; if (still) still.setLang(l); else schedule(); },
    resize() { if (still) still.resize(); else { size(); wake(); } },
    dispose() {
      if (disposed) return;
      teardownLive();
      disposed = true;
      still?.dispose();
      root.remove();
      delete host.dataset.globe;
    },
  };
}
