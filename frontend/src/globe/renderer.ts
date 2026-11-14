// Three.js side of the globe: one full-screen triangle with the ray-sphere shader, and the textures.
// The low-res world goes up first so the first frame is fast; plates stream in afterwards and fade on.
import {
  BufferAttribute, BufferGeometry, ClampToEdgeWrapping, DataTexture, GLSL3, LinearFilter, LinearMipmapLinearFilter,
  Mesh, NoColorSpace, OrthographicCamera, RawShaderMaterial, RepeatWrapping, RGBAFormat, Scene, Texture, Vector2,
  Vector3, Vector4, WebGLRenderer, type IUniform,
} from 'three';
import { groundAt } from './camera';
import { screenKm, smooth, type DiveFrame } from './dive';
import manifest from './plates.json';
import { FRAG, PLATE_SLOTS, VERT } from './shader';

export interface PlateDef { id: string; file: string; box: number[]; size: number; kind: string; credit: string; feather: number; fade: number[] }
export const PLATES = manifest.plates as PlateDef[];
export type CreditId = keyof typeof manifest.credits;
export function creditText(id: string, lang: 'en' | 'es'): string {
  const c = (manifest.credits as Record<string, { en: string; es: string }>)[id];
  return c ? c[lang] : '';
}

interface Slot { tex: Texture | null; born: number }
const FADE_MS = 450;
const D2R = Math.PI / 180;

export class GlobeRenderer {
  readonly canvas: HTMLCanvasElement;
  private gl: WebGLRenderer;
  private scene = new Scene();
  private camera = new OrthographicCamera(-1, 1, 1, -1, 0, 1);
  private geo = new BufferGeometry();
  private mat: RawShaderMaterial;
  private blank: DataTexture;
  private u: Record<string, IUniform>;
  private slots: Slot[] = PLATES.map(() => ({ tex: null, born: 0 }));
  private cloud: Slot = { tex: null, born: 0 };
  private nightR: Slot = { tex: null, born: 0 };
  private textures: Texture[] = [];
  private dead = false;
  private aniso = 1;
  private pr = 1;
  /** Called whenever a texture finished uploading (a new frame is worth drawing). */
  onTexture: () => void = () => {};
  onContextLost: () => void = () => {};

  constructor(parent: HTMLElement, private base: string) {
    this.canvas = document.createElement('canvas');
    Object.assign(this.canvas.style, { position: 'absolute', inset: '0', width: '100%', height: '100%', display: 'block' });
    this.canvas.setAttribute('role', 'img');
    // Throws when WebGL2 is not available: the caller falls back to the still frame.
    this.gl = new WebGLRenderer({ canvas: this.canvas, antialias: false, alpha: false, depth: false, stencil: false, powerPreference: 'default' });
    if (!this.gl.capabilities.isWebGL2) throw new Error('WebGL2 not available');
    this.gl.autoClear = false;
    this.aniso = Math.min(8, this.gl.capabilities.getMaxAnisotropy());
    parent.appendChild(this.canvas);
    this.canvas.addEventListener('webglcontextlost', this.lost);

    this.blank = new DataTexture(new Uint8Array([0, 0, 0, 255]), 1, 1, RGBAFormat);
    this.blank.needsUpdate = true;
    const tex = () => ({ value: this.blank as Texture });
    const v4 = () => ({ value: new Vector4(0, 0, 1, 1) });
    this.u = {
      uRes: { value: new Vector2(1, 1) }, uShift: { value: new Vector2() }, uF: { value: 1 },
      uR: { value: new Vector3() }, uU: { value: new Vector3() }, uFw: { value: new Vector3() }, uCl: { value: new Vector3() },
      uC2: { value: 1 }, uSinLat0: { value: 0 }, uCosLat0: { value: 1 }, uLat0: { value: 0 }, uLon0: { value: 0 },
      tWorld: tex(), tNight: tex(), tNightR: tex(), tCloud: tex(),
      bCloud: v4(), uCloud: { value: 0 }, uNightR: { value: 0 }, uCloudShift: { value: new Vector2() }, uClear: { value: new Vector3() },
      uSun: { value: new Vector3(0, 0, 1) }, uSunVis: { value: new Vector3(0, 1, 0) },
      uSunAmt: { value: 1 }, uExpo: { value: 1 }, uShade: { value: 1 }, uRelief: { value: 1 }, uWarm: { value: 1 }, uOcean: { value: 1 }, uTime: { value: 0 },
    };
    for (let i = 0; i < PLATE_SLOTS; i++) { this.u['tP' + i] = tex(); this.u['bP' + i] = v4(); this.u['gP' + i] = { value: new Vector4() }; }
    this.mat = new RawShaderMaterial({ glslVersion: GLSL3, vertexShader: VERT, fragmentShader: FRAG, uniforms: this.u, depthTest: false, depthWrite: false });
    this.geo.setAttribute('position', new BufferAttribute(new Float32Array([-1, -1, 0, 3, -1, 0, -1, 3, 0]), 3));
    const mesh = new Mesh(this.geo, this.mat);
    mesh.frustumCulled = false;
    this.scene.add(mesh);
  }

