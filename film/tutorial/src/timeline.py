"""One timeline for picture, voice, captions and sound: writes timeline.json next to this file.
Each narration take (voice/<g>-<lang>.wav) is placed whole at its film time AT; captions come from the
faster-whisper word timings (voice/words.json). g5b is a retake whose last sentence alone is used.
usage: python timeline.py [lang]   (default en)"""
import json, os, re, sys, unicodedata
HERE = os.path.dirname(os.path.abspath(__file__))
LANG = sys.argv[1] if len(sys.argv) > 1 else 'en'
VO = os.path.join(HERE, '..', 'voice')
W = json.load(open(os.path.join(VO, 'words.json'), encoding='utf-8'))
DUR = 141.5

# film time of each take (the picture's chapters are built around these)
AT = {'en': {'g1': 1.0, 'g2': 14.0, 'g3': 24.6, 'g4': 38.3, 'g5a': 56.5, 'g5b': 79.6, 'g6': 87.6, 'g7': 117.0, 'g8': 129.0},
      'es': {'g1': 0.6, 'g2': 14.6, 'g3': 25.0, 'g4': 38.3, 'g5a': 56.3, 'g5b': 80.4, 'g6': 88.9, 'g7': 117.0, 'g8': 128.4}}[LANG]
# Spanish runs longer: some takes are tightened with atempo (pitch kept) to fit the picture's chapters
TEMPO = {'en': {}, 'es': {'g1': 1.1, 'g5a': 1.08, 'g6': 1.05, 'g7': 1.05, 'g8': 1.08}}[LANG]
# captions: (text, first word, last word); a word may be (word, n) = its n-th match; None = the next word
CAPS_EN = {
 'g1': [("Every year, pests destroy up to 40% of the world's crops.", 'every', 'crops'),
        ("And as the climate warms, many of them breed faster.", 'and', 'faster'),
        ("So farmers spray more often,", 'so', 'often'),
        ("pay more, and their soil grows poorer.", 'pay', 'poorer')],
 'g2': [("Field Shift turns NASA's view of the Earth", 'field', 'earth'),
        ("into advice for one field.", 'into', 'field'),
        ("Here is how it works, on a pea field", 'here', 'field'),
        ("in the Huaral valley, in Peru.", None, 'peru')],
 'g3': [("Start with your field. Use your GPS, search a place, or open the example site.", 'start', 'site'),
        ("Drop a point, or draw the outline.", 'drop', 'outline'),
        ("Field Shift measures the area for you.", 'field', 'you')],
 'g4': [("Then tell it what you would tell your agronomist.", 'then', 'agronomist'),
        ("What grows now, what grew before, how you water.", 'what', 'water'),
        ("If you don't know, say so.", 'if', 'so'),
        ("Send photos at full resolution, or simply talk.", 'send', 'talk'),
        ("It asks for what is missing, like a closer look at the underside of a leaf.", 'it', 'leaf')],
 'g5a': [("Meanwhile, NASA data reads your field.", 'meanwhile', 'field'),
         ("40 years of NASA POWER weather show these weeks", 'forty', 'weeks'),
         ("running 5 °C warmer than normal.", 'running', 'normal'),
         ("NASA's SMAP satellite measures soil moisture.", 'nasas', 'moisture'),
         ("Sentinel-2 maps crop vigor, 10 m per pixel.", 'sentinel', 'pixel'),
         ("Every reading shows its source, its date,", 'every', 'date'),
         ("and whether it describes your field or the whole region.", 'and', 'region')],
 'g5b': [("From the temperature, degree-days count pest generations:", 'from', 'generations'),
         ("aphids here complete about 3 more generations than normal.", None, 'normal')],
 'g6': [("Then the readings become a plan.", 'then', 'plan'),
        ("Prevention ranks the pests likely on your field,", 'prevention', 'field'),
        ("with NASA's GEOS-FP weather forecast.", 'with', 'forecast'),
        ("Organic care names the natural enemies to protect,", 'organic', 'protect'),
        ("like lady beetles and parasitoid wasps,", 'like', 'wasps'),
        ("and beneficial fungi such as Trichoderma.", 'and', 'trichoderma'),
        ("Rotation compares your current rotation with two alternatives,", 'rotation', 'alternatives'),
        ("for soil health, water and pest cycles.", 'for', 'cycles'),
        ("And you can ask: what if I grow avocado?", 'and', 'avocado'),
        ("It is scored the same way.", 'it', 'way')],
 'g7': [("It all ends in a pre-diagnosis, not a diagnosis.", 'it', ('diagnosis', 2)),
        ("Download the technical sheet, with every number cited,", 'download', 'cited'),
        ("and confirm it with your agronomist.", 'and', 'agronomist')],
 'g8': [("NASA watches the whole planet.", 'nasa', 'planet'),
        ("Field Shift helps you read your own field.", 'field', 'field')],
}

