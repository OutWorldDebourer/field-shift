"""Score, foley and narration for the Field Shift demo film, from the same timeline as the picture.
usage: python mix.py <work dir> <lang en|es>   ->  <work>/mix-<lang>.wav (and <work>/music.wav once)
Sound palette: Satellite Briefing telemetry (FM pings, ticks, pulse bass, riser, timpani) for orbit and dive;
Mascot Screencast hi-fi bed (upright piano, pizzicato bass, shaker, hi-hat, glockenspiel) with UI foley in the app.
All instruments are CC0 samples (VSCO 2 CE, VCSL) played by the Lemo-Opuscar sampler, or synthesised."""
import sys, os, json, subprocess, tempfile
import numpy as np, soundfile as sf, soxr
LIB = os.environ.get('LIB', 'C:/Users/OutWorldDevourer/lemo-opuscar')
sys.path.insert(0, LIB)
from core.audio import sampler as S
from core.audio.sfx import SR, add, limit, compress, click, whoosh, noise, lp, hp, bp, t_, env_exp, norm, brown

HERE = os.path.dirname(os.path.abspath(__file__))
WORK, LANG = sys.argv[1], sys.argv[2]
TL = json.load(open(os.path.join(HERE, 'timeline.json'), encoding='utf-8'))
EV = json.load(open(os.path.join(WORK, 'events.json'), encoding='utf-8'))['ev']
DUR = TL['dur']; N = int(DUR * SR)
S.seed(7); np.random.seed(7)
BEAT = 60 / 96

def db(x): return 10 ** (x / 20)
def ev(kind): return [e['t'] for e in EV if e['type'] == kind]
def fm(f, d, idx=3.0, ratio=1.41, tau=.18, v=1.0):
    tt = t_(d); e = env_exp(d, tau)
    return np.sin(2 * np.pi * f * tt + idx * e * np.sin(2 * np.pi * f * ratio * tt)) * e * v * np.minimum(1, tt / .003)
def sawpad(freqs, d, cutoff, v=1.0):
    tt = t_(d); x = np.zeros_like(tt)
    for f in freqs:
        for det in (-0.12, 0.0, 0.13):
            ph = np.random.rand()
            x += 2 * ((tt * f * 2 ** (det / 12) + ph) % 1) - 1
    return norm(lp(x, cutoff, 2)) * v
def ramp(d, a, b, fade=.4):
    tt = t_(d); g = np.ones_like(tt)
    k = int(fade * SR); g[:k] *= np.linspace(0, 1, k); g[-k:] *= np.linspace(1, 0, k)
    return g

