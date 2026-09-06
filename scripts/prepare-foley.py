"""Prepare downloaded CC0 recordings without sharp synthetic impacts.

Requires NumPy, ffmpeg, and bsdtar (included on macOS). Source archives stay
untouched. Re-run after downloading the archives documented in assets/CREDITS.md.
"""
from pathlib import Path
import subprocess
import zipfile
import numpy as np

ROOT = Path(__file__).resolve().parent.parent
sources = ROOT / 'art/audio-source/foley'
out = ROOT / 'assets/audio/foley'
out.mkdir(parents=True, exist_ok=True)
subprocess.run(['bsdtar', '-xf', str(sources / 'Fantozzi-footsteps.7z'), '-C', str(sources)], check=True)


def prepare(data, target, peak=0.18):
    decoded = subprocess.run(['ffmpeg', '-v', 'error', '-i', 'pipe:0', '-ac', '1', '-ar', '44100',
                              '-af', 'highpass=f=95,lowpass=f=2400,acompressor=threshold=0.10:ratio=3:attack=12:release=90',
                              '-f', 'f32le', 'pipe:1'], input=data, capture_output=True, check=True).stdout
    pcm = np.frombuffer(decoded, dtype='<f4').copy()
    audible = np.flatnonzero(np.abs(pcm) > 0.003)
    if len(audible):
        pcm = pcm[max(0, audible[0]-220):min(len(pcm), audible[-1]+700)]
    pcm *= min(peak / max(float(np.abs(pcm).max()), 0.001), 0.038 / max(float(np.sqrt(np.mean(pcm**2))), 0.001))
    fade = min(220, len(pcm)//4)
    pcm[:fade] *= np.linspace(0, 1, fade)
    pcm[-fade:] *= np.linspace(1, 0, fade)
    subprocess.run(['ffmpeg', '-v', 'error', '-y', '-f', 'f32le', '-ar', '44100', '-ac', '1',
                    '-i', 'pipe:0', '-c:a', 'libvorbis', '-q:a', '4', str(out / target)],
                   input=pcm.astype('<f4').tobytes(), check=True)
    print(f'{target}: {len(pcm)/44100:.2f}s; peak {20*np.log10(np.abs(pcm).max()):.1f} dBFS')


for surface in ['Stone', 'Sand']:
    for take in range(3):
        for foot in ['L', 'R']:
            src = sources / 'Fantozzi-footsteps/flac' / f'Fantozzi-{surface}{foot}{take+1}.flac'
            index = take * 2 + (foot == 'R')
            prepare(src.read_bytes(), f'{surface.lower()}-{index}.ogg')
with zipfile.ZipFile(sources / 'kenney-impact-sounds.zip') as archive:
    prepare(archive.read('Audio/footstep_carpet_002.ogg'), 'scuff.ogg', peak=0.12)
