"""Score, foley and narration for the Field Shift tutorial film, from the same timeline as the picture.
usage: python mix.py <work dir> <lang en|es>   ->  <work>/mix-<lang>.wav
Sound palette (as the 30 s film): Satellite Briefing telemetry (FM pings, ticks, pulse bass, riser, timpani) for orbit, dive and climb;
Mascot Screencast hi-fi bed (upright piano, pizzicato bass, shaker, hi-hat, vibraphone, glockenspiel) with UI foley in the app.
All instruments are CC0 samples (VSCO 2 CE, VCSL) played by the Lemo-Opuscar sampler, or synthesised."""
import sys, os, json, subprocess
import numpy as np, soundfile as sf, soxr
LIB = os.environ.get('LIB', 'C:/Users/OutWorldDevourer/lemo-opuscar')
sys.path.insert(0, LIB)
from core.audio import sampler as S
from core.audio.sfx import SR, add, limit, compress, click, whoosh, noise, lp, hp, bp, t_, env_exp, norm, brown

HERE = os.path.dirname(os.path.abspath(__file__))
WORK, LANG = sys.argv[1], sys.argv[2]
TL = json.load(open(os.path.join(HERE, f'timeline-{LANG}.json'), encoding='utf-8'))
EV = json.load(open(os.path.join(WORK, 'events.json'), encoding='utf-8'))['ev']
DUR = TL['dur']; N = int(DUR * SR)
S.seed(11); np.random.seed(11)
BEAT = 60 / 92

def db(x): return 10 ** (x / 20)
def ev(kind): return [e['t'] for e in EV if e['type'] == kind]
def fm(f, d, idx=3.0, ratio=1.41, tau=.18, v=1.0):
    tt = t_(d); e = env_exp(d, tau)
    return np.sin(2 * np.pi * f * tt + idx * e * np.sin(2 * np.pi * f * ratio * tt)) * e * v * np.minimum(1, tt / .003)
def sawpad(freqs, d, cutoff, v=1.0):
    tt = t_(d); x = np.zeros_like(tt)
    for f in freqs:
        for det in (-0.12, 0.0, 0.13):
            ph = np.random.rand(); x += 2 * ((tt * f * 2 ** (det / 12) + ph) % 1) - 1
    return norm(lp(x, cutoff, 2)) * v
def fit(x, g):
    n = min(len(x), len(g)); return x[:n] * g[:n]
def ramp(d, fade=.4):
    g = np.ones(int(d * SR)); k = int(fade * SR); g[:k] *= np.linspace(0, 1, k); g[-k:] *= np.linspace(1, 0, k); return g

