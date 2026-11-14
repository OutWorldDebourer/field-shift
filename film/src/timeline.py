"""One timeline for picture, voice, captions and sound: writes timeline.json next to this file.
Voice clips are cut from film/voice/vo-*.wav at the word timings in film/voice/words.json (faster-whisper)."""
import json, os
HERE = os.path.dirname(os.path.abspath(__file__))
W = json.load(open(os.path.join(HERE, '..', 'voice', 'words.json'), encoding='utf-8'))
DUR = 29.96
ES_TEMPO = 1.07           # Spanish narration tightened with atempo (pitch kept)
# (first word, last word, film time of the first word, caption text)
LINES = {
  'en': [
    ('This', 'Orbit.', 0.80, 'This is the Huaral valley, from orbit.'),
    ('Field', 'field.', 4.00, "Field Shift dives to one farmer's field."),
    ('The', 'them.', 10.40, 'The farmer sends photos and NASA data checks them.'),
    ('These', 'normal,', 14.10, 'These weeks run 5 °C warmer than the 40-year normal,'),
    ('so', 'faster.', None, 'so aphids breed faster.'),
    ('Three', 'cited.', 20.80, 'Three options, every number cited.'),
    ('A', 'verify.', 23.80, 'A pre-diagnosis your agronomist can verify.'),
  ],
  'es': [
    ('Este', 'órbita.', 0.80, 'Este es el valle de Huaral, desde la órbita.'),
    ('Field', 'agricultor.', 4.00, 'Field Shift baja hasta el campo de un agricultor.'),
    ('El', 'contrastan.', 10.40, 'El agricultor envía fotos y los datos de la NASA las contrastan.'),
    ('Estas', 'años.', 14.10, 'Estas semanas están 5 °C más cálidas que la normal de 40 años.'),
    ('Los', 'rápido.', 18.10, 'Los pulgones se multiplican más rápido.'),
    ('3', 'fuente.', 20.80, 'Tres opciones, cada número con su fuente.'),
    ('Un', 'verificar.', 23.80, 'Un prediagnóstico que tu agrónomo puede verificar.'),
  ],
}
out = {'dur': DUR, 'fps': 30, 'vo': {}, 'cues': {}}
for lang, lines in LINES.items():
    words = W[lang]['words']; tempo = ES_TEMPO if lang == 'es' else 1.0
    used = 0; clips = []; cues = []
    for i, (w0, w1, T, text) in enumerate(lines):
        a = next(k for k in range(used, len(words)) if words[k][0] == w0)
        b = next(k for k in range(a, len(words)) if words[k][0] == w1)
        used = b + 1
        s0, s1 = words[a][1], words[b][2]
        prev_end = words[a - 1][2] if a else 0.0
        nxt = words[b + 1][1] if b + 1 < len(words) else s1 + 0.6
        cin = max(prev_end + 0.02, s0 - 0.10) if a else 0.0
        cout = min(nxt - 0.04, s1 + 0.22)
        if T is None:   # continues the previous clip's sentence: keep the recorded pause
            T = clips[-1]['T'] + (s0 - clips[-1]['s0']) / tempo
            clips[-1]['out'] = cin  # previous clip ends where this one starts
        at = T - (s0 - cin) / tempo
        clips.append({'id': f'{lang}{i+1}', 'in': round(cin, 3), 'out': round(cout, 3), 'at': round(at, 3), 'T': round(T, 3), 's0': s0, 'speech_end': round(T + (s1 - s0) / tempo, 3)})
        cues.append({'t0': round(T - 0.05, 2), 'speech_end': round(T + (s1 - s0) / tempo, 2), 'text': text})
    for i, c in enumerate(cues):
        end = c['speech_end'] + 0.55
        if i + 1 < len(cues): end = min(end, cues[i + 1]['t0'] - 0.05)
        c['t1'] = round(max(end, c['t0'] + 1.8), 2)
    out['vo'][lang] = {'tempo': tempo, 'clips': clips}
    out['cues'][lang] = cues
json.dump(out, open(os.path.join(HERE, 'timeline.json'), 'w', encoding='utf-8'), ensure_ascii=False, indent=1)
for lang in out['cues']:
    for c in out['cues'][lang]: print(lang, c['t0'], c['speech_end'], c['t1'], c['text'])
