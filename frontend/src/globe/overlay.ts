// The measured layer: a 2D canvas over the imagery. Reticle, field outline, leader line and ground label
// are anchored to the ground through the same camera as the shader; only the instruments (altitude,
// scale bar, imagery credit) live in screen space, in the corners.
//
// The hairline-with-dark-halo and bracket drawing follow the Satellite Briefing demo overlay
// (overlay.js: hair / brackets), © 2026 Lemomo/LemoLab, MIT.
import { project, type Cam } from './camera';
import { clamp, smooth } from './dive';
import type { GlobeTarget } from './types';

const INK = '#F4F7F8';
const HALO = 'rgba(3,6,10,.55)';
const MONO = '"IBM Plex Mono", ui-monospace, "Cascadia Mono", Consolas, monospace';

export interface OverlayFrame {
  cam: Cam;
  target: GlobeTarget;
  lang: 'en' | 'es';
  /** Milliseconds, monotonic. */
  now: number;
  /** 0..1 how much of the field outline has printed. */
  ring: number;
  /** 0..1 visibility of the leader line and ground label. */
  label: number;
  instruments: boolean;
  altitudeKm: number;
  scaleBarMeters: number;
  scaleBarPx: number;
  credit: string;
  /** Skip the time-based entrances (still frame, reduced motion). */
  settled: boolean;
}

export function formatAlt(km: number): string {
  if (km >= 100) return `ALT ${Math.round(km)} km`;
  if (km >= 10) return `ALT ${km.toFixed(0)} km`;
  if (km >= 1) return `ALT ${km.toFixed(1)} km`;
  return `ALT ${Math.round(km * 1000)} m`;
}
export function formatScale(m: number): string { return m >= 1000 ? `${m / 1000} km` : `${m} m`; }
export function formatCoords(lat: number, lon: number, lang: 'en' | 'es'): string {
  const f = (v: number) => { const s = Math.abs(v).toFixed(2); return lang === 'es' ? s.replace('.', ',') : s; };
  const ew = lon < 0 ? (lang === 'es' ? 'O' : 'W') : 'E';
  return `${f(lat)}°${lat < 0 ? 'S' : 'N'} ${f(lon)}°${ew}`;
}

const outBack = (x: number) => { const c1 = 1.70158, c3 = c1 + 1, t = clamp(x) - 1; return 1 + c3 * t * t * t + c1 * t * t; };
const outCubic = (x: number) => 1 - Math.pow(1 - clamp(x), 3);

export class Overlay {
  readonly canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private W = 1; private H = 1; private dpr = 1;
  /** Horizontal extent of the canvas that is actually inside the viewport (CSS px). */
  private safeL = 0; private safeR = 1;
  /** When the reticle starts snapping onto the target (ms); Infinity = still searching. */
  private lockAt = Infinity;

  constructor(parent: HTMLElement) {
    this.canvas = document.createElement('canvas');
    this.canvas.setAttribute('aria-hidden', 'true');
    Object.assign(this.canvas.style, { position: 'absolute', inset: '0', width: '100%', height: '100%', pointerEvents: 'none' });
    parent.appendChild(this.canvas);
    this.ctx = this.canvas.getContext('2d')!;
  }

  resize(W: number, H: number, dpr: number) {
    this.W = W; this.H = H; this.dpr = dpr;
    // the host can be wider than what the visitor sees (page overflow on a phone): keep text inside the visible part
    const r = this.canvas.getBoundingClientRect();
    const vw = Math.min(document.documentElement.clientWidth || W, window.visualViewport?.width ?? Infinity, window.innerWidth || Infinity);
    this.safeL = Math.max(0, -r.left);
    this.safeR = Math.max(this.safeL + 1, Math.min(W, vw - r.left));
    this.canvas.width = Math.max(1, Math.round(W * dpr));
    this.canvas.height = Math.max(1, Math.round(H * dpr));
  }

  /** Start (or restart) the lock-on: the reticle snaps onto the target `delay` ms from `now`. */
  lock(now: number, delay = 0) { this.lockAt = now + delay; }

  private hair(path: () => void, w: number, alpha: number, halo = 3) {
    const c = this.ctx;
    if (alpha <= 0.004) return;
    c.save();
    c.globalAlpha *= Math.min(1, alpha);
    c.lineJoin = 'miter'; c.lineCap = 'butt';
    if (halo) { c.beginPath(); path(); c.strokeStyle = HALO; c.lineWidth = w + halo; c.stroke(); }
    c.beginPath(); path(); c.strokeStyle = INK; c.lineWidth = w; c.stroke();
    c.restore();
  }

