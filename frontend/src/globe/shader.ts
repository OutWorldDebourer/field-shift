// Ray-sphere globe shader: imagery plates sampled by lon/lat where the camera ray meets the ground,
// nested by resolution with feathered edges, day/night across a sun terminator, clouds, limb haze,
// atmosphere rim, sun flare and stars.
//
// The ray-sphere set-up, the nested feathered plates and the limb shading are adapted from the
// Satellite Briefing demo engine (engine.js), © 2026 Lemomo/LemoLab, MIT. New here: the local-frame
// ground solve (metre precision near the ground), lighting, night lights, clouds, sun and stars.

export const PLATE_SLOTS = 5;

export const VERT = /* glsl */ `
in vec3 position;
void main() { gl_Position = vec4(position.xy, 0., 1.); }
`;

export const FRAG = /* glsl */ `
precision highp float;
precision highp sampler2D;
out vec4 o;

uniform vec2 uRes, uShift;
uniform float uF;
uniform vec3 uR, uU, uFw, uCl;          // camera basis and position, local ENU frame of the look-at point (km)
uniform float uC2;                      // |C|^2 - R^2 from double precision
uniform float uSinLat0, uCosLat0, uLat0, uLon0;

uniform sampler2D tWorld, tNight, tNightR, tCloud;
uniform sampler2D tP0, tP1, tP2, tP3, tP4;
uniform vec4 bP0, bP1, bP2, bP3, bP4;   // plate boxes (w, s, e, n) in degrees relative to the look-at point
uniform vec4 gP0, gP1, gP2, gP3, gP4;   // alpha, feather, gain, texture size (0 = plain bilinear)
uniform vec4 bCloud;                    // regional box shared by the cloud and night-lights plates
uniform float uCloud, uNightR;
uniform vec2 uCloudShift;               // cloud shadow offset in plate uv (towards the sun)
uniform vec3 uClear;                    // thinner cloud around the target: dlat, dlon (deg, relative), radius (deg)
uniform vec2 uSunPx;                    // where the sun sits on screen (device px, origin bottom-left)
uniform float uSunFlare;

uniform vec3 uSun;                      // light direction on the ground, local frame
uniform vec3 uSunVis;                   // where the sun is drawn, local frame
uniform float uSunAmt, uExpo, uShade, uRelief, uWarm, uOcean, uTime;

const float RE = 6371.0088;
const float PI = 3.14159265;

float lum(vec3 c) { return dot(c, vec3(.299, .587, .114)); }
float hash(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
float hash3(vec3 p) { return fract(sin(dot(p, vec3(127.1, 311.7, 74.7))) * 43758.5453); }

float inside(vec2 uv, float f) {
  vec2 e = smoothstep(vec2(0.), vec2(f), uv) * smoothstep(vec2(0.), vec2(f), 1. - uv);
  return e.x * e.y;
}

// Catmull-Rom in 9 bilinear taps: keeps 10 m imagery smooth (not blocky) when the camera is close.
vec3 texCR(sampler2D t, vec2 uv, float size) {
  vec2 sp = uv * size;
  vec2 t1 = floor(sp - .5) + .5;
  vec2 f = sp - t1;
  vec2 w0 = f * (-.5 + f * (1. - .5 * f));
  vec2 w1 = 1. + f * f * (-2.5 + 1.5 * f);
  vec2 w2 = f * (.5 + f * (2. - 1.5 * f));
  vec2 w3 = f * f * (-.5 + .5 * f);
  vec2 w12 = w1 + w2;
  vec2 p0 = (t1 - 1.) / size, p3 = (t1 + 2.) / size, p12 = (t1 + w2 / w12) / size;
  vec3 c = vec3(0.);
  c += textureLod(t, vec2(p0.x, p0.y), 0.).rgb * w0.x * w0.y;
  c += textureLod(t, vec2(p12.x, p0.y), 0.).rgb * w12.x * w0.y;
  c += textureLod(t, vec2(p3.x, p0.y), 0.).rgb * w3.x * w0.y;
  c += textureLod(t, vec2(p0.x, p12.y), 0.).rgb * w0.x * w12.y;
  c += textureLod(t, vec2(p12.x, p12.y), 0.).rgb * w12.x * w12.y;
  c += textureLod(t, vec2(p3.x, p12.y), 0.).rgb * w3.x * w12.y;
  c += textureLod(t, vec2(p0.x, p3.y), 0.).rgb * w0.x * w3.y;
  c += textureLod(t, vec2(p12.x, p3.y), 0.).rgb * w12.x * w3.y;
  c += textureLod(t, vec2(p3.x, p3.y), 0.).rgb * w3.x * w3.y;
  return max(c, 0.);
}

// rgb + relief term (local contrast of the shaded-relief plates; zero for the close-range plates)
vec4 plate(sampler2D t, vec4 b, vec4 g, vec2 dll, vec4 base) {
  if (g.x <= .002) return base;
  vec2 uv = vec2((dll.y - b.x) / (b.z - b.x), (b.w - dll.x) / (b.w - b.y));
  float w = inside(uv, g.y) * g.x;
  if (w <= .002) return base;
  vec3 c;
  float hp = 0.;
  if (g.w > 0.) {
    float mag = 1. / max(max(fwidth(uv.x), fwidth(uv.y)) * g.w, 1e-6);
    c = texture(t, uv).rgb;
    if (mag > 1.15) c = mix(c, texCR(t, uv, g.w), smoothstep(1.15, 1.8, mag));
  } else {
    c = texture(t, uv).rgb;
    float m1 = lum(texture(t, uv, 3.2).rgb);
    hp = lum(c) - m1 + .7 * (m1 - lum(texture(t, uv, 5.6).rgb));
  }
  return mix(base, vec4(c * g.z, hp), w);
}

vec3 stars(vec3 d) {
  vec3 p = d * 190.;
  vec3 i = floor(p), f = fract(p) - .5;
  float h = hash3(i);
  float s = step(.993, h) * smoothstep(.26, .04, length(f)) * 1.3;
  float tw = .75 + .25 * sin(uTime * (1.5 + 4. * fract(h * 57.3)) + h * 40.);
  return vec3(.85, .9, 1.) * s * (.35 + .65 * fract(h * 91.7)) * tw;
}

vec3 sunGlow(float cs, float hit, vec2 frag) {
  float ang = acos(clamp(cs, -1., 1.));
  float a = ang / .014;
  vec3 c = vec3(1., .96, .82) * smoothstep(1.35, .5, a) * 6. * (1. - hit);           // white-hot core
  c += vec3(1., .70, .26) * (1. / (1. + a * a * .30)) * mix(2.2, .85, hit);          // saturated inner halo
  c += vec3(1., .42, .10) * exp(-ang * 10.5) * mix(.95, .60, hit);                   // orange bloom
  c += vec3(.95, .26, .06) * exp(-ang * 4.6) * mix(.13, .085, hit);                  // ember haze
  // flare: a thin streak along the limb and a few faint rays
  vec2 d = (frag - uSunPx) / uF;
  float r = length(d);
  float streak = exp(-abs(d.y) * 110.) * exp(-abs(d.x) * 2.4) * .50 + exp(-abs(d.y) * 34.) * exp(-abs(d.x) * 4.5) * .22;
  float th = atan(d.y, d.x);
  float rays = pow(abs(cos(th * 3. + .5)), 60.) + .5 * pow(abs(cos(th * 2. - .35)), 110.);
  c += vec3(1., .58, .22) * (streak + rays * exp(-r * 10.) * smoothstep(.0, .03, r) * .20) * uSunFlare * mix(1., .55, hit);
  return c * uSunAmt;
}

void main() {
  vec2 frag = gl_FragCoord.xy;
  vec2 n = (frag - uRes * .5 - uShift * vec2(1., -1.)) / uF;
  vec3 dir = normalize(uFw + n.x * uR + n.y * uU);
  vec3 Cc = -(uCl + vec3(0., 0., RE));               // camera -> Earth centre
  float b = dot(dir, Cc);
  float disc = b * b - uC2;
  float cs = dot(dir, uSunVis);
  float sunProx = pow(max(cs, 0.), 34.);
  vec3 blue = vec3(.26, .54, 1.);
  vec3 warm = vec3(1., .58, .26);
  vec3 col;

  if (disc < 0. || b < 0.) {
    // space: stars, the atmosphere rim, the sun
    float dmin = sqrt(max(dot(Cc, Cc) - b * b, 0.));
    float x = max(dmin - RE, 0.) / 26.;
    vec3 pm = normalize(b * dir - Cc);
    float litL = smoothstep(-.22, .18, dot(pm, uSun));
    float rim = (exp(-x * 1.05) * 1.9 + exp(-x * .26) * .50) * step(0., b);
    vec3 rimCol = mix(blue + vec3(.42, .36, 0.) * exp(-x * 2.4), warm * 1.3, clamp(sunProx * 1.3, 0., 1.));
    col = vec3(.008, .013, .026) + stars(dir) * (1. - clamp(rim * 2., 0., 1.));
    col += rimCol * rim * (.10 + .90 * litL) * uShade;
    col += sunGlow(cs, 0., frag);
  } else {
    float t = uC2 / (b + sqrt(disc));                // conjugate form: no cancellation near the ground
    vec3 H = uCl + t * dir;                          // hit point, km from the look-at point
    float e = H.x, nn = H.y, z = RE + H.z;
    float A = z * uCosLat0 - nn * uSinLat0;
    float B = nn * uCosLat0 + z * uSinLat0;
    float rho = sqrt(A * A + e * e);
    float dlon = degrees(atan(e, A));
    float dlat = degrees(atan(nn - uSinLat0 * e * e / (rho + A), rho * uCosLat0 + B * uSinLat0));
    vec2 dll = vec2(dlat, dlon);
    vec3 N = normalize(vec3(e, nn, z));

    // imagery: world, then nested plates
    vec2 uw = vec2((uLon0 + dlon + 180.) / 360., (90. - (uLat0 + dlat)) / 180.);
    vec2 uwd = vec2(dlon / 360., -dlat / 180.);      // seam-free derivatives
    vec4 g = vec4(textureGrad(tWorld, uw, dFdx(uwd), dFdy(uwd)).rgb, 0.);
    g = plate(tP0, bP0, gP0, dll, g);
    g = plate(tP1, bP1, gP1, dll, g);
    g = plate(tP2, bP2, gP2, dll, g);
    g = plate(tP3, bP3, gP3, dll, g);
    g = plate(tP4, bP4, gP4, dll, g);
    vec3 alb = g.rgb;

    // light: a soft terminator, warm at a low sun
    float mu = dot(N, uSun);
    float lit = smoothstep(-.06, .46, mu);
    float low = 1. - smoothstep(.04, .55, mu);
    vec3 sunCol = mix(vec3(1.), vec3(1.10, .83, .58), low * uWarm);
    vec3 amb = vec3(.050, .082, .150) * smoothstep(-.30, .10, mu);
    // relief: the shaded-relief plates carry their own hillshade (lit from the north-west); a low sun deepens it
    float rl = g.a * uRelief * (.55 + .9 * low);
    alb *= exp(rl * 3.6);
    alb += rl * vec3(.55, .20, -.34);
    alb = max(alb, 0.);
    // open water: Blue Marble's ocean is near black; give it the sky it reflects
    float water = smoothstep(-.02, .012, alb.b - alb.r) * (1. - smoothstep(.07, .17, lum(alb))) * uOcean;
    alb = mix(alb, vec3(.034, .081, .169) + alb * .35, water * .92);
    col = alb * (lit * sunCol * uExpo + amb);

    // clouds (regional plate) and their shadow
    vec2 uc = vec2((dlon - bCloud.x) / (bCloud.z - bCloud.x), (bCloud.w - dlat) / (bCloud.w - bCloud.y));
    float cw = inside(uc, .07) * uCloud;
    if (cw > .002) {
      // thin veil; thinner still in a band along the coast around the target, so the ground stays readable
      vec2 cp = vec2((dlon - uClear.y) * uCosLat0, dlat - uClear.x);
      float ac = dot(cp, vec2(.78, .62));               // + inland, - out to sea
      float cd = length(vec2(dot(cp, vec2(-.62, .78)) / 3.4, ac < 0. ? ac * 2.4 : ac)) / uClear.z;
      float clear = mix(.06, 1., smoothstep(.45, 1.25, cd));
      float cl = smoothstep(.12, .95, texture(tCloud, uc, .8).r) * cw * clear * .66;
      float sh = smoothstep(.12, .95, texture(tCloud, uc + uCloudShift, 1.6).r) * cw * clear * .66;
      col *= 1. - .30 * sh * (1. - cl);
      vec3 cc = (.30 + .70 * lit) * mix(vec3(1.), vec3(1.08, .92, .78), low * uWarm) * .98 + amb * 1.6;
      col = mix(col, cc, cl);
    }

    // night lights in the twilight band and beyond
    float nl = 1. - smoothstep(.0, .20, mu);
    if (nl > .002) {
      vec3 nt = textureGrad(tNight, uw, dFdx(uwd), dFdy(uwd)).rgb;
      float nw = inside(uc, .05) * uNightR;
      if (nw > .002) nt = mix(nt, texture(tNightR, uc).rgb, nw);
      float l = max(lum(nt) - .035, 0.);
      col += vec3(1., .80, .50) * l * l * 5.5 * nl;
    }

    // limb: haze thickens towards the edge of the disc, warm near the sun
    float muV = clamp(dot(N, -dir), 0., 1.);
    float haze = pow(1. - muV, 3.4) * uShade;
    vec3 hazeCol = mix(vec3(.36, .58, 1.) * .95, warm * 1.1, clamp(sunProx * 1.4 + pow(max(cs, 0.), 9.) * .35, 0., 1.)) * (.14 + .86 * lit);
    col = mix(col, hazeCol, clamp(haze * .92, 0., 1.));
    col += hazeCol * pow(1. - muV, 8.) * .80 * uShade;
    col += sunGlow(cs, 1., frag) * (.35 + .65 * uShade);
  }

  // shoulder, vignette, dither
  float mx = max(col.r, max(col.g, col.b));          // on the brightest channel, so the sun keeps its orange
  if (mx > .82) {
    vec3 hue = col * ((.82 + .18 * (1. - exp(-(mx - .82) / .18))) / mx);
    col = mix(hue, min(col, vec3(1.)), smoothstep(1.3, 3.2, mx));          // only the core burns to white
  }
  vec2 q = frag / uRes - .5;
  col *= 1. - .30 * dot(q, q);
  col += (hash(frag + fract(uTime) * 61.) - .5) / 255.;
  o = vec4(clamp(col, 0., 1.), 1.);
}
`;
