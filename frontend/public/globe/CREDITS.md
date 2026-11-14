# Globe imagery: sources and licences

Every pixel of ground in the globe is real satellite imagery. Nothing here is AI-generated, and there are
no Google, Bing, Esri, Mapbox, Planet or Maxar basemaps. All files were downloaded on 2026-11-14.
The lon/lat box of each plate is in `frontend/src/globe/plates.json`.

On-screen credits (bottom right, switching with the plate dominant on screen):
`NASA Blue Marble` and `Contains modified Copernicus Sentinel data 2026`.

## NASA (public domain; credit NASA / GIBS)

"We acknowledge the use of imagery provided by services from NASA's Global Imagery Browse Services (GIBS),
part of NASA's Earth Science Data and Information System (ESDIS)."

All fetched from the GIBS WMS endpoint
`https://gibs.earthdata.nasa.gov/wms/epsg4326/best/wms.cgi?SERVICE=WMS&REQUEST=GetMap&VERSION=1.1.1&SRS=EPSG:4326&STYLES=`
followed by the parameters listed per file.

| File | Layer | Request parameters | Changes |
|---|---|---|---|
| `world-bmng.jpg` (4096×2048) | Blue Marble: Next Generation, shaded relief (`BlueMarble_ShadedRelief`) | `&LAYERS=BlueMarble_ShadedRelief&BBOX=-180,-90,180,90&WIDTH=4096&HEIGHT=2048&FORMAT=image/jpeg` | none (file as served) |
| `world-bmng-lo.jpg` (1024×512) | same | same | downscaled, for the first frame |
| `andes-bmng.jpg` (2560×2560) | same | `&LAYERS=BlueMarble_ShadedRelief&BBOX=-93,-24,-61,8&WIDTH=2560&HEIGHT=2560&FORMAT=image/png` | Lima cloud residual replaced (see below), JPEG re-encode |
| `coast-bmng.jpg` (2048×2048) | same | `&LAYERS=BlueMarble_ShadedRelief&BBOX=-81.2061,-15.4729,-73.2061,-7.4729&WIDTH=2048&HEIGHT=2048&FORMAT=image/png` | Lima cloud residual replaced (see below), JPEG re-encode |
| `world-night.jpg` (2048×1024) | VIIRS Black Marble night lights (`VIIRS_Black_Marble`) | `&LAYERS=VIIRS_Black_Marble&BBOX=-180,-90,180,90&WIDTH=4096&HEIGHT=2048&FORMAT=image/jpeg` | downscaled |
| `andes-night.jpg` (1536×1536) | same | `&LAYERS=VIIRS_Black_Marble&BBOX=-93,-24,-61,8&WIDTH=2048&HEIGHT=2048&FORMAT=image/png` | downscaled |
| `andes-clouds-viirs-20261006.jpg` (1024×1024, greyscale) | VIIRS / Suomi NPP corrected reflectance, true colour, 2026-10-06 (`VIIRS_SNPP_CorrectedReflectance_TrueColor`) | `&LAYERS=VIIRS_SNPP_CorrectedReflectance_TrueColor&TIME=2026-10-06&BBOX=-93,-24,-61,8&WIDTH=2048&HEIGHT=2048&FORMAT=image/png` | cloud opacity derived from the real image (bright, unsaturated pixels), blurred and downscaled so it draws as a soft veil |

Blue Marble carries a baked-in cloud residual over Lima (a white patch on the coast, about 60 km south-east
of the target). In `andes-bmng.jpg` and `coast-bmng.jpg` only those white pixels, inside the box
`-77.45,-12.55,-76.65,-11.65`, are replaced with the NASA/USGS Landsat WELD annual true-colour composite
(public domain; GIBS layer `Landsat_WELD_CorrectedReflectance_TrueColor_Global_Annual`, same WMS endpoint,
`&BBOX=<that box snapped to the plate grid>&FORMAT=image/png`), gain-matched to the surrounding Blue Marble
land. Andean snow and everything else is untouched. `world-bmng.jpg` is not patched.

