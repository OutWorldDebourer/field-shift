"""Synthesize narration with Gemini text-to-speech and save it as a WAV file.

Usage (from the project root):
  python -I tools/gemini_tts.py "[calm] [slow] <text to speak>" <out.wav> [voice] [model]

Defaults: voice Algenib (the owner's choice, a low gravelly voice), model gemini-3.8-flash-tts.
The API key is read from env.txt (line `gemini <key>`) and sent in a header, never printed.
Output is AI-generated audio: the contest rules require acknowledging it in text and metadata.
"""
import base64
import json
import sys
import urllib.error
import urllib.request
import wave
from pathlib import Path


def api_key() -> str:
    for line in Path("env.txt").read_text(encoding="utf-8").splitlines():
        parts = line.split()
        if len(parts) == 2 and parts[0] == "gemini":
            return parts[1]
    sys.exit("no `gemini <key>` line in env.txt")


def synthesize(text: str, out: str, voice: str = "Algenib", model: str = "gemini-3.8-flash-tts") -> None:
    # Tone: with gemini-3.8-flash-tts use inline tags such as "[calm] [slow] text" (tested 2026-11-14).
    # A plain "Say calmly: ..." prefix is read aloud by this model, and system instructions are rejected (HTTP 400).
    body = {
        "contents": [{"parts": [{"text": text}]}],
        "generationConfig": {
            "responseModalities": ["AUDIO"],
            "speechConfig": {"voiceConfig": {"prebuiltVoiceConfig": {"voiceName": voice}}},
        },
    }
    req = urllib.request.Request(
        f"https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent",
        data=json.dumps(body).encode(),
        headers={"x-goog-api-key": api_key(), "Content-Type": "application/json"},
    )
    try:
        with urllib.request.urlopen(req, timeout=120) as resp:
            data = json.load(resp)
    except urllib.error.HTTPError as err:
        sys.exit(f"HTTP {err.code}: {err.read().decode(errors='replace')[:400]}")
    part = data["candidates"][0]["content"]["parts"][0]["inlineData"]
    mime = part["mimeType"]  # e.g. audio/L16;codec=pcm;rate=24000
    rate = int(mime.split("rate=")[1].split(";")[0]) if "rate=" in mime else 24000
    pcm = base64.b64decode(part["data"])
    Path(out).parent.mkdir(parents=True, exist_ok=True)
    with wave.open(out, "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(rate)
        w.writeframes(pcm)
    print(f"{out}: {len(pcm) / 2 / rate:.1f} s, {rate} Hz, voice {voice}, model {model}")


if __name__ == "__main__":
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    synthesize(*sys.argv[1:])
