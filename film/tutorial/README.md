# film/tutorial

How-to film (~2:20) for fieldshift.earth and YouTube. Same grammar as the 30 s film: Satellite Briefing opening, Mascot Screencast walkthrough with the cursor as the farmer and the reticle as the software.

| Path | What it is |
|------|------------|
| `field-shift-tutorial-en.mp4` | English film, burned-in captions, −14 LUFS. Not in git (72 MB): on YouTube, https://youtu.be/siLFiw89jxk |
| `field-shift-tutorial-en.srt` | English captions as text |
| `field-shift-tutorial-es.mp4`, `.srt` | Spanish film (not in git: https://youtu.be/MyDycfiTa2w) (Spanish app screens, Spanish narration, some takes tightened with atempo) and captions |
| `poster.jpg`, `poster-es.jpg` | Poster frames (end card) per language |
| `YOUTUBE.md` | Titles, descriptions with chapters, and settings for the two uploads |
| `TREATMENT.md` | Structures, shot list, narration, cue map, sound |
| `FACTS.md` | Every claim and number with its source |
| `CREDITS.md` | Every asset with its source and licence |
| `build.sh` | One command from material to deliverables (`sh build.sh en`) |
| `voice/` | Narration takes (Gemini TTS, AI audio), script, Whisper check and word timings |
| `src/capture.mjs` | Screenshots + element boxes of the live app (`en` → `work/app`, `es` → `work/app-es`) |
| `src/globe-world.mjs` | Opening on the product's globe: world tour, orbit over Huaral, dive (per language) |
| `src/globe-frames.mjs` | Climb frames (and the earlier orbit schedule) |
| `src/opening-data.json` | Real NASA POWER months and aphid generations drawn in the opening |
| `src/timeline.py` | Takes placement and caption cues → `timeline-<lang>.json` |
| `src/comp.html` | The film page: camera, cursor, reticle, labels, captions |
| `src/mix.py` | Score, foley, narration mix |
| `work/` | Rendered frames and intermediates (git-ignored) |
| `.gitignore` | Keeps the two rendered tutorial films out of git |

Build: `sh build.sh en` or `sh build.sh es` (narration takes in `voice/` are made beforehand).
