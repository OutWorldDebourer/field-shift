#!/bin/sh
# Field Shift demo films: one command from material to deliverables (Git Bash on Windows, or any POSIX sh).
# Needs: the backend on http://127.0.0.1:8787 serving frontend/dist (run `npx vite build` in frontend/ first),
# Node 20+, ffmpeg 7+, and the Lemo-Opuscar library (LIB) with its .venv (core + voice tiers) and sample packs.
#   sh film/build.sh            everything
#   SKIP_GLOBE=1 sh film/build.sh   reuse the rendered globe frames in film/work/
set -e
LIB=${LIB:-C:/Users/OutWorldDevourer/lemo-opuscar}
export LIB
HERE=$(cd "$(dirname "$0")" && pwd)
W="$HERE/work"; PY="$LIB/.venv/Scripts/python.exe"; [ -x "$PY" ] || PY="$LIB/.venv/bin/python"
mkdir -p "$W/out" "$W/fonts" "$HERE/loop"

# 1. the globe: the product's own dive, rendered frame by frame from frontend/globe.html on a private Vite port
if [ -z "$SKIP_GLOBE" ]; then
  (cd "$HERE/../frontend" && npx vite --port 5190 --strictPort >"$W/vite.log" 2>&1 & echo $! >"$W/vite.pid")
  sleep 6
  (cd "$HERE/src" && node globe-frames.mjs "$W/globe-dive" dive && node globe-frames.mjs "$W/globe-climb" climb) || { kill "$(cat "$W/vite.pid")" 2>/dev/null; exit 1; }
  kill "$(cat "$W/vite.pid")" 2>/dev/null || true
fi

# 2. the real app: screenshots and measured element boxes
(cd "$HERE/src" && node capture-app.mjs "$W/app")

# 3. one timeline; the page; events for the mixer
"$PY" "$HERE/src/timeline.py" >/dev/null
cp "$HERE/src/comp.html" "$W/index.html"; cp "$HERE/src/timeline.json" "$W/"; cp "$HERE/../frontend/public/fonts/"*.woff2 "$W/fonts/"
cd "$LIB"
node core/render/events.mjs "$W"

for L in en es; do
  # 4. sound and picture
  "$PY" "$HERE/src/mix.py" "$W" $L
  node core/render/video.mjs "$W" --fps 30 --workers 4 --q "lang=$L" --out "$W/out/video-$L.mp4"
  # 5. master (two-pass loudnorm -14 LUFS, true peak <= -1 dBTP), then a small two-pass H.264 with the metadata
  sh core/render/mux.sh "$W/out/video-$L.mp4" "$W/mix-$L.wav" "$W/out/master-$L.mp4" 30 0
  if [ $L = en ]; then T="Field Shift: demo film (English)"; else T="Field Shift: película demo (español)"; fi
  C="Field Shift · Team InnoHuando · NASA Space Apps Challenge 2026. Narration: AI voice, Gemini TTS. SI-assisted visuals. Imagery: NASA Blue Marble, VIIRS (NASA GIBS); contains modified Copernicus Sentinel data 2026. Music: original, CC0 samples."
  ( cd "$W/out"
    ffmpeg -y -loglevel error -i "video-$L.mp4" -c:v libx264 -preset slow -b:v 3400k -maxrate 5000k -bufsize 8000k -pix_fmt yuv420p -pass 1 -passlogfile "x264-$L" -an -f null -
    ffmpeg -y -loglevel error -i "video-$L.mp4" -i "master-$L.mp4" -map 0:v -map 1:a -c:v libx264 -preset slow -b:v 3400k -maxrate 5000k -bufsize 8000k -pix_fmt yuv420p \
      -pass 2 -passlogfile "x264-$L" -c:a copy -movflags +faststart -metadata title="$T" -metadata comment="$C" -metadata artist="Team InnoHuando" \
      -metadata:s:a:0 language=$( [ $L = en ] && echo eng || echo spa ) "$HERE/field-shift-demo-$L.mp4" )
done

# 6. subtitles (.srt) from the same cues
"$PY" -c "import json,sys; tl=json.load(open(sys.argv[1],encoding='utf-8')); [json.dump([{k:c[k] for k in ('t0','t1','text')} for c in tl['cues'][l]], open(sys.argv[2]+'/cues-'+l+'.json','w',encoding='utf-8'), ensure_ascii=False) for l in ('en','es')]" "$HERE/src/timeline.json" "$W"
for L in en es; do "$PY" core/render/srt.py "$W/cues-$L.json" "$HERE/field-shift-demo-$L.srt"; done

# 7. silent hero loop: orbit -> dive, the last second dissolving back into the first (8.6 s)
F="[0:v]split=2[a][b];[a]trim=start=1.0:end=9.6,setpts=PTS-STARTPTS[m];[b]trim=start=0:end=1.0,setpts=PTS-STARTPTS[h];[m][h]xfade=transition=fade:duration=1.0:offset=7.6,format=yuv420p[v]"
( cd "$W/out"
  ffmpeg -y -loglevel error -framerate 30 -i "$W/globe-dive/f%04d.jpg" -filter_complex "$F" -map "[v]" -c:v libx264 -preset slow -b:v 2800k -maxrate 3600k -bufsize 6000k -pass 1 -passlogfile loop -an -f null -
  ffmpeg -y -loglevel error -framerate 30 -i "$W/globe-dive/f%04d.jpg" -filter_complex "$F" -map "[v]" -c:v libx264 -preset slow -b:v 2800k -maxrate 3600k -bufsize 6000k -pass 2 -passlogfile loop -movflags +faststart -an \
    -metadata title="Field Shift hero loop" -metadata comment="Imagery: NASA Blue Marble, VIIRS (NASA GIBS); contains modified Copernicus Sentinel data 2026" "$HERE/loop/hero-loop.mp4"
  ffmpeg -y -loglevel error -framerate 30 -i "$W/globe-dive/f%04d.jpg" -filter_complex "$F" -map "[v]" -c:v libvpx-vp9 -b:v 2600k -row-mt 1 -pass 1 -passlogfile loopvp9 -an -f null -
  ffmpeg -y -loglevel error -framerate 30 -i "$W/globe-dive/f%04d.jpg" -filter_complex "$F" -map "[v]" -c:v libvpx-vp9 -b:v 2600k -row-mt 1 -pass 2 -passlogfile loopvp9 -an -metadata title="Field Shift hero loop" "$HERE/loop/hero-loop.webm" )
ffmpeg -y -loglevel error -i "$W/globe-dive/f0030.jpg" -q:v 3 "$HERE/loop/hero-loop.jpg"
ffmpeg -y -loglevel error -ss 27.8 -i "$HERE/field-shift-demo-en.mp4" -frames:v 1 -q:v 3 "$HERE/poster.jpg"
echo "done: $HERE"
