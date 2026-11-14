// Camera math for the globe: one perspective camera around a spherical Earth and the same camera in JS
// for the 2D annotation layer, so every mark goes through one world -> screen function.
//
// Adapted from the Satellite Briefing demo engine (engine.js: makeCamera / project / kmPerPx),
// © 2026 Lemomo/LemoLab, MIT. Changes: TypeScript, and everything is expressed in the local
// east/north/up frame of the look-at point so the shader keeps metre precision at 2 km altitude.

export const R = 6371.0088; // Earth radius, km
const D2R = Math.PI / 180;

export type V3 = [number, number, number];
const dot = (a: V3, b: V3) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
const cross = (a: V3, b: V3): V3 => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
const norm = (a: V3): V3 => { const l = Math.hypot(a[0], a[1], a[2]) || 1; return [a[0] / l, a[1] / l, a[2] / l]; };

export interface CamParams {
  /** Look-at ground point. */
  lat: number; lon: number;
  /** Distance camera -> look-at point, km. */
  d: number;
  /** Degrees from straight down. */
  tilt: number;
  /** Direction the camera faces, degrees clockwise from north. */
  heading: number;
  /** Vertical field of view, degrees. */
  fov: number;
  /** Lens shift: where the look-at point sits, in CSS px from the frame centre. */
  sx: number; sy: number;
}

export interface Cam extends CamParams {
  W: number; H: number;
  /** Focal length in CSS px. */
  F: number;
  /** Camera position and basis in the local ENU frame of the look-at point (km / unit vectors). */
  Cl: V3; f: V3; u: V3; r: V3;
  /** |C|^2 - R^2, computed in double precision (float32 fails below a few tens of km). */
  c2: number;
  /** Altitude above the sphere, km. */
  alt: number;
  sinLat: number; cosLat: number;
}

export function makeCamera(p: CamParams, W: number, H: number): Cam {
  const hd = p.heading * D2R, tl = p.tilt * D2R;
  const st = Math.sin(tl), ct = Math.cos(tl), sh = Math.sin(hd), ch = Math.cos(hd);
  const Cl: V3 = [-p.d * st * sh, -p.d * st * ch, p.d * ct];
  const f = norm([-Cl[0], -Cl[1], -Cl[2]]);
  let u: V3 = [sh * ct, ch * ct, st];
  const k = dot(u, f);
  u = norm([u[0] - f[0] * k, u[1] - f[1] * k, u[2] - f[2] * k]);
  const r = norm(cross(f, u));
  const c2 = p.d * p.d + 2 * R * p.d * ct;
  return {
    ...p, W, H, F: (H / 2) / Math.tan((p.fov * D2R) / 2), Cl, f, u, r, c2,
    alt: Math.sqrt(R * R + c2) - R, sinLat: Math.sin(p.lat * D2R), cosLat: Math.cos(p.lat * D2R),
  };
}

/** Ground point (lat, lon) in the camera's local ENU frame, km, relative to the look-at point. */
export function toLocal(cam: Cam, lat: number, lon: number, h = 0): V3 {
  const p = lat * D2R, dl = (lon - cam.lon) * D2R, rr = R + h;
  const cp = Math.cos(p), sp = Math.sin(p);
  const A = rr * cp * Math.cos(dl), e = rr * cp * Math.sin(dl), B = rr * sp;
  // rotate the meridian plane by the reference latitude: (A, B) -> (north, up)
  return [e, B * cam.cosLat - A * cam.sinLat, A * cam.cosLat + B * cam.sinLat - R];
}

/** World -> screen in CSS px. `visible` = in front of the camera and on its side of the globe. */
export function project(cam: Cam, lat: number, lon: number): { x: number; y: number; visible: boolean } {
  const P = toLocal(cam, lat, lon);
  const q: V3 = [P[0] - cam.Cl[0], P[1] - cam.Cl[1], P[2] - cam.Cl[2]];
  const z = dot(q, cam.f);
  const n: V3 = [P[0], P[1], P[2] + R];
  const facing = dot(n, q) < 0;
  return {
    x: cam.W / 2 + cam.sx + (cam.F * dot(q, cam.r)) / z,
    y: cam.H / 2 + cam.sy - (cam.F * dot(q, cam.u)) / z,
    visible: z > 0 && facing,
  };
}

/** Screen px -> unit view direction in the local frame. */
export function rayDir(cam: Cam, x: number, y: number): V3 {
  const nx = (x - cam.W / 2 - cam.sx) / cam.F, ny = -(y - cam.H / 2 - cam.sy) / cam.F;
  return norm([cam.f[0] + cam.r[0] * nx + cam.u[0] * ny, cam.f[1] + cam.r[1] * nx + cam.u[1] * ny, cam.f[2] + cam.r[2] * nx + cam.u[2] * ny]);
}

/** Screen px -> ground (lat, lon), or null when the ray misses the Earth. */
export function groundAt(cam: Cam, x: number, y: number): { lat: number; lon: number } | null {
  const d = rayDir(cam, x, y);
  const Cc: V3 = [-cam.Cl[0], -cam.Cl[1], -(cam.Cl[2] + R)];
  const b = dot(d, Cc), disc = b * b - cam.c2;
  if (disc < 0 || b < 0) return null;
  const t = cam.c2 / (b + Math.sqrt(disc));
  const e = cam.Cl[0] + t * d[0], n = cam.Cl[1] + t * d[1], z = R + cam.Cl[2] + t * d[2];
  const A = z * cam.cosLat - n * cam.sinLat, B = n * cam.cosLat + z * cam.sinLat;
  return { lat: Math.atan2(B, Math.hypot(A, e)) / D2R, lon: cam.lon + Math.atan2(e, A) / D2R };
}

/** Local direction -> screen px (for things at infinity, e.g. the sun). */
export function projectDir(cam: Cam, d: V3): { x: number; y: number; z: number } {
  const z = dot(d, cam.f);
  return { x: cam.W / 2 + cam.sx + (cam.F * dot(d, cam.r)) / z, y: cam.H / 2 + cam.sy - (cam.F * dot(d, cam.u)) / z, z };
}

/** Kilometres per CSS px at the look-at point, across the view direction (drives the scale bar). */
export function kmPerPx(cam: Cam): number { return cam.d / cam.F; }

/** Distance that puts the camera at altitude `alt` for a given tilt at the look-at point. */
export function distanceForAltitude(alt: number, tiltDeg: number): number {
  const c = R * Math.cos(tiltDeg * D2R);
  return -c + Math.sqrt(c * c + alt * (2 * R + alt));
}

/** Direction (local frame) of a point on the Earth's limb, `lift` degrees above it, at angle `a` around nadir. */
export function limbDir(cam: Cam, a: number, liftDeg: number): V3 {
  const C: V3 = [cam.Cl[0], cam.Cl[1], cam.Cl[2] + R];
  const down = norm([-C[0], -C[1], -C[2]]);
  const e1 = norm(cross(down, [0, 1, 0]));
  const e2 = cross(down, e1);
  const th = Math.asin(R / Math.hypot(C[0], C[1], C[2])) + liftDeg * D2R;
  const s = Math.sin(th), c = Math.cos(th), ca = Math.cos(a), sa = Math.sin(a);
  return norm([down[0] * c + (e1[0] * ca + e2[0] * sa) * s, down[1] * c + (e1[1] * ca + e2[1] * sa) * s, down[2] * c + (e1[2] * ca + e2[2] * sa) * s]);
}
