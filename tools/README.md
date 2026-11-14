# tools

One-off data tools. Python environment: `python -m venv tools/.venv` then `tools/.venv/Scripts/python.exe -m pip install h5py numpy` (the venv is git-ignored).

| File | What it does |
|------|--------------|
| `grace_point.py` | Extracts the GRACE/GRACE-FO mascon series (cm water equivalent) at a lat/lon from the JPL CRI v4 netCDF into JSON |
| `nex_gddp_point.py` | NEX-GDDP-CMIP6 (NCCS THREDDS, point NCSS requests, standard library only) ensemble change in temperature and rain ~2040 vs 1995-2014 at a lat/lon into JSON, with each model's license |
| `gemini_tts.py` | Narration with Gemini text-to-speech (voice Algenib by default) into a WAV; tone via inline tags like `[calm] [slow]` |