  private lost = (e: Event) => { e.preventDefault(); this.onContextLost(); };

  private async load(file: string, opts: { mip?: boolean; wrap?: boolean } = {}): Promise<Texture> {
    const res = await fetch(this.base + file);
    if (!res.ok) throw new Error(`${file}: ${res.status}`);
    const bmp = await createImageBitmap(await res.blob(), { imageOrientation: 'none', premultiplyAlpha: 'none', colorSpaceConversion: 'none' });
    const t = new Texture(bmp as unknown as HTMLImageElement);
    t.flipY = false; t.colorSpace = NoColorSpace; t.premultiplyAlpha = false;
    t.wrapS = opts.wrap ? RepeatWrapping : ClampToEdgeWrapping; t.wrapT = ClampToEdgeWrapping;
    t.generateMipmaps = opts.mip !== false;
    t.minFilter = opts.mip !== false ? LinearMipmapLinearFilter : LinearFilter; t.magFilter = LinearFilter;
    t.anisotropy = this.aniso;
    t.needsUpdate = true;
    if (this.dead) { bmp.close(); t.dispose(); throw new Error('disposed'); }
    this.gl.initTexture(t);       // upload now (off the draw call), then free the decoded bitmap
    bmp.close();
    this.textures.push(t);
    return t;
  }

  /** Stream the textures: low-res world, then what the orbit frame needs, then the dive plates. */
  async loadAll(onFirst: () => void): Promise<void> {
    const now = () => performance.now();
    const quiet = (p: Promise<unknown>) => p.catch(() => undefined);
    try {
      this.u.tWorld.value = await this.load(manifest.world.lo, { wrap: true });
    } catch { /* a blank world still renders the atmosphere and the marks */ }
    if (this.dead) return;
    onFirst();
    const plate = (i: number) => quiet(this.load(PLATES[i].file).then((t) => { this.slots[i] = { tex: t, born: now() }; this.u['tP' + i].value = t; this.onTexture(); }));
    const order = PLATES.map((_, i) => i);
    await Promise.all([
      plate(order[0]), plate(order[1]),
      quiet(this.load(manifest.regional.clouds).then((t) => { this.cloud = { tex: t, born: now() }; this.u.tCloud.value = t; this.onTexture(); })),
      quiet(this.load(manifest.regional.night).then((t) => { this.nightR = { tex: t, born: now() }; this.u.tNightR.value = t; this.onTexture(); })),
      quiet(this.load(manifest.night.file, { wrap: true }).then((t) => { this.u.tNight.value = t; this.onTexture(); })),
    ]);
    for (let i = 2; i < order.length && !this.dead; i++) await plate(order[i]);
    if (this.dead) return;
    await quiet(this.load(manifest.world.file, { wrap: true }).then((t) => {
      const old = this.u.tWorld.value as Texture;
      this.u.tWorld.value = t;
      if (old !== this.blank) { old.dispose(); this.textures = this.textures.filter((x) => x !== old); }
      this.onTexture();
    }));
  }

  setSize(W: number, H: number, pr: number) {
    this.pr = pr;
    this.gl.setPixelRatio(pr);
    this.gl.setSize(W, H, false);
  }

  /** Plate opacities for this camera and the credit of the imagery dominant at the target. */
  plateState(f: DiveFrame, target: { lat: number; lon: number }, now: number): { alpha: number[]; credit: string; loading: boolean } {
    const state = this.plateAlpha(f, target, now);
    // a plate that is opaque across the whole frame hides everything under it: do not sample those
    const cam = f.cam, pts: ({ lat: number; lon: number } | null)[] = [];
    for (const fx of [0, 0.5, 1]) for (const fy of [0, 0.5, 1]) pts.push(groundAt(cam, fx * cam.W, fy * cam.H));
    if (pts.every((p) => p)) {
      for (let i = PLATES.length - 1; i > 0; i--) {
        const [w, s, e, n] = PLATES[i].box, m = PLATES[i].feather * 1.02;
        const mx = (e - w) * m, my = (n - s) * m;
        if (state.alpha[i] > 0.999 && pts.every((p) => p!.lon > w + mx && p!.lon < e - mx && p!.lat > s + my && p!.lat < n - my)) {
          for (let j = 0; j < i; j++) state.alpha[j] = 0;
          break;
        }
      }
    }
    return state;
  }

