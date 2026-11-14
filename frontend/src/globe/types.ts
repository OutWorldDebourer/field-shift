// Public contract of the globe module. The index and the app import only this and `createGlobe` from `./index`.

export interface GlobeTarget {
  lat: number;
  lon: number;
  /** Ground label shown with a leader line once the reticle locks on, e.g. "Huaral valley, Peru". */
  label?: string;
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
}

export interface GlobeOptions {
  target: GlobeTarget;
  /** 'en' | 'es' for any text the globe draws itself. */
  lang?: 'en' | 'es';
  /** Draw the reticle, leader line, label and corner instruments inside the globe's own overlay. Default true. */
  overlay?: boolean;
  /** Force the still-frame fallback (reduced motion, low power or no WebGL2). */
  still?: boolean;
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
