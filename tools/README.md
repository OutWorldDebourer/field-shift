# tools

One-off data tools. Python environment: `python -m venv tools/.venv` then `tools/.venv/Scripts/python.exe -m pip install h5py numpy` (the venv is git-ignored).

| File | What it does |
|------|--------------|
| `grace_point.py` | Extracts the GRACE/GRACE-FO mascon series (cm water equivalent) at a lat/lon from the JPL CRI v4 netCDF into JSON |
| `nex_gddp_point.py` | NEX-GDDP-CMIP6 (NCCS THREDDS, point NCSS requests, standard library only) ensemble change in temperature and rain ~2040 vs 1995-2014 at a lat/lon into JSON, with each model's license |
| `gemini_tts.py` | Narration with Gemini text-to-speech (voice Algenib by default) into a WAV; tone via inline tags like `[calm] [slow]` |
| `phones.mjs` | Phone check: index, field picker, the five workspace views and the technical sheet on iPhone 15 / iPhone SE (WebKit, Safari's engine) and Pixel 7 / Galaxy A55 (Chromium); full-page screenshots, horizontal-overflow and console-error report. `node tools/phones.mjs <base-url> <out-dir> [analysis-id] [lang]` |
| `phone-scroll.mjs` | Screen-by-screen capture while scrolling, as a person sees it on a phone. `node tools/phone-scroll.mjs <url> <out-prefix> "<device>" [screens]` |
| `overflow.mjs` | Lists the elements that really make a page wider than a phone (ignoring those clipped by an ancestor). `node tools/overflow.mjs <url> "<device>"` |
| `contact-sheet.py` | Lays screenshots side by side for review. `python tools/contact-sheet.py <out.jpg> <images…>` |
| `row-overlap.mjs` | Checks the compact data rows (value, sparkline, label) for overlaps on a phone, also with enlarged text like Android font scaling; measures the text itself, not only its box. `node tools/row-overlap.mjs <url> "<device>" [text scale]` |