CAPS_ES = {
 'g1': [("Cada año, las plagas destruyen hasta el 40% de los cultivos del mundo.", 'cada', 'mundo'),
        ("Y con un clima más cálido, muchas se reproducen más rápido.", 'y', 'rapido'),
        ("Así, los agricultores fumigan más seguido,", 'asi', 'seguido'),
        ("gastan más, y su suelo se empobrece.", 'gastan', 'empobrece')],
 'g2': [("Con Field Shift, la mirada de la NASA sobre la Tierra", 'con', 'tierra'),
        ("se convierte en consejos para un campo.", 'se', 'campo'),
        ("Así funciona, en un campo de arveja", 'asi', 'arveja'),
        ("del valle de Huaral, en Perú.", 'del', 'peru')],
 'g3': [("Empieza por tu campo. Usa tu GPS, busca un lugar, o abre el sitio de ejemplo.", 'empieza', 'ejemplo'),
        ("Marca un punto, o dibuja el contorno.", 'marca', 'contorno'),
        ("Field Shift mide el área por ti.", None, 'ti')],
 'g4': [("Luego, cuéntale lo que le dirías a tu agrónomo.", 'luego', 'agronomo'),
        ("Qué siembras ahora, qué sembraste antes, cómo riegas.", 'que', 'riegas'),
        ("Si no sabes, dilo.", 'si', 'dilo'),
        ("Envía fotos a resolución completa, o simplemente habla.", 'envia', 'habla'),
        ("Te pide lo que falta, como ver de cerca el envés de una hoja.", 'te', 'hoja')],
 'g5a': [("Mientras tanto, los datos de la NASA leen tu campo.", 'mientras', 'campo'),
         ("40 años de clima de NASA POWER muestran estas semanas", '40', 'semanas'),
         ("5 °C más cálidas de lo normal.", '5', 'normal'),
         ("El satélite SMAP de la NASA mide la humedad del suelo.", 'el', 'suelo'),
         ("Sentinel-2 mapea el vigor del cultivo, 10 m por píxel.", 'sentinel', 'pixel'),
         ("Cada lectura muestra su fuente, su fecha,", 'cada', 'fecha'),
         ("y si describe tu campo o toda la región.", 'y', 'region')],
 'g5b': [("Con la temperatura, los grados-día cuentan generaciones de plagas:", 'con', 'plagas'),
         ("aquí los pulgones completan unas 3 generaciones más de lo normal.", 'aqui', 'normal')],
 'g6': [("Luego, las lecturas se vuelven un plan.", 'luego', 'plan'),
        ("Prevención ordena las plagas probables en tu campo,", 'prevencion', 'campo'),
        ("con el pronóstico GEOS-FP de la NASA.", 'con', 'nasa'),
        ("Control orgánico nombra a los enemigos naturales que debes proteger,", None, 'proteger'),
        ("como mariquitas y avispas parasitoides,", 'como', 'parasitoides'),
        ("y hongos benéficos como Trichoderma.", 'y', 'derma'),
        ("Rotación compara tu rotación actual con dos alternativas,", 'rotacion', 'alternativas'),
        ("por salud del suelo, agua y ciclos de plagas.", 'por', 'plagas'),
        ("Y puedes preguntar: ¿y si siembro palta?", 'y', 'falta'),
        ("Se evalúa igual.", 'se', 'igual')],
 'g7': [("Todo termina en un prediagnóstico, no un diagnóstico.", 'todo', 'diagnostico'),
        ("Descarga la ficha técnica, con cada número citado,", 'descarga', 'citado'),
        ("y confírmala con tu agrónomo.", 'y', 'agronomo')],
 'g8': [("La NASA observa todo el planeta.", 'la', 'planeta'),
        ("Field Shift te ayuda a leer tu propio campo.", None, 'campo')],
}
CAPS = {'en': CAPS_EN, 'es': CAPS_ES}[LANG]
norm = lambda w: re.sub(r'[^a-z0-9]', '', unicodedata.normalize('NFKD', w.lower()).encode('ascii', 'ignore').decode())