# ------------------------------------------------------------------ music
def music():
    m = np.zeros((N, 2), np.float32)
    LOCK = ev('lock')[0]; OPEN = ev('open')[0]; CLOSE = ev('close')[0]; END = ev('end')[0]
    # orbit hum and drone (D), opening up during the dive
    d = OPEN - 0.25
    hum = lp(brown(d), 180) ; hum = norm(hum) * ramp(d, 0, 0, .8)
    add(m, hum.astype(np.float32), 0, .10, 0)
    pad = sawpad([73.42, 110.0, 146.83], d, 500) * np.linspace(.6, 1, len(t_(d)))
    pad2 = sawpad([73.42, 110.0, 146.83, 220.0], d, 1600) * np.linspace(0, 1, len(t_(d))) ** 2
    add(m, (pad * ramp(d, 0, 0, 1.0)).astype(np.float32), 0, .09, -.2)
    add(m, (pad2 * ramp(d, 0, 0, .3)).astype(np.float32), 0, .05, .2)
    # pulse bass with 2nd/3rd harmonics, on the beat from the dive
    t = 1.1
    while t < LOCK - 0.05:
        dd = .32; tt = t_(dd); f = 73.42
        x = (np.sin(2 * np.pi * f * tt) + .5 * np.sin(4 * np.pi * f * tt) + .3 * np.sin(6 * np.pi * f * tt)) * env_exp(dd, .09)
        add(m, x.astype(np.float32), t, .16 * (0.6 + 0.4 * (t / LOCK)), 0); t += BEAT
    # telemetry ticks that divide to accelerate into the lock-on
    t = 0.3
    while t < LOCK - 0.03:
        step = BEAT if t < 4.0 else BEAT / 2 if t < 6.4 else BEAT / 4
        add(m, (hp(click(1.6, 1), 1500)).astype(np.float32), t, .07, .35 * np.sin(t * 3)); t += step
    # riser into the lock
    d = LOCK - 3.6; tt = t_(d); x = noise(d); out = np.zeros_like(x)
    for i in range(0, len(x), 960):
        f = 300 + 3200 * (i / len(x)) ** 2; hi = min(len(x), i + 960)
        out[i:hi] = bp(x[max(0, i - 4000):hi], f * .8, f * 1.25)[-(hi - i):]
    add(m, (norm(out) * (tt / d) ** 2).astype(np.float32), 3.6, .10, 0)
    # searching ping at orbit, the lock-on ping (it returns when the window closes onto the field)
    add(m, fm(1318.5, 1.2, 2.0, 1.5, .25).astype(np.float32), 0.35, .10, -.3)
    for tl in ev('lock'):
        add(m, fm(1760, 1.6, 2.5, 1.5, .32).astype(np.float32), tl, .20, 0)
        add(m, fm(880, 1.6, 1.5, 2.0, .4).astype(np.float32), tl, .10, 0)
    add(m, S.hit('timpani', 'drum2', .75), LOCK, .55, 0)
    add(m, S.note('horn_stac', 'D3', .5, .7), LOCK, .22, 0)
    # ---- the app: a hi-fi bed (upright piano, pizzicato bass, shaker, hats, glockenspiel)
    chords = [('D2', ['A3', 'D4', 'E4', 'F#4']), ('B1', ['F#3', 'B3', 'C#4', 'D4']), ('G1', ['F#3', 'A3', 'B3', 'D4']), ('A1', ['E3', 'A3', 'B3', 'D4'])]
    bar = 4 * BEAT; t0 = OPEN + 0.05; k = 0; t = t0
    PROMISE = ev('promise')[0]
    score = []
    while t < CLOSE - 0.2:
        root, ch = chords[k % 4]
        if t >= PROMISE - 0.1: break   # the band drops under the promise: only the held pad remains
        score.append((t, 'upright', ch[0], bar, .40, -.15))
        for j, p in enumerate(ch[1:]): score.append((t + .012 * (j + 1), 'upright', p, bar, .38, -.1 + .1 * j))
        score.append((t + 2 * BEAT, 'upright', ch[-1], BEAT * 2, .30, .2))
        if t < PROMISE - 0.1:
            score.append((t, 'contrabass_pizz', S.midi(root) + 12, BEAT * 1.5, .62, 0))
            score.append((t + 2 * BEAT, 'contrabass_pizz', S.midi(root) + 19, BEAT * 1.5, .5, 0))
        t += bar; k += 1
    # shaker 8ths from chapter 02 to the promise; soft hats on 2 and 4 from chapter 03
    t = 10.35
    while t < PROMISE - 0.1:
        score.append((t, 'shaker', 'down' if int(round(t / (BEAT / 2))) % 2 == 0 else 'up', None, .32, .35, .5)); t += BEAT / 2
    t = 13.55 + BEAT
    while t < PROMISE - 0.1:
        score.append((t, 'hihat', 'closed', None, .35, -.3, .45)); t += 2 * BEAT
    # the held pad under the promise
    score.append((PROMISE, 'vibraphone_bowed', 'B4', CLOSE - PROMISE + .4, .4, -.2, .45))
    score.append((PROMISE, 'vibraphone_bowed', 'F#4', CLOSE - PROMISE + .4, .35, .2, .45))
    # glockenspiel on the amber reading, the one measured hit
    AMB = ev('amber')[0]
    score += [(AMB, 'glockenspiel', 'A5', 1.5, .62, .1), (AMB + .01, 'glockenspiel', 'D6', 1.5, .55, -.1), (AMB, 'vibraphone', 'D4', 1.6, .55, 0)]
    # ending: telemetry pad back, timpani and a bowed chord under the end card
    score += [(END, 'vibraphone_bowed', 'D4', DUR - END, .5, -.2, .7), (END, 'vibraphone_bowed', 'A4', DUR - END, .45, .2, .7),
              (END, 'vibraphone_bowed', 'F#4', DUR - END, .4, 0, .6), (END, 'contrabass', 'D2', DUR - END, .5, 0, .7)]
    bed = S.render(score, dur=DUR, master=False)[:N]
    m[:len(bed)] += bed * .55
    add(m, S.hit('timpani', 'drum2', .55), END, .38, 0)
    d = DUR - CLOSE - 0.3
    add(m, (sawpad([73.42, 110.0, 146.83, 220.0], d, 900) * ramp(d, 0, 0, 1.2)).astype(np.float32), CLOSE + .3, .06, 0)
    t = CLOSE + .35
    while t < END + 1.6:   # ticks slowing down as the camera climbs
        add(m, hp(click(1.6, 1), 1500).astype(np.float32), t, .05, .3); t += BEAT / 4 if t < CLOSE + 1.2 else BEAT / 2 if t < END + .5 else BEAT
    # suck-outs: music down ~25 dB for a moment before the turns
    g = np.ones(N, np.float32)
    for ts in ev('suck'):
        a, b = int((ts) * SR), int((ts + 0.3) * SR)
        env = np.ones(N, np.float32); fall = int(.06 * SR); rise = int(.08 * SR)
        env[a:a + fall] = np.linspace(1, db(-25), fall); env[a + fall:b] = db(-25); env[b:b + rise] = np.linspace(db(-25), 1, rise)
        g *= env
    m *= g[:, None]
    # tail fade
    f = int(1.0 * SR); m[-f:] *= np.linspace(1, 0, f)[:, None]
    return m