  private text(s: string, x: number, y: number, size: number, alpha: number, align: CanvasTextAlign = 'left', weight = 400) {
    const c = this.ctx;
    if (alpha <= 0.004 || !s) return;
    c.save();
    c.globalAlpha *= Math.min(1, alpha);
    c.font = `${weight} ${size}px ${MONO}`;
    c.textAlign = align; c.textBaseline = 'alphabetic';
    c.shadowColor = 'rgba(3,6,10,.9)'; c.shadowBlur = size * 0.5;
    c.fillStyle = INK; c.fillText(s, x, y);
    c.shadowBlur = size * 0.16; c.fillText(s, x, y);
    c.restore();
  }

  private measure(s: string, size: number, weight = 400) {
    const c = this.ctx;
    c.save(); c.font = `${weight} ${size}px ${MONO}`; const w = c.measureText(s).width; c.restore();
    return w;
  }

  draw(f: OverlayFrame) {
    const c = this.ctx, { W, H } = this;
    c.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);
    c.clearRect(0, 0, W, H);
    const cam = f.cam, tg = f.target;
    const ts = project(cam, tg.lat, tg.lon);
    const small = W < 620;
    const sL = this.safeL, sR = this.safeR;

    // ---- field outline: prints on along its perimeter
    let bw = 0, bh = 0, bcx = ts.x, bcy = ts.y;
    if (tg.ring && tg.ring.length > 2) {
      const pts = tg.ring.map(([lon, lat]) => project(cam, lat, lon));
      if (pts.every((p) => p.visible)) {
        const xs = pts.map((p) => p.x), ys = pts.map((p) => p.y);
        const x0 = Math.min(...xs), x1 = Math.max(...xs), y0 = Math.min(...ys), y1 = Math.max(...ys);
        bw = x1 - x0; bh = y1 - y0; bcx = (x0 + x1) / 2; bcy = (y0 + y1) / 2;
        if (f.ring > 0.004 && Math.max(bw, bh) > 6) {
          const seg = pts.map((p, i) => { const q = pts[(i + 1) % pts.length]; return Math.hypot(q.x - p.x, q.y - p.y); });
          const total = seg.reduce((a, b) => a + b, 0);
          this.hair(() => {
            let left = total * f.ring;
            c.moveTo(pts[0].x, pts[0].y);
            for (let i = 0; i < pts.length && left > 0; i++) {
              const q = pts[(i + 1) % pts.length], p = pts[i], t = Math.min(1, left / seg[i]);
              c.lineTo(p.x + (q.x - p.x) * t, p.y + (q.y - p.y) * t);
              left -= seg[i];
            }
            if (f.ring >= 0.999) c.closePath();
          }, 1.6, smooth(0, 0.08, f.ring), 3.2);
        }
      }
    }