def music():
    m = np.zeros((N, 2), np.float32)
    LOCK = ev('lock')[0]; DIVE = ev('dive')[0]; OPEN = ev('open')[0]; CLOSE = ev('close')[0]; END = ev('end')[0]
    PROMISE = ev('promise')[0]; AMB = ev('amber')[0]
    CH = {e['n']: e['t'] for e in EV if e['type'] == 'chapter'}
    # ---- orbit: hum and a D drone that opens up through the dive
    d = OPEN - 0.25
    add(m, fit(norm(lp(brown(d), 180)), ramp(d, .8)).astype(np.float32), 0, .06, 0)
    pad = sawpad([73.42, 110.0, 146.83], d, 500)[:int(d * SR)] * np.linspace(.6, 1, int(d * SR))
    pad2 = sawpad([73.42, 110.0, 146.83, 220.0], d, 1600)[:int(d * SR)] * np.clip((np.linspace(0, d, int(d * SR)) - DIVE) / (LOCK - DIVE), 0, 1) ** 2
    add(m, fit(pad, ramp(d, 1.0)).astype(np.float32), 0, .09, -.2)
    add(m, fit(pad2, ramp(d, .3)).astype(np.float32), 0, .05, .2)
    # a slow searching ping over the planet, then pulse bass from the dive
    for tp in [0.35] + list(np.arange(5.6, DIVE, 4 * BEAT)):
        add(m, fm(1318.5, 1.2, 2.0, 1.5, .25).astype(np.float32), tp, .07, -.3 + .2 * np.sin(tp))
    t = DIVE
    while t < LOCK - 0.05:
        dd = .32; tt = t_(dd); f = 73.42
        x = (np.sin(2 * np.pi * f * tt) + .5 * np.sin(4 * np.pi * f * tt) + .3 * np.sin(6 * np.pi * f * tt)) * env_exp(dd, .09)
        add(m, x.astype(np.float32), t, .16 * (0.6 + 0.4 * (t - DIVE) / (LOCK - DIVE)), 0); t += BEAT
    t = 0.3
    while t < LOCK - 0.03:   # telemetry ticks that divide to accelerate into the lock-on
        step = BEAT * 2 if t < DIVE else BEAT if t < DIVE + 3 else BEAT / 2 if t < DIVE + 5.5 else BEAT / 4
        add(m, hp(click(1.6, 1), 1500).astype(np.float32), t, .06, .35 * np.sin(t * 3)); t += step
    d = 3.6; tt = t_(d); x = noise(d); out = np.zeros_like(x)
    for i in range(0, len(x), 960):
        f = 300 + 3200 * (i / len(x)) ** 2; hi = min(len(x), i + 960)
        out[i:hi] = bp(x[max(0, i - 4000):hi], f * .8, f * 1.25)[-(hi - i):]
    add(m, (norm(out) * (tt / d) ** 2).astype(np.float32), LOCK - 3.6, .10, 0)
    for tl in ev('lock'):
        add(m, fm(1760, 1.6, 2.5, 1.5, .32).astype(np.float32), tl, .20, 0)
        add(m, fm(880, 1.6, 1.5, 2.0, .4).astype(np.float32), tl, .10, 0)
    add(m, S.hit('timpani', 'drum2', .75), LOCK, .55, 0)
    add(m, S.note('horn_stac', 'D3', .5, .7), LOCK, .22, 0)
    # ---- the app: hi-fi bed; layers enter chapter by chapter and drop under the promise
    chords = [('D2', ['A3', 'D4', 'E4', 'F#4']), ('B1', ['F#3', 'B3', 'C#4', 'D4']), ('G1', ['F#3', 'A3', 'B3', 'D4']), ('A1', ['E3', 'A3', 'B3', 'D4'])]
    bar = 4 * BEAT; t = OPEN + 0.05; k = 0; score = []
    sil = (AMB - 0.55 - 2 * BEAT, AMB)          # one bar of near-silence before the amber reading
    while t < PROMISE - 0.1:
        root, ch = chords[k % 4]
        quiet = sil[0] <= t < sil[1]
        v = .25 if quiet else .40
        score.append((t, 'upright', ch[0], bar, v, -.15))
        for j, p in enumerate(ch[1:]): score.append((t + .012 * (j + 1), 'upright', p, bar, v - .02, -.1 + .1 * j))
        if not quiet: score.append((t + 2 * BEAT, 'upright', ch[-1], BEAT * 2, .30, .2))
        if t >= CH['02'] - .1 and not quiet:
            score.append((t, 'contrabass_pizz', S.midi(root) + 12, BEAT * 1.5, .62, 0))
            score.append((t + 2 * BEAT, 'contrabass_pizz', S.midi(root) + 19, BEAT * 1.5, .5, 0))
        if t >= CH['04'] - .1 and not quiet:     # chapter 04: a vibraphone figure on top
            for j, p in enumerate([ch[2], ch[3], ch[1], ch[3]]):
                score.append((t + j * BEAT, 'vibraphone', S.midi(p) + 12, BEAT * .9, .32, .25 - .15 * j))
        t += bar; k += 1
    t = CH['02']
    while t < PROMISE - 0.1:
        if not (sil[0] <= t < sil[1]): score.append((t, 'shaker', 'down' if int(round(t / (BEAT / 2))) % 2 == 0 else 'up', None, .30, .35, .5))
        t += BEAT / 2
    t = CH['03'] + BEAT
    while t < PROMISE - 0.1:
        if not (sil[0] <= t < sil[1]): score.append((t, 'hihat', 'closed', None, .33, -.3, .45))
        t += 2 * BEAT
    for n, tc in CH.items():   # a small glockenspiel call on every chapter pill
        score += [(tc + .05, 'glockenspiel', 'D6', .8, .32, .2), (tc + .05 + BEAT / 2, 'glockenspiel', 'A5', .8, .28, -.2)]
    score += [(AMB, 'glockenspiel', 'A5', 1.5, .62, .1), (AMB + .01, 'glockenspiel', 'D6', 1.5, .55, -.1), (AMB, 'vibraphone', 'D4', 1.6, .55, 0)]
    score.append((PROMISE, 'vibraphone_bowed', 'B4', CLOSE - PROMISE + .4, .4, -.2, .45))
    score.append((PROMISE, 'vibraphone_bowed', 'F#4', CLOSE - PROMISE + .4, .35, .2, .45))
    score += [(END, 'vibraphone_bowed', 'D4', DUR - END, .5, -.2, .7), (END, 'vibraphone_bowed', 'A4', DUR - END, .45, .2, .7),
              (END, 'vibraphone_bowed', 'F#4', DUR - END, .4, 0, .6), (END, 'contrabass', 'D2', DUR - END, .5, 0, .7)]
    bed = S.render(score, dur=DUR, master=False)[:N]
    m[:len(bed)] += bed * .55
    add(m, S.hit('timpani', 'drum2', .55), END, .38, 0)
    # ---- climb: telemetry pad back, ticks slowing down
    d = DUR - CLOSE - 0.3
    add(m, fit(sawpad([73.42, 110.0, 146.83, 220.0], d, 900), ramp(d, 1.2)).astype(np.float32), CLOSE + .3, .06, 0)
    t = CLOSE + .35
    while t < END + 1.6:
        add(m, hp(click(1.6, 1), 1500).astype(np.float32), t, .05, .3); t += BEAT / 4 if t < CLOSE + 1.2 else BEAT / 2 if t < CLOSE + 4 else BEAT
    # suck-outs: music down ~25 dB for a moment before the turns
    g = np.ones(N, np.float32)
    for ts in ev('suck'):
        a, b = int(ts * SR), int((ts + 0.3) * SR); fall, rise = int(.06 * SR), int(.08 * SR)
        env = np.ones(N, np.float32); env[a:a + fall] = np.linspace(1, db(-25), fall); env[a + fall:b] = db(-25); env[b:b + rise] = np.linspace(db(-25), 1, rise)
        g *= env
    m *= g[:, None]
    f = int(1.5 * SR); m[-f:] *= np.linspace(1, 0, f)[:, None]
    return m