Licence: NASA imagery is not copyrighted and is free to reuse with credit
(https://www.earthdata.nasa.gov/engage/open-data-services-software-policies/data-use-guidance).

## Copernicus Sentinel-2 (free, full and open; attribution required)

Credit: **Contains modified Copernicus Sentinel data 2026.**
Licence: Copernicus Sentinel data legal notice,
https://sentinels.copernicus.eu/documents/247904/690755/Sentinel_Data_Legal_Notice

Sentinel-2 L2A, true-colour (`visual`) asset, all acquired on 2026-10-06, read through the Microsoft
Planetary Computer (no account): STAC search at `https://planetarycomputer.microsoft.com/api/stac/v1/search`
(collection `sentinel-2-l2a`), pixels from the data API

`https://planetarycomputer.microsoft.com/api/data/v1/item/bbox/{w},{s},{e},{n}/{W}x{H}.png?collection=sentinel-2-l2a&item={ITEM}&assets=visual&resampling=bilinear`

| File | Box (w, s, e, n) and size | Items |
|---|---|---|
| `region-s2-20261006.jpg` | `-78.1061,-12.3729,-76.3061,-10.5729`, 2048×2048 | relative orbit 25 (Sentinel-2A, 15:29 UTC): `S2A_MSIL2A_20261006T152931_R025_T17LRG_20261006T231403`, `…_T17LRH_…`, `…_T17LRJ_…`, `…_T18LTM_…`, `…_T18LTN_…`, `…_T18LTP_…`, `…_T18LUN_…`, `…_T18LUP_…`; relative orbit 125 (Sentinel-2B, 15:17 UTC): `S2B_MSIL2A_20261006T151719_R125_T17LRG_20261006T202909`, `…_T18LTM_…`, `…_T18LTN_…`, `…_T18LTP_…`, `…_T18LUM_…`, `…_T18LUN_…`, `…_T18LUP_…` |
| `valley-s2-20261006.jpg` | `-77.4061,-11.6729,-77.0061,-11.2729`, 2048×2048 | `S2A_MSIL2A_20261006T152931_R025_T18LTN_20261006T231403`, `S2A_MSIL2A_20261006T152931_R025_T17LRH_20261006T231403` |
| `field-s2-20261006.jpg` | `-77.2561,-11.5229,-77.1561,-11.4229`, 1152×1152 (native 10 m) | same two items |

The same requests with `&assets=B08&rescale=0,4000` (near-infrared) gave the water mask used below; the
B08 images are not shipped.

Modifications (the credit above still applies):

- `region`: the two same-day passes are mosaicked, feathered across their overlap, and the eastern pass is
  gain-matched to the western one on land.
- `region`, `valley`: over open water (near-infrared water mask, large water bodies only) the large-scale
  sun glint and detector striping are removed and the base tone is matched to the Blue Marble ocean, so
  plates dissolve without a visible patch. Waves, plumes, surf, clouds and their shadows are kept.
- All three: reprojection to lon/lat by the data API, JPEG encoding.
- Clouds present in the scenes (offshore, and north-west of the valley) are real and left in.

## Still frames

`still-orbit.jpg` (1920×1200), `still-orbit-portrait.jpg` (900×1600) and `still-field.jpg` (1600×1600) are
frames of this globe's own renderer (headless Chromium, overlay off), used when motion is reduced or WebGL2
is missing. They contain only the imagery above: the orbit stills NASA Blue Marble and VIIRS; the field
still "Contains modified Copernicus Sentinel data 2026".

## Rendering notes (what is not imagery)

- Sun, atmosphere rim, limb haze, stars and the day/night light are drawn in the shader.
- The sun's azimuth is compositional (top left of the orbit frame, as in the approved direction); it is
  not an ephemeris position for a date. Its elevation rises with the dive.
- Relief contrast at orbit comes from the shaded relief already present in the Blue Marble plates,
  deepened under the low sun. The ocean at orbit is tinted towards navy; Blue Marble's own ocean is near black.
- Cloud cover is drawn thin (at most about two-thirds opaque) and thinned further in a band along the coast
  around the target so the ground stays readable; it fades out below about 300 km altitude.
