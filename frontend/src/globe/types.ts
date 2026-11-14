// Public contract of the globe module. The index and the app import only this and `createGlobe` from `./index`.

export interface GlobeTarget {
  lat: number;
  lon: number;
  /** Ground label shown with a leader line once the reticle locks on, e.g. "Huaral valley, Peru". */
  label?: string;
  /** Spanish ground label, used after `setLang('es')`; falls back to `label`. */
  labelEs?: string;
  /** Optional field outline as [lon, lat] vertices, drawn as a white hairline near the end of the dive. */
  ring?: [number, number][];
}

export interface GlobeCamera {
  /** Altitude above ground in km (drives the "ALT 420 km" instrument). */
  altitudeKm: number;
  /** Metres represented by `scaleBarPx` pixels on screen at the target (drives the scale bar). */
  scaleBarMeters: number;
  scaleBarPx: number;
  /** Credit for the imagery currently dominant on screen, e.g. "NASA Blue Marble". */
  imageryCredit: string;
  /** Screen position of the target in CSS px, or null when it is behind the planet. */
  targetScreen: { x: number; y: number } | null;
  /** Dive progress actually on screen (eased towards the last `setProgress` value). */
  progress?: number;
  /** Sun elevation over the target in degrees: about 7 before dawn at orbit, about 54 in daylight over the field. */
  sunElevationDeg?: number;
  /** 'live' = WebGL2 globe, 'still' = the pre-rendered frame fallback. */
  mode?: 'live' | 'still';
}

export interface GlobeOptions {
  target: GlobeTarget;
  /** 'en' | 'es' for any text the globe draws itself. */
  lang?: 'en' | 'es';
  /** Draw the reticle, leader line, label and corner instruments inside the globe's own overlay. Default true. */
  overlay?: boolean;
  /** With the overlay on, set false to leave the corner instruments (ALT, scale bar, credit) to the page. Default true. */
  instruments?: boolean;
  /** Force the still-frame fallback (reduced motion, low power or no WebGL2). */
  still?: boolean;
  /** Where the target rests at the end of the dive, as viewport fractions. Default { x: 0.5, y: 0.5 }. */
  anchor?: { x: number; y: number };
  /** Folder holding the textures and plates. Default `${import.meta.env.BASE_URL}globe/`. */
  assetBase?: string;
  /** Fixed device-pixel ratio; turns off the automatic cap (1.5 to 2) and the adaptive scaling. */
  pixelRatio?: number;
  /** Idle camera creep (a 2 % breath so the frame never freezes). Default true. */
  creep?: boolean;
  onCamera?: (c: GlobeCamera) => void;
  onReady?: () => void;
}

export interface GlobeHandle {
  /** Scroll-driven dive: 0 = orbit at sunrise, 1 = locked nadir over the field. */
  setProgress(p: number): void;
  /** Move the lock-on to a new target (animated). */
  lockOn(target: GlobeTarget): void;
  setLang(lang: 'en' | 'es'): void;
  resize(): void;
  dispose(): void;
}
