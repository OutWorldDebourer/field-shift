#!/bin/sh
# Field Shift tutorial film: one command from material to deliverables (Git Bash on Windows, or any POSIX sh).
# Needs: Node 20+, ffmpeg 7+, the Lemo-Opuscar library (LIB) with its .venv (core + voice tiers) and sample packs,
# network access to https://fieldshift.earth (app screenshots) and the product's globe on a private Vite port.
#   sh film/tutorial/build.sh [en|es]          everything for one language (default en)
#   SKIP_GLOBE=1 SKIP_CAPTURE=1 sh ...         reuse the rendered globe frames and app screenshots in work/
# The narration takes in voice/ are made beforehand with tools/gemini_tts.py and checked with core/tts/asr_check.py.
set -e
LIB=${LIB:-C:/Users/OutWorldDevourer/lemo-opuscar}
export LIB
L=${1:-en}
HERE=$(cd "$(dirname "$0")" && pwd)
HW=$(cygpath -m "$HERE" 2>/dev/null || echo "$HERE")   # the same folder as Windows Python sees it
W="$HERE/work"; PY="$LIB/.venv/Scripts/python.exe"; [ -x "$PY" ] || PY="$LIB/.venv/bin/python"
mkdir -p "$W/out" "$W/fonts"
"$PY" "$LIB/core/audio/sampler.py" --need "$HERE/src/mix.py" || true

# 1. the globe: orbit hold + dive (new) and the climb (from the 30 s film's work folder)
if [ -z "$SKIP_GLOBE" ]; then
  (cd "$HERE/../../frontend" && npx vite --port 5190 --strictPort >"$W/vite.log" 2>&1 & echo $! >"$W/vite.pid")
  sleep 6
  (cd "$HERE/src" && node globe-world.mjs "$W/globe-world-$L" "$L" && node globe-frames.mjs "$W/globe-climb" climb) || { kill "$(cat "$W/vite.pid")" 2>/dev/null; exit 1; }
  kill "$(cat "$W/vite.pid")" 2>/dev/null || true
fi

# 2. the live app: screenshots and measured element boxes
S=""; [ "$L" = es ] && S="-es"
[ -n "$SKIP_CAPTURE" ] || (cd "$HERE/src" && node capture.mjs "$W/app$S" "$L")

# 3. timeline, page, events
"$PY" "$HERE/src/timeline.py" "$L" >/dev/null
cp "$HERE/src/comp.html" "$W/index.html"; cp "$HERE/src/timeline-$L.json" "$HERE/src/opening-data.json" "$W/"; cp "$HERE/../../frontend/public/fonts/"*.woff2 "$W/fonts/"
cd "$LIB"
node core/render/events.mjs "$W" --q "lang=$L"

# 4. sound and picture
"$PY" "$HERE/src/mix.py" "$W" "$L"
node core/render/video.mjs "$W" --fps 30 --workers 4 --q "lang=$L" --out "$W/out/video-$L.mp4"

# 5. master (two-pass loudnorm -14 LUFS, true peak <= -1 dBTP), then a two-pass H.264 with the metadata
sh core/render/mux.sh "$W/out/video-$L.mp4" "$W/mix-$L.wav" "$W/out/master-$L.mp4" 30 0
if [ "$L" = en ]; then T="Field Shift: how it works (English)"; else T="Field Shift: cómo funciona (español)"; fi
C="Field Shift · Team InnoHuando · NASA Space Apps Challenge 2026. Narration: AI voice, Gemini TTS. SI-assisted visuals. Imagery: NASA Blue Marble, VIIRS (NASA GIBS); contains modified Copernicus Sentinel data 2026. Music: original, CC0 samples."
( cd "$W/out"
  ffmpeg -y -loglevel error -i "master-$L.mp4" -c:v libx264 -preset slow -b:v 4000k -maxrate 6000k -bufsize 9000k -pix_fmt yuv420p -pass 1 -passlogfile "x264-$L" -an -f null -
  ffmpeg -y -loglevel error -i "master-$L.mp4" -c:v libx264 -preset slow -b:v 4000k -maxrate 6000k -bufsize 9000k -pix_fmt yuv420p -pass 2 -passlogfile "x264-$L" \
    -c:a aac -b:a 160k -movflags +faststart -metadata title="$T" -metadata comment="$C" "$HERE/field-shift-tutorial-$L.mp4" )

# 6. subtitles and poster
"$PY" -c "import json;t=json.load(open('$HW/src/timeline-$L.json',encoding='utf-8'));json.dump([{'t0':c['t0'],'t1':c['t1'],'text':c['text']} for c in t['cues']],open('$HW/work/cues-$L.json','w',encoding='utf-8'),ensure_ascii=False)"
"$PY" core/render/srt.py "$HW/work/cues-$L.json" "$HW/field-shift-tutorial-$L.srt"
ffmpeg -y -loglevel error -ss 136 -i "$HERE/field-shift-tutorial-$L.mp4" -frames:v 1 -q:v 3 "$HERE/poster$S.jpg"
echo "done: $HERE/field-shift-tutorial-$L.mp4"
