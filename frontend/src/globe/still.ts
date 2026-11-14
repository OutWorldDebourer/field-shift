// Still-frame fallback (reduced motion, no WebGL2, lost context, or opts.still): frames pre-rendered by
// this same renderer, shown cover-fit, with the measured overlay drawn through the camera that made them.
import { kmPerPx, makeCamera, project, type Cam } from './camera';
import { diveFrame, type DiveFrame } from './dive';
import { niceScale, Overlay } from './overlay';
import manifest from './plates.json';
import { creditText } from './renderer';
import type { GlobeCamera, GlobeOptions, GlobeTarget } from './types';

interface StillDef { id: string; file: string; w: number; h: number; p: number; lat: number; lon: number; credit: string }
const STILLS = manifest.stills as StillDef[];

export class StillView {
  private imgs = new Map<string, HTMLImageElement>();
  private overlay: Overlay | null;
  private p = 0;
  private lang: 'en' | 'es';
  private target: GlobeTarget;
  private dead = false;
  private ready = false;

  constructor(private root: HTMLElement, private base: string, private opts: GlobeOptions) {
    this.lang = opts.lang ?? 'en';
    this.target = opts.target;
    this.overlay = opts.overlay === false ? null : new Overlay(root);
    this.show();
    if (document.fonts) document.fonts.load('500 16px "IBM Plex Mono"').then(() => this.draw(), () => undefined);
  }

  private pick(): StillDef | undefined {
    const W = this.root.clientWidth || 1, H = this.root.clientHeight || 1;
    const want = this.p < 0.5 ? (W / H >= 1 ? 'orbit' : 'orbit-portrait') : 'field';
    return STILLS.find((s) => s.id === want) ?? STILLS[0];
  }

  private show() {
    const s = this.pick();
    if (!s) { this.fireReady(); return; }
    if (!this.imgs.has(s.id)) {
      const img = new Image();
      img.alt = ''; img.decoding = 'async'; img.draggable = false;
      Object.assign(img.style, { position: 'absolute', left: '0', top: '0', maxWidth: 'none', opacity: '0', transition: 'opacity .6s ease', transformOrigin: '0 0' });
      img.onload = () => { if (!this.dead) { this.draw(); this.fireReady(); } };
      img.onerror = () => this.fireReady();
      img.src = this.base + s.file;
      this.root.insertBefore(img, this.overlay ? this.overlay.canvas : null);
      this.imgs.set(s.id, img);
    }
    this.draw();
  }

  private fireReady() {
    if (this.ready || this.dead) return;
    this.ready = true;
    this.opts.onReady?.();
  }

  /** The camera that rendered still `s`, re-fitted to the viewport (cover). */
  private fit(s: StillDef, W: number, H: number): { cam: Cam; fr: DiveFrame; k: number; ox: number; oy: number } {
    const fr = diveFrame({ p: s.p, lat: s.lat, lon: s.lon, W: s.w, H: s.h, anchor: { x: 0.5, y: 0.5 }, time: 0 }, false);
    const c0 = fr.cam, k = Math.max(W / s.w, H / s.h);
    const t = project(c0, s.lat, s.lon);
    // centre the crop on the target, without showing past the image edge
    const ox = Math.min(0, Math.max(W - s.w * k, W / 2 - t.x * k));
    const oy = (H - s.h * k) / 2;
    const cam = makeCamera({ ...c0, sx: ox + k * (s.w / 2 + c0.sx) - W / 2, sy: oy + k * (s.h / 2 + c0.sy) - H / 2 }, W, H);
    cam.F = c0.F * k;
    return { cam, fr, k, ox, oy };
  }

  private draw() {
    if (this.dead) return;
    const s = this.pick();
    if (!s) return;
    const W = this.root.clientWidth || 1, H = this.root.clientHeight || 1;
    const { cam, fr, k, ox, oy } = this.fit(s, W, H);
    this.imgs.forEach((img, id) => {
      if (id !== s.id) { img.style.opacity = '0'; return; }
      img.style.width = s.w * k + 'px'; img.style.height = s.h * k + 'px';
      img.style.transform = `translate(${ox}px, ${oy}px)`;
      if (img.complete && img.naturalWidth) img.style.opacity = '1';
    });
    const sc = niceScale(kmPerPx(cam), W);
    const credit = creditText(s.credit, this.lang);
    const ts = project(cam, this.target.lat, this.target.lon);
    if (this.overlay) {
      this.overlay.resize(W, H, Math.min(devicePixelRatio || 1, 2));
      this.overlay.draw({
        cam, target: this.target, lang: this.lang, now: 0, ring: fr.ring, label: fr.label, settled: true,
        instruments: this.opts.instruments !== false, altitudeKm: cam.alt, scaleBarMeters: sc.meters, scaleBarPx: sc.px, credit,
      });
    }
    const c: GlobeCamera = {
      altitudeKm: cam.alt, scaleBarMeters: sc.meters, scaleBarPx: sc.px, imageryCredit: credit,
      targetScreen: ts.visible ? { x: ts.x, y: ts.y } : null, progress: s.p, sunElevationDeg: fr.sunEl, mode: 'still',
    };
    this.opts.onCamera?.(c);
  }

  setProgress(p: number) {
    const before = this.pick()?.id;
    this.p = p;
    if (this.pick()?.id !== before) this.show();
  }
  lockOn(t: GlobeTarget) { this.target = t; this.draw(); }
  setLang(l: 'en' | 'es') { this.lang = l; this.draw(); }
  resize() { this.show(); }
  dispose() { this.dead = true; this.overlay?.dispose(); this.imgs.forEach((i) => i.remove()); this.imgs.clear(); }
}