  private plateAlpha(f: DiveFrame, target: { lat: number; lon: number }, now: number): { alpha: number[]; credit: string; loading: boolean } {
    const cam = f.cam, sk = screenKm(cam);
    let credit: string = manifest.world.credit, loading = false;
    const alpha = PLATES.map((p, i) => {
      const [w, s, e, n] = p.box;
      const km = (e - w) * 111.19 * Math.cos(((s + n) / 2) * D2R);
      const want = smooth(p.fade[0], p.fade[1], km / sk);
      const sl = this.slots[i];
      if (want > 0.01 && !sl.tex) loading = true;
      const a = sl.tex ? want * smooth(0, FADE_MS, now - sl.born) : 0;
      if (a < 1 && a > 0 && now - sl.born < FADE_MS) loading = true;
      if (a >= 0.5 && target.lon > w && target.lon < e && target.lat > s && target.lat < n) credit = p.credit;
      return a;
    });
    return { alpha, credit, loading };
  }

  render(f: DiveFrame, alpha: number[], target: { lat: number; lon: number }, time: number, now: number) {
    if (this.dead) return;
    const cam = f.cam, u = this.u, pr = this.pr;
    (u.uRes.value as Vector2).set(Math.round(cam.W * pr), Math.round(cam.H * pr));
    this.gl.getDrawingBufferSize(u.uRes.value as Vector2);
    (u.uShift.value as Vector2).set(cam.sx * pr, cam.sy * pr);
    u.uF.value = cam.F * pr;
    (u.uR.value as Vector3).fromArray(cam.r); (u.uU.value as Vector3).fromArray(cam.u); (u.uFw.value as Vector3).fromArray(cam.f);
    (u.uCl.value as Vector3).fromArray(cam.Cl);
    u.uC2.value = cam.c2; u.uSinLat0.value = cam.sinLat; u.uCosLat0.value = cam.cosLat; u.uLat0.value = cam.lat; u.uLon0.value = cam.lon;
    const rel = (b: number[], v: Vector4) => v.set(b[0] - cam.lon, b[1] - cam.lat, b[2] - cam.lon, b[3] - cam.lat);
    PLATES.forEach((p, i) => {
      rel(p.box, u['bP' + i].value as Vector4);
      const close = p.kind === 'close';
      (u['gP' + i].value as Vector4).set(alpha[i], p.feather, close ? f.closeGain : 1, close ? p.size : 0);
    });
    rel(manifest.regional.box, u.bCloud.value as Vector4);
    u.uCloud.value = this.cloud.tex ? f.cloud * smooth(0, FADE_MS, now - this.cloud.born) : 0;
    u.uNightR.value = this.nightR.tex ? smooth(0, FADE_MS, now - this.nightR.born) : 0;
    const az = f.sunAz * D2R, boxDeg = manifest.regional.box[2] - manifest.regional.box[0];
    const shadowKm = Math.min(26, Math.max(4, 3.2 / Math.tan(Math.max(4, f.sunEl) * D2R)));
    (u.uCloudShift.value as Vector2).set((Math.sin(az) * shadowKm) / 111.19 / boxDeg, (-Math.cos(az) * shadowKm) / 111.19 / boxDeg);
    (u.uClear.value as Vector3).set(target.lat - cam.lat, target.lon - cam.lon, 2.6);
    (u.uSun.value as Vector3).fromArray(f.sun); (u.uSunVis.value as Vector3).fromArray(f.sunVis);
    u.uSunAmt.value = f.sunAmt; u.uExpo.value = f.expo; u.uShade.value = f.shade; u.uRelief.value = f.relief; u.uWarm.value = f.warm; u.uOcean.value = f.relief;
    u.uTime.value = time % 1000;
    this.gl.render(this.scene, this.camera);
  }

  dispose() {
    this.dead = true;
    this.canvas.removeEventListener('webglcontextlost', this.lost);
    this.textures.forEach((t) => t.dispose());
    this.blank.dispose(); this.geo.dispose(); this.mat.dispose();
    this.gl.dispose();
    this.gl.forceContextLoss();
    this.canvas.remove();
  }
}
