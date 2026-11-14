// The dive: scroll progress (0..1) -> camera, light and atmosphere. Distance interpolates in log space,
// every segment eases, and a slow creep keeps the frame alive when the scroll rests.
import { distanceForAltitude, limbDir, makeCamera, projectDir, R, type Cam, type CamParams, type V3 } from './camera';

const D2R = Math.PI / 180;
export const clamp = (x: number, a = 0, b = 1) => Math.min(b, Math.max(a, x));
export const mix = (a: number, b: number, t: number) => a + (b - a) * t;
export const smooth = (a: number, b: number, x: number) => { const t = clamp((x - a) / (b - a)); return t * t * (3 - 2 * t); };
const smoother = (x: number) => { const t = clamp(x); return t * t * t * (t * (t * 6 - 15) + 10); };

export const ORBIT_ALT_KM = 420;
export const END_DIST_KM = 2.2;
const ORBIT_TILT = 54;
const ORBIT_HEADING = 9;
const SUN_EL0 = 15;

export interface DiveFrame {
  cam: Cam;
  /** Ground light direction (local frame) and where the sun is drawn. */
  sun: V3; sunVis: V3;
  /** Sun azimuth (deg from north) and elevation at the target (deg). */
  sunAz: number; sunEl: number;
  sunAmt: number; expo: number; shade: number; relief: number; warm: number; cloud: number;
  /** Gain applied to the close-range (Sentinel-2) plates. */
  closeGain: number;
  /** 0..1 how far the field outline has printed. */
  ring: number;
  /** 0..1 visibility of the ground label. */
  label: number;
}

export interface DiveInput {
  p: number; lat: number; lon: number; W: number; H: number;
  /** Where the target rests at the end of the dive, as viewport fractions. */
  anchor: { x: number; y: number };
  /** Seconds, for the idle creep (pass a constant to freeze it). */
  time: number;
}

/** Framing that depends on the viewport shape: a portrait phone needs a wider lens to keep the coast in frame. */
function framing(W: number, H: number) {
  const portrait = 1 - smooth(0.75, 1.45, W / H);
  return {
    portrait,
    fov0: mix(45, 60, portrait), fov1: mix(45, 52, portrait),
    ax: mix(0.495, 0.5, portrait), ay: mix(0.55, 0.585, portrait),
    sunX: mix(0.14, 0.2, portrait),
    heading0: mix(ORBIT_HEADING, 14, portrait),
  };
}

function params(inp: DiveInput, g: number, creep: boolean): CamParams {
  const fr = framing(inp.W, inp.H);
  // The creep: at orbit a slow rotation at constant altitude (the instrument holds 420 km); lower down a
  // 2 % breath in distance. The frame never freezes.
  const w = (period: number) => (creep ? Math.sin((inp.time * 2 * Math.PI) / period) : 0);
  const tilt0 = ORBIT_TILT + 0.45 * w(23);
  const d0 = distanceForAltitude(ORBIT_ALT_KM, tilt0);
  const d = Math.exp(mix(Math.log(d0), Math.log(END_DIST_KM), g)) * (1 + 0.022 * w(19) * smooth(0.02, 0.3, g));
  const k = smooth(0, 0.82, g);
  const tilt = tilt0 * Math.pow(1 - k, 1.15);
  const heading = fr.heading0 * (1 - smooth(0, 0.7, g)) + 0.9 * w(31) * (0.35 + 0.65 * (1 - g));
  const a = smooth(0.1, 0.9, g);
  return {
    lat: inp.lat, lon: inp.lon, d, tilt, heading, fov: mix(fr.fov0, fr.fov1, g),
    sx: (mix(fr.ax, inp.anchor.x, a) - 0.5) * inp.W, sy: (mix(fr.ay, inp.anchor.y, a) - 0.5) * inp.H,
  };
}

// The sun sits on the limb, top-left of the orbit frame. Its azimuth comes from that composition and
// then stays fixed on the ground; only its elevation rises with the dive.
const sunCache = { key: '', az: -40, elVis: -15 };
function sunFor(inp: DiveInput) {
  const key = `${inp.W}x${inp.H}`;
  if (sunCache.key === key) return sunCache;
  const fr = framing(inp.W, inp.H);
  const cam0 = makeCamera(params({ ...inp, p: 0 }, 0, false), inp.W, inp.H);
  let best: V3 = [0, 1, 0], err = Infinity;
  for (let i = 0; i < 1440; i++) {
    const dir = limbDir(cam0, (i / 1440) * 2 * Math.PI, 0.35);
    const s = projectDir(cam0, dir);
    if (s.z <= 0.05 || s.y > inp.H * 0.6) continue;
    const e = Math.abs(s.x - fr.sunX * inp.W);
    if (e < err) { err = e; best = dir; }
  }
  sunCache.key = key;
  sunCache.az = Math.atan2(best[0], best[1]) / D2R;
  sunCache.elVis = Math.asin(best[2]) / D2R;
  return sunCache;
}

const dirOf = (az: number, el: number): V3 => {
  const a = az * D2R, e = el * D2R;
  return [Math.sin(a) * Math.cos(e), Math.cos(a) * Math.cos(e), Math.sin(e)];
};

export function diveFrame(inp: DiveInput, creep = true): DiveFrame {
  const p = clamp(inp.p);
  // the dive proper ends at p = 0.9; the last tenth is the lock: a slow push-in while the outline prints
  const x = clamp(p / 0.9);
  const g = 0.955 * (0.55 * smoother(x) + 0.45 * x) + 0.045 * smooth(0.9, 1, p);
  const cam = makeCamera(params(inp, g, creep), inp.W, inp.H);
  const s = sunFor(inp);
  const rise = smooth(0.02, 0.8, p);
  const sunEl = mix(SUN_EL0, 54, rise);
  const altK = cam.alt;
  return {
    cam,
    sun: dirOf(s.az, sunEl), sunVis: dirOf(s.az, s.elVis + (sunEl - SUN_EL0) * 0.8),
    sunAz: s.az, sunEl,
    sunAmt: 1 - smooth(0.25, 0.6, p),
    expo: mix(1.85, 1.22, rise),
    shade: smooth(Math.log(25), Math.log(300), Math.log(altK)),
    relief: smooth(Math.log(40), Math.log(260), Math.log(altK)),
    warm: 1 - smooth(0.05, 0.7, p),
    cloud: smooth(Math.log(120), Math.log(330), Math.log(altK)),
    closeGain: mix(0.86, 1.0, smooth(0.45, 0.9, p)),
    ring: smooth(0.86, 0.97, p),
    label: Math.max(1 - smooth(0.06, 0.16, p), smooth(0.9, 0.97, p)),
  };
}

/** Ground width across the screen at the look-at point, km. */
export function screenKm(cam: Cam): number { return (cam.W * cam.d) / cam.F; }
export const EARTH_R = R;
