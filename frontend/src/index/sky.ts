// The fixed sky behind the index: the Three.js globe (src/globe) when it renders, otherwise the illustrative
// orbit still with a scroll push-in. Drives the corner instruments and the reticle/ground labels in both cases.
import type { GlobeCamera, GlobeHandle } from '../globe';
import { HUARAL_FIELD } from '../shared/demo/huaral';
import { $ } from '../shared/dom';
import { lang, onLang, t } from '../shared/i18n';
import { reducedMotion } from '../shared/dom';

// Reticle centre and size inside public/img/orbit-still.jpg (1672×768).
const STILL = { w: 1672, h: 768, cx: 828, cy: 438, rw: 184, rh: 144 };

export interface Sky { update(globeP: number, diveP: number, heroVis?: number): void; isLive(): boolean; camera(): GlobeCamera | null; onCamera(fn: () => void): void }

export function mountSky(): Sky {
  const host = $('#globe-host');
  const still = $('#still');
  const stillImg = still.querySelector('img')!;
  const reticle = $('#hero-reticle');
  const leaders = $('#hero-leaders') as unknown as SVGSVGElement;
  const lblValley = $('#lbl-valley'), lblField = $('#lbl-field'), lblHow = $('#lbl-how');
  const alt = $('#hud-alt'), credit = $('#hud-credit'), scale = $('#hud-scale'), scaleT = $('#hud-scale-t');
  let live = false;
  const camFns: (() => void)[] = [];
  let cam: GlobeCamera | null = null;
  let lastP = 0, lastDive = 0;

  let globe: GlobeHandle | null = null;
  // three.js is code-split: the still paints first, the globe module loads after.
  void import('../globe').then(({ createGlobe }) => { globe = createGlobe(host, {
    target: { lat: HUARAL_FIELD.center.lat, lon: HUARAL_FIELD.center.lon, label: 'Huaral valley, Peru', labelEs: 'Valle de Huaral, Perú', ring: HUARAL_FIELD.ring },
    lang: lang(),
    overlay: true,
    instruments: false,
    anchor: innerWidth >= 900 ? { x: 0.5, y: 0.56 } : { x: 0.5, y: 0.5 },
    still: reducedMotion(),
    onCamera: (c) => { cam = c; if (live) { drawInstruments(); camFns.forEach((f) => f()); guardOverlay(); } },
    onReady: () => setTimeout(detect, 60),
  }); globe.setProgress(lastP); });
  onLang((l) => { globe?.setLang(l); drawInstruments(); });
  addEventListener('resize', () => { globe?.resize(); update(lastP, lastDive); });

  function detect() {
    live = host.dataset.globe !== 'placeholder' && host.childElementCount > 0;
    document.body.classList.toggle('has-globe', live);
    update(lastP, lastDive);
  }
  setTimeout(detect, 1500);

  /** Hide the globe's own ground label/reticle while it would sit under page text or a button. */
  function guardOverlay() {
    const cv = host.querySelectorAll('canvas');
    const ov = cv[cv.length - 1] as HTMLCanvasElement | undefined;
    const ts = cam?.targetScreen;
    if (!ov || cv.length < 2) return;
    let hide = false;
    if (ts) {
      for (const el of document.querySelectorAll<HTMLElement>('.hero__text, .ctas, .sec-title, .sec-lede, .ds, .step, .get__strip, .cta__title, .foot, .phone')) {
        const r = el.getBoundingClientRect();
        if (r.bottom < 0 || r.top > innerHeight || !r.width) continue;
        // the label runs ~260 px right and ~90 px up from the target
        if (ts.x + 270 > r.left && ts.x + 20 < r.right && ts.y + 70 > r.top && ts.y - 110 < r.bottom) { hide = true; break; }
      }
    }
    ov.style.transition = 'opacity 0.3s';
    ov.style.opacity = hide ? '0' : '1';
  }

  function drawInstruments() {
    if (live && cam) {
      alt.textContent = t('alt', { km: cam.altitudeKm >= 10 ? Math.round(cam.altitudeKm) : cam.altitudeKm.toFixed(1) });
      credit.textContent = cam.imageryCredit;
      scale.hidden = !(cam.scaleBarPx > 8);
      if (!scale.hidden) {
        (scale.querySelector('i') as HTMLElement).style.width = cam.scaleBarPx + 'px';
        scaleT.textContent = cam.scaleBarMeters >= 1000 ? `${Math.round(cam.scaleBarMeters / 1000)} km` : `${Math.round(cam.scaleBarMeters)} m`;
      }
    } else {
      // The still is illustrative: no measured scale bar, honest credit, altitude only as the story's readout.
      const km = lastDive < 0.5 ? 420 * Math.pow(40 / 420, lastDive / 0.5) : 40 * Math.pow(2 / 40, (lastDive - 0.5) / 0.5);
      alt.textContent = t('alt', { km: Math.round(km) });
      credit.textContent = t('still_credit');
      scale.hidden = true;
    }
  }

  /** Screen position of the still's reticle point after cover-fit and push-in. */
  function stillPoint(zoom: number) {
    const W = innerWidth, H = innerHeight;
    const s = Math.max(W / STILL.w, H / STILL.h);
    const ox = (W - STILL.w * s) / 2, oy = (H - STILL.h * s) / 2;
    return { x: ox + STILL.cx * s, y: oy + STILL.cy * s, rw: STILL.rw * s * zoom, rh: STILL.rh * s * zoom };
  }

  let lastHero = 1;
  function update(globeP: number, diveP: number, heroVis = lastHero) {
    lastP = globeP; lastDive = diveP; lastHero = heroVis;
    if (live) globe?.setProgress(globeP);
    const zoom = 1 + diveP * 0.8;
    let pt: { x: number; y: number; rw: number; rh: number } | null;
    if (live) {
      still.style.opacity = '0';
      const ts = cam?.targetScreen ?? null;
      pt = ts ? { x: ts.x, y: ts.y, rw: 0, rh: 0 } : null;
      reticle.hidden = true; // the globe draws its own reticle
    } else {
      const p = stillPoint(1);
      still.style.opacity = '1';
      stillImg.style.transformOrigin = `${(STILL.cx / STILL.w) * 100}% ${(STILL.cy / STILL.h) * 100}%`;
      stillImg.style.transform = reducedMotion() ? 'none' : `scale(${zoom.toFixed(4)})`;
      const z = reducedMotion() ? 1 : zoom;
      pt = { x: p.x, y: p.y, rw: p.rw * z, rh: p.rh * z };
      reticle.hidden = false;
      Object.assign(reticle.style, { left: pt.x - pt.rw / 2 + 'px', top: pt.y - pt.rh / 2 + 'px', width: pt.rw + 'px', height: pt.rh + 'px' });
    }
    // Ground labels: hero labels fade out as the dive starts; the valley label belongs to "How it works".
    const heroA = Math.max(0, Math.min(heroVis, 1 - diveP * 5));
    const howA = Math.max(0, Math.min(1, (diveP - 0.3) * 5)) * Math.max(0, Math.min(1, (1 - diveP) * 4));
    const wide = innerWidth >= 1100;
    let d = '';
    if (pt && wide) {
      const vx = pt.x + 40, vy = pt.y - Math.max(86, pt.rh * 0.62);
      place(lblValley, vx + 6, vy - 52, heroA * (live ? 0 : 1));
      place(lblField, pt.x + 70, pt.y + 82, heroA);
      if (heroA > 0) {
        if (!live) d += `M${pt.x - 8} ${pt.y + 4}L${vx} ${vy}L${vx + 236} ${vy}`;
        d += `M${pt.x + 26} ${pt.y + 40}L${pt.x + 64} ${pt.y + 92}`;
      }
      place(lblHow, pt.x + Math.max(70, pt.rw * 0.55), pt.y - 12, live ? 0 : howA);
    } else {
      [lblValley, lblField].forEach((l) => place(l, 0, 0, 0));
      if (pt && !live) place(lblHow, Math.max(16, pt.x - 120), pt.y + 70, howA); else place(lblHow, 0, 0, 0);
    }
    leaders.innerHTML = d ? `<path d="${d}" style="opacity:${heroA}"/>` : '';
    drawInstruments();
  }
  function place(n: HTMLElement, x: number, y: number, a: number) {
    n.style.transform = `translate(${x}px, ${y}px)`;
    n.style.opacity = String(a);
    n.style.visibility = a > 0.01 ? 'visible' : 'hidden';
  }

  return { update, isLive: () => live, camera: () => cam, onCamera: (fn) => { camFns.push(fn); } };
}
