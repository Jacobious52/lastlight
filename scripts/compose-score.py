#!/usr/bin/env python3
"""Reproducible lyric-free electroacoustic score. Requires NumPy and FFmpeg.

No samples or external music: modal strings, detuned struck metal, filtered
friction, and an asymmetric multi-tap room. Outputs small Vorbis game assets.
"""
from pathlib import Path
import subprocess
import tempfile
import wave
import numpy as np

SR = 22050
ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'assets/audio'
OUT.mkdir(parents=True, exist_ok=True)


def band_noise(rng, size, center, width):
    bins = np.fft.rfftfreq(size, 1/SR)
    spectrum = np.fft.rfft(rng.normal(0, 1, size))
    spectrum *= np.exp(-0.5 * ((bins - center) / width)**2)
    signal = np.fft.irfft(spectrum, n=size)
    return signal / max(np.std(signal), 1e-9)


def compose(name, length, seed, refuge=False):
    rng = np.random.default_rng(seed)
    size = int(length * SR)
    t = np.arange(size) / SR
    dry = np.zeros((size, 2))

    def place(signal, when, pan=0):
        start = int(when * SR)
        count = min(len(signal), size-start)
        if count <= 0:
            return
        dry[start:start+count, 0] += signal[:count] * np.sqrt((1-pan)/2)
        dry[start:start+count, 1] += signal[:count] * np.sqrt((1+pan)/2)

    def bow(hz, duration, onset, gain, pan):
        n = int(duration*SR)
        age = np.arange(n)/SR
        envelope = np.sin(np.pi*age/duration)**2.5
        flutter = .003 * np.sin(age*.91) + .0018*np.sin(age*3.17)
        signal = np.zeros(n)
        for k in range(1, 14):
            ratio = k * (1 + .00028*k*k)
            phase = 2*np.pi*hz*ratio*age + flutter * (k**.6)*24
            signal += np.sin(phase + rng.uniform(-np.pi,np.pi)) / k**1.45
        rasp = band_noise(rng, n, hz*5.7, hz*1.6)
        pressure = .75 + .15*np.sin(age*1.13) + .1*np.sin(age*2.73)
        place((signal*.55 + rasp*.065)*envelope*pressure*gain, onset, pan)

    def strike(hz, onset, gain, pan, decay=8):
        age=np.arange(int(decay*3*SR))/SR
        signal=np.zeros_like(age)
        # Inharmonic, felt-muted partials: recognisable physical excitation,
        # with no perfect oscillator intervals or ascending reward arpeggio.
        for k in range(1, 18):
            ratio=k*np.sqrt(1+.0009*k*k)
            envelope=np.exp(-age*(1+k*.29)/decay)*(1-np.exp(-age*90))
            signal += np.sin(2*np.pi*hz*ratio*age + .013*np.sin(age*3.1))*envelope/(k**1.8)
            signal += np.sin(2*np.pi*hz*ratio*1.0027*age)*envelope/(k**1.8)*.25
        place(signal*gain, onset, pan)

    # The air is nearly inaudible and leaves space for gameplay signatures.
    air=band_noise(rng,size,105,80)*.006
    air*=np.sin(np.pi*t/length)**2 * (.6+.4*np.sin(t*.041)**2)
    place(air,0)
    if refuge:
        bow(65.41, 25, 8, .040, -.35)
        bow(130.1, 20, 14, .026, .4)
        strike(261.1, 18, .042, -.25, 6)
        strike(246.5, 35.7, .030, .55, 8)
        bow(97.3, 27, 39, .038, -.2)
        strike(130.8, 57.3, .048, .1, 9)
    else:
        bow(46.25, 29, 6, .055, -.3)
        bow(94.7, 22, 16, .026, .45)
        strike(185.0, 12.8, .054, -.5, 9)
        strike(174.4, 29.6, .034, .55, 8)
        bow(69.3, 29, 41, .037, .25)
        bow(70.45, 18, 48, .026, -.45)
        strike(369.6, 57.7, .025, .35, 6)
        strike(195.9, 63.9, .038, -.3, 8)
        bow(46.0, 27, 74, .045, .15)
        strike(92.1, 81.5, .048, -.2, 9)
    # Long, diffuse irregular tails, differing between ears.
    wet=dry.copy()
    for k in range(23):
        delay=.13 + k*.107 + rng.uniform(0,.09)
        off=int(delay*SR)
        gain=.16*np.exp(-delay*.95)*(-1 if k%4==0 else 1)
        wet[off:]+=dry[:-off, ::-1] * gain
    wet *= np.minimum(t/4,1)[:,None]*np.minimum((length-t)/6,1)[:,None]
    peak=np.max(np.abs(wet))
    wet *= .38/max(peak,.38)
    with tempfile.TemporaryDirectory() as temp:
        wav=Path(temp)/'score.wav'
        with wave.open(str(wav),'wb') as f:
            f.setnchannels(2);f.setsampwidth(2);f.setframerate(SR)
            f.writeframes((wet*32767).astype('<i2').tobytes())
        output=OUT/f'{name}.ogg'
        subprocess.run(['ffmpeg','-hide_banner','-loglevel','error','-y','-i',str(wav),'-c:a','libvorbis','-q:a','4',str(output)],check=True)
        print(output.name, output.stat().st_size, 'bytes', 'peak',float(np.max(np.abs(wet))))

compose('stone',104,71419)
compose('refuge',76,18451,True)
