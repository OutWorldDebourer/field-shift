"""Transcribe a finished film's soundtrack with faster-whisper (word timings), to check the narration survives the mix."""
import sys
from faster_whisper import WhisperModel
path, lang = sys.argv[1], sys.argv[2]
m = WhisperModel('small', device='cpu', compute_type='int8')
segs, _ = m.transcribe(path, language=lang, word_timestamps=True, vad_filter=False)
for s in segs:
    print(f'{s.start:6.2f}-{s.end:6.2f} {s.text.strip()}')
