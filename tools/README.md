# tools

One-off data tools. Python environment: `python -m venv tools/.venv` then `tools/.venv/Scripts/python.exe -m pip install h5py numpy` (the venv is git-ignored).

| File | What it does |
|------|--------------|
| `grace_point.py` | Extracts the GRACE/GRACE-FO mascon series (cm water equivalent) at a lat/lon from the JPL CRI v4 netCDF into JSON |
| `gemini_tts.py` | Narration with Gemini text-to-speech (voice Algenib by default) into a WAV; tone via inline tags like `[calm] [slow]` |