def foley():
    x = np.zeros((N, 2), np.float32)
    for t in ev('click'):
        add(x, click(1.0, 1).astype(np.float32), t, .22, .1); add(x, click(.7, 1).astype(np.float32), t + .07, .10, .1)
    for t in ev('photos'):
        for i, dt in enumerate((0, .09, .18, .27)):
            d = .08; tt = t_(d); p = np.sin(2 * np.pi * (520 + 120 * i) * tt * (1 + 2 * np.exp(-tt / .01))) * env_exp(d, .02)
            add(x, p.astype(np.float32), t + dt, .16, -.4 + .25 * i)
        d = .14; sh = hp(noise(d), 1800) * (env_exp(d, .012) + .6 * np.roll(env_exp(d, .012), int(.06 * SR)))
        add(x, norm(sh).astype(np.float32), t - .08, .09, 0)
    for t in ev('whip'):
        add(x, whoosh(.36, 1).astype(np.float32), t - .18, .20, 0); add(x, S.hit('toms', 'low_mallet', .35), t + .02, .18, 0)
    for t in ev('swish'):
        add(x, lp(whoosh(.3, 1), 3000).astype(np.float32), t - .05, .09, 0)
    for t in ev('snap'):
        add(x, fm(2093, .25, 1.2, 2.0, .05).astype(np.float32), t + .05, .06, .2)
    # the opening: a cut-whoosh and a searching ping per region, a tick per aphid generation (season brighter, right;
    # normal softer, left), a short spray hiss per calendar mark, an FM ping as each footprint locks on during the dive
    for t in ev('hop'):
        add(x, whoosh(.28, 1).astype(np.float32), t - .14, .16, 0)
        add(x, fm(1318.5, .9, 2.0, 1.5, .2).astype(np.float32), t + .08, .07, .25 * np.sin(t * 2))
    for e in EV:
        if e['type'] == 'gen':
            k = hp(click(2.4 if e.get('n') else 1.4, 1), 2500 if e.get('n') else 1200)
            add(x, k.astype(np.float32), e['t'], .11 if e.get('n') else .07, .35 if e.get('n') else -.35)
    for t in ev('spray'):
        d = .14; add(x, (norm(bp(noise(d), 3000, 7000)) * env_exp(d, .05)).astype(np.float32), t, .05, .3)
    for t in ev('ladder'):
        add(x, fm(1046.5, 1.1, 1.8, 1.5, .25).astype(np.float32), t, .09, 0)
        add(x, fm(1568, 1.1, 1.2, 2.0, .22).astype(np.float32), t + .02, .05, 0)
    for t in ev('open') + ev('close'):
        add(x, whoosh(.55, 1).astype(np.float32), t - .1, .14, 0)
    # soft room tone under the app, thin air under orbit and climb
    o, c = ev('open')[0], ev('close')[0]
    d = c - o; add(x, fit(norm(lp(brown(d), 900)), ramp(d, 1.0)).astype(np.float32), o, .018, 0)
    for a, b in ((0, o), (c, DUR)):
        d = b - a; add(x, fit(norm(bp(noise(d), 2500, 7000)), ramp(d, .8)).astype(np.float32), a, .012, 0)
    return x

