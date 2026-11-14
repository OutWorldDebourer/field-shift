# Globe

One continuous dive from an oblique orbit at sunrise to a locked nadir frame over the field, on real NASA
and Copernicus imagery. Dev page: `frontend/globe.html` (scroll drives the dive; query options are listed
in its script).

## Files

- `index.ts`: `createGlobe(host, opts)`, the render loop, progress easing, low-power handling, fallback switch.
- `types.ts`: the public contract (`GlobeOptions`, `GlobeHandle`, `GlobeCamera`, `GlobeTarget`).
- `camera.ts`: camera math in the look-at point's local frame, `project` (world to screen), `groundAt`.
- `dive.ts`: progress to camera, sun and atmosphere (log-space distance, easing, idle creep, mobile framing).
- `shader.ts`: the ray-sphere fragment shader (nested plates, terminator, night lights, clouds, limb, sun, stars).
- `renderer.ts`: Three.js full-screen pass, texture streaming, plate opacities and the dominant credit.
- `overlay.ts`: 2D canvas marks (reticle, field outline, leader and label, ALT, scale bar, credit).
- `still.ts`: still-frame fallback (reduced motion, no WebGL2, lost context, `opts.still`).
- `plates.json`: manifest of every texture and plate with its lon/lat box, credit and fade range.
- `screens/`: reference screenshots at 1440×900 and 390×844.

## API

```ts
import { createGlobe } from './globe';

const globe = createGlobe(host, {
  target: { lat: -11.4729, lon: -77.2061, label: 'Huaral valley, Peru', labelEs: 'Valle de Huaral, Perú', ring },
  lang: 'en',
  overlay: true,          // reticle, outline, leader and label
  instruments: true,      // ALT, scale bar, credit; set false when the page draws its own from onCamera
  onCamera: (c) => {},    // altitudeKm, scaleBarMeters/Px, imageryCredit, targetScreen, progress, mode
  onReady: () => {},
});
globe.setProgress(0.5);   // 0 = orbit at 420 km, 1 = nadir at 2.2 km with the outline printed
globe.lockOn(target);     // glide to another target
globe.setLang('es');
globe.resize();
globe.dispose();
```

Other options: `still`, `anchor` (where the target rests at the end), `assetBase`, `pixelRatio`, `creep`.
`host.dataset.globe` is `live` or `still`.

## Credits

Textures and plates live in `frontend/public/globe/`; every file's source URL, licence and modification is
in `frontend/public/globe/CREDITS.md`. Imagery: NASA Blue Marble and VIIRS (public domain, via NASA GIBS);
"Contains modified Copernicus Sentinel data 2026".

The camera math, the ray-sphere set-up with nested feathered plates, the limb shading and the hairline
drawing are adapted from the Satellite Briefing demo engine of Lemo-Opuscar (`engine.js`, `overlay.js`),
© 2026 Lemomo/LemoLab, MIT.
| `LICENSE-lemolab-engine.txt` | MIT licence of the Lemo-Opuscar Satellite Briefing engine adapted in camera.ts, shader.ts, overlay.ts |