# ------------------------------------------------------------------ foley
def foley():
    x = np.zeros((N, 2), np.float32)
    for t in ev('click'):
        add(x, click(1.0, 1).astype(np.float32), t, .22, .1)
        add(x, click(.7, 1).astype(np.float32), t + .07, .10, .1)      # release
    for t in ev('photos'):
        for i, dt in enumerate((0, .09, .18, .27)):                      # four photos land: soft pops
            d = .08; tt = t_(d); p = np.sin(2 * np.pi * (520 + 120 * i) * tt * (1 + 2 * np.exp(-tt / .01))) * env_exp(d, .02)
            add(x, p.astype(np.float32), t + dt, .16, -.4 + .25 * i)
        d = .14; sh = hp(noise(d), 1800) * (env_exp(d, .012) + .6 * np.roll(env_exp(d, .012), int(.06 * SR)))
        add(x, norm(sh).astype(np.float32), t - .08, .09, 0)              # a shutter
    for t in ev('whip'):
        add(x, whoosh(.36, 1).astype(np.float32), t - .18, .20, 0)
        add(x, S.hit('toms', 'low_mallet', .35), t + .02, .18, 0)
    for t in ev('snap'):
        add(x, fm(2093, .25, 1.2, 2.0, .05).astype(np.float32), t + .05, .06, .2)
    for t in ev('open') + ev('close'):
        add(x, whoosh(.55, 1).astype(np.float32), t - .1, .14, 0)
    return x

# ------------------------------------------------------------------ voice
def voice():
    v = np.zeros((N, 2), np.float32)
    src = os.path.join(HERE, '..', 'voice', f'vo-{LANG}.wav')
    info = TL['vo'][LANG]; tempo = info['tempo']
    if tempo != 1.0:
        tmp = os.path.join(WORK, f'vo-{LANG}-tempo.wav')
        subprocess.run(['ffmpeg', '-y', '-loglevel', 'error', '-i', src, '-af', f'atempo={tempo}', '-ar', '24000', tmp], check=True)
        src = tmp
    y, sr = sf.read(src, dtype='float32')
    if y.ndim > 1: y = y.mean(1)
    y = soxr.resample(y, sr, SR)
    y = hp(y, 80).astype(np.float32)
    y = compress(y / (np.abs(y).max() + 1e-9) * .9, thr=.22, ratio=3.0).astype(np.float32)
    mask = np.zeros(N, np.float32)
    for c in info['clips']:
        a, b = int(c['in'] / tempo * SR), int(c['out'] / tempo * SR)
        seg = y[a:b].copy(); f = int(.012 * SR); seg[:f] *= np.linspace(0, 1, f); seg[-f:] *= np.linspace(1, 0, int(f))
        add(v, seg, c['at'], .5, 0)
        s0 = int(c['at'] * SR); mask[s0:s0 + len(seg)] = 1
    return v, mask

M = music(); F = foley(); V, mask = voice()
# duck the music under the voice (4 dB, smooth), then set voice ~10 dB over music by RMS where both play
from scipy.ndimage import uniform_filter1d
duck = 1 - (1 - db(-4)) * np.clip(uniform_filter1d(mask, int(.25 * SR)), 0, 1)
M *= duck[:, None]
on = mask > 0
rms = lambda a: float(np.sqrt(np.mean(a[on] ** 2)) + 1e-12)
gv = db(10) * rms(M + F) / rms(V)
mix = M + F + V * gv
for ch in range(2): mix[:, ch] = limit(mix[:, ch], .89)
sf.write(os.path.join(WORK, f'mix-{LANG}.wav'), mix, SR, subtype='PCM_24')
print(LANG, 'voice gain', round(20 * np.log10(gv), 1), 'dB; peak', round(float(np.abs(mix).max()), 3))