def find(words, w, start):
    n = 1
    if isinstance(w, tuple): w, n = w
    for k in range(start, len(words)):
        if norm(words[k][0]) == w:
            n -= 1
            if n == 0: return k
    raise SystemExit(f'word {w!r} not found after {start}: ' + ' '.join(x[0] for x in words[start:start + 12]))

clips, cues, lines = [], [], {}
for g, caps in CAPS.items():
    words = W[f'{g}-{LANG}']; pos = 0; first_used = None
    for i, (text, w0, w1) in enumerate(caps):
        a = pos if w0 is None else find(words, w0, pos)
        b = find(words, w1, a + 1)
        pos = b + 1
        s0, s1 = max(0.0, words[a][1]), words[b][2]
        if first_used is None: first_used = s0
        cues.append({'id': f'{g}.{i}', 'w0': s0, 'w1': s1, 'g': g, 'text': text})
    # the take is placed whole, except g5b (its last sentence only)
    clip_in = max(0.0, first_used - 0.12) if g == 'g5b' else 0.0
    k = TEMPO.get(g, 1.0); ft = lambda w: AT[g] + (w - clip_in) / k
    clips.append({'g': g, 'file': f'{g}-{LANG}.wav', 'in': round(clip_in, 3), 'at': round(AT[g], 3), 'tempo': k})
    for c in cues:
        if c['g'] == g and 't0' not in c:
            c['t0'] = round(ft(c['w0']) - 0.05, 2); c['speech_end'] = round(ft(c['w1']), 2)
            lines[c['id']] = [round(ft(c['w0']), 2), round(ft(c['w1']), 2)]
cues.sort(key=lambda c: c['t0'])
warn = []
for i, c in enumerate(cues):
    end = c['speech_end'] + 0.45
    nxt = cues[i + 1]['t0'] - 0.05 if i + 1 < len(cues) else DUR
    c['t1'] = round(min(max(end, c['t0'] + 1.8), nxt), 2)
    if c['t1'] - c['t0'] < 1.8 or c['t1'] < c['speech_end']: warn.append(c['id'])
    if i + 1 < len(cues) and cues[i + 1]['t0'] < c['speech_end']: warn.append('overlap ' + c['id'])
out = {'dur': DUR, 'fps': 30, 'lang': LANG, 'clips': clips, 'lines': lines,
       'cues': [{k: c[k] for k in ('id', 't0', 't1', 'speech_end', 'text')} for c in cues]}
json.dump(out, open(os.path.join(HERE, f'timeline-{LANG}.json'), 'w', encoding='utf-8'), ensure_ascii=False, indent=1)
for c in cues: print(c['id'], c['t0'], c['speech_end'], c['t1'], c['text'])
print('warnings:', warn or 'none')
