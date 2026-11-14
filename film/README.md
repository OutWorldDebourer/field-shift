# film

Demo films (Lemo-Opuscar workflow: Satellite Briefing opening + Mascot Screencast app walkthrough, no mascot: the cursor is the farmer, the lock-on reticle is the software).

| Path | What it is |
|------|------------|
| `field-shift-demo-en.mp4` | **Submission film**, English narration and burned-in English subtitles. 1920×1080, 30 fps, H.264 + AAC, ≤ 30 s, −14 LUFS |
| `field-shift-demo-es.mp4` | Same picture, Spanish narration and burned-in Spanish subtitles (for the team) |
| `field-shift-demo-en.srt`, `field-shift-demo-es.srt` | Subtitles as text |
| `loop/hero-loop.mp4`, `loop/hero-loop.webm`, `loop/hero-loop.jpg` | Silent 8.6 s seamless loop of the orbit → field dive (for a page hero), and its still fallback |
| `poster.jpg` | Poster frame (end card over the climb back to orbit) |
| `TREATMENT.md` | Structures considered, shot list, narration, sound, titles |
| `CREDITS.md` | Every asset in the films with its source and licence |
| `FACTS.md` | Every on-screen claim and number with its source |
| `build.sh` | One command from material to deliverables (see its header for what must be running) |
| `src/` | The film's source: `comp.html` (the page that composes every frame), `globe-frames.mjs` (deterministic frames of the product's globe), `capture-app.mjs` (screenshots of the real app), `timeline.py` (one timeline for picture, voice and captions), `mix.py` (score, foley, narration), `whisper_check.py` |
| `voice/vo-en.wav`, `voice/vo-es.wav` | Narration, Gemini TTS voice Algenib (AI-generated audio) |
| `voice/words.json`, `voice/whisper-check.txt` | faster-whisper transcripts and word timings used to verify and cut |

Both films carry the "SI-assisted visuals" mark (the chat shows the app's AI-assisted demo photos) and acknowledge the AI narration on screen and in the file metadata. `work/` holds rendered frames and intermediate files; it is git-ignored.