    // ---- reticle: four brackets, snapping onto the target with a small overshoot
    const tLock = f.settled ? 1e6 : f.now - this.lockAt;
    const snap = tLock / 620;
    const locked = snap >= 1;
    if (ts.visible) {
      const base = clamp(W * 0.11, 84, 180);
      const fit = smooth(40, 140, Math.max(bw, bh));
      const cx = ts.x + (bcx - ts.x) * fit, cy = ts.y + (bcy - ts.y) * fit;
      const pad = small ? 22 : 30;
      let rw = Math.max(base, (bw + pad * 2) * fit), rh = Math.max(base * 0.79, (bh + pad * 2) * fit);
      let alpha = 1, ox = 0, oy = 0;
      if (snap < 0) {
        // searching: larger, dim, drifting a little off the target
        const w = f.now / 1000;
        rw *= 1.7; rh *= 1.7; alpha = 0.42;
        ox = Math.sin(w * 1.7) * 9 + 34; oy = Math.cos(w * 1.3) * 7 - 26;
      } else if (snap < 1.6) {
        const s = 1.7 + (1 - 1.7) * outBack(snap);
        const k = 1 - outCubic(snap * 1.4);
        rw *= s; rh *= s; alpha = 0.42 + 0.58 * outCubic(snap * 2);
        ox = 34 * k; oy = -26 * k;
      }
      const x0 = cx + ox - rw / 2, y0 = cy + oy - rh / 2, x1 = x0 + rw, y1 = y0 + rh;
      const L = Math.min(rw, rh) * 0.24;
      this.hair(() => {
        c.moveTo(x0, y0 + L); c.lineTo(x0, y0); c.lineTo(x0 + L, y0);
        c.moveTo(x1 - L, y0); c.lineTo(x1, y0); c.lineTo(x1, y0 + L);
        c.moveTo(x1, y1 - L); c.lineTo(x1, y1); c.lineTo(x1 - L, y1);
        c.moveTo(x0 + L, y1); c.lineTo(x0, y1); c.lineTo(x0, y1 - L);
      }, small ? 1.8 : 2.1, alpha, 2.6);

      // ---- leader line and ground label, drawn on once the reticle has locked
      const since = f.settled ? 1e6 : tLock - 620;
      const la = f.label * (locked ? 1 : 0);
      if (la > 0.004 && tg.label) {
        const name = (f.lang === 'es' && tg.labelEs) || tg.label;
        const coords = formatCoords(tg.lat, tg.lon, f.lang);
        const m = small ? 20 : 28;
        let s1 = small ? 13.5 : clamp(W * 0.0145, 17, 21), s2 = s1 * 0.86;
        let tw = Math.max(this.measure(name, s1), this.measure(coords, s2));
        const room = sR - sL - 2 * m - 30;
        if (tw > room) { const k = Math.max(0.7, room / tw); s1 *= k; s2 *= k; tw = Math.max(this.measure(name, s1), this.measure(coords, s2)); }
        let ex = cx + rw * 0.36, ey = cy - rh * 0.76;
        ex = Math.min(ex, sR - m - tw - 30);
        ex = Math.max(ex, sL + m);
        if (ey - s1 * 2.6 < m) ey = cy + rh * 0.76 + s1 * 2.6;       // no room above: hang the label below
        const shelf = tw + 24;
        const grow = outCubic(since / 420), shelfGrow = outCubic((since - 260) / 380);
        const px = cx - rw * 0.06, py = cy + rh * 0.02;
        this.hair(() => {
          c.moveTo(px, py);
          c.lineTo(px + (ex - px) * grow, py + (ey - py) * grow);
          if (shelfGrow > 0) c.lineTo(ex + shelf * shelfGrow, ey);
        }, 1.3, la, 2.6);
        c.save();
        c.globalAlpha = la * outCubic(since / 200);
        c.beginPath(); c.arc(px, py, 3.2, 0, Math.PI * 2); c.fillStyle = HALO; c.fill();
        c.beginPath(); c.arc(px, py, 2.4, 0, Math.PI * 2); c.fillStyle = INK; c.fill();
        c.restore();
        // the label prints on character by character
        const typed = (s: string, t0: number) => s.slice(0, Math.ceil(s.length * clamp((since - t0) / 380)));
        this.text(typed(name, 380), ex + 12, ey - s2 - 17, s1, la);
        this.text(typed(coords, 560), ex + 12, ey - 11, s2, la * 0.94);
      }
    }

    // ---- instruments: screen space, corners only
    if (f.instruments) {
      const m = clamp(W * 0.03, 16, 44), mb = clamp(H * 0.045, 18, 44);
      const sAlt = clamp(W * 0.0155, 14, 23), sCr = Math.max(11, sAlt * 0.78);
      const alt = formatAlt(f.altitudeKm);
      const stacked = this.measure(alt, sAlt, 500) + this.measure(f.credit, sCr) + 36 > sR - sL - 2 * m;
      const yBase = H - mb, yAlt = stacked ? yBase - sCr - 12 : yBase;
      this.text(alt, sL + m, yAlt, sAlt, 1, 'left', 500);
      this.text(f.credit, sR - m, yBase, sCr, 0.92, 'right');
      if (f.scaleBarPx > 8) {
        const y = yAlt - sAlt - 16, x = sL + m + 1, w = f.scaleBarPx;
        this.hair(() => {
          c.moveTo(x, y - 6); c.lineTo(x, y); c.lineTo(x + w, y); c.lineTo(x + w, y - 6);
        }, 1.5, 0.95, 2.6);
        this.text(formatScale(f.scaleBarMeters), x + w + 10, y + 1, sCr, 0.92);
      }
    }
  }

  dispose() { this.canvas.remove(); }
}

/** A round scale-bar length (1/2/5 x 10^n metres) that lands between ~70 and ~150 px. */
export function niceScale(kmPerPx: number, W: number): { meters: number; px: number } {
  const target = clamp(W * 0.085, 64, 130);
  const raw = kmPerPx * 1000 * target;
  const pow = Math.pow(10, Math.floor(Math.log10(raw)));
  const n = raw / pow;
  const meters = (n >= 5 ? 5 : n >= 2 ? 2 : 1) * pow;
  return { meters, px: meters / 1000 / kmPerPx };
}