def voice():
    v = np.zeros((N, 2), np.float32); mask = np.zeros(N, np.float32)
    for c in TL['clips']:
        src = os.path.join(HERE, '..', 'voice', c['file'])
        k = c.get('tempo', 1.0)
        if k != 1.0:   # tightened with atempo, pitch kept
            tmp = os.path.join(WORK, 'tempo-' + c['file'])
            subprocess.run(['ffmpeg', '-y', '-loglevel', 'error', '-i', src, '-af', f'atempo={k}', '-ar', '24000', tmp], check=True)
            src = tmp
        y, sr = sf.read(src, dtype='float32')
        if y.ndim > 1: y = y.mean(1)
        y = soxr.resample(y, sr, SR)[int(c['in'] / k * SR):]
        y = hp(y, 80).astype(np.float32)
        y = compress(y / (np.abs(y).max() + 1e-9) * .9, thr=.22, ratio=3.0).astype(np.float32)
        f = int(.015 * SR); y[:f] *= np.linspace(0, 1, f); y[-f:] *= np.linspace(1, 0, f)
        add(v, y, c['at'], .5, 0)
        s0 = int(c['at'] * SR); mask[s0:s0 + len(y)] = 1
    return v, mask

M = music(); F = foley(); V, mask = voice()
for _c in range(2): M[:, _c] = (M[:, _c] - .35 * lp(M[:, _c], 110)).astype(np.float32)   # tame the low end (synth bass + timpani)
from scipy.ndimage import uniform_filter1d
duck = 1 - (1 - db(-4)) * np.clip(uniform_filter1d(mask, int(.25 * SR)), 0, 1)
M *= duck[:, None]
on = mask > 0
rms = lambda a: float(np.sqrt(np.mean(a[on] ** 2)) + 1e-12)
gv = db(10) * rms(M + F) / rms(V)
mix = M + F + V * gv
for ch in range(2): mix[:, ch] = limit(mix[:, ch], .89)
sf.write(os.path.join(WORK, f'mix-{LANG}.wav'), mix, SR, subtype='PCM_24')
# band check: low end vs the rest
spec = np.abs(np.fft.rfft(mix[:, 0][::4])) ** 2; fr = np.fft.rfftfreq(len(mix[:, 0][::4]), 4 / SR)
lo = spec[(fr > 20) & (fr < 120)].mean(); mi = spec[(fr > 120) & (fr < 4000)].mean()
print(LANG, 'voice gain', round(20 * np.log10(gv), 1), 'dB; peak', round(float(np.abs(mix).max()), 3), '; low/mid', round(10 * np.log10(lo / mi), 1), 'dB')
