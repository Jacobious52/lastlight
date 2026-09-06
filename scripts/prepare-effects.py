"""Adapt CC0 recordings to the world. No oscillator/noise synthesis.
Sources, original files and licenses are retained in art/audio-source.
"""
from pathlib import Path
import subprocess, zipfile
import numpy as np
ROOT=Path(__file__).resolve().parent.parent
OUT=ROOT/'assets/audio/effects'; OUT.mkdir(parents=True,exist_ok=True)
kenney=zipfile.ZipFile(ROOT/'art/audio-source/foley/kenney-impact-sounds.zip')
ghosts=zipfile.ZipFile(ROOT/'art/audio-source/creatures/qubodup-GhostMoans.zip')
def render(data,name,speed=1.,lp=2600,echo=False,level=.11):
    filters=f'asetrate=44100*{speed},aresample=44100,highpass=f=45,lowpass=f={lp}'
    if echo: filters+=',aecho=0.8:0.7:137|311|593:0.30|0.18|0.08'
    raw=subprocess.run(['ffmpeg','-v','error','-i','pipe:0','-ac','1','-ar','44100','-af',filters,'-f','f32le','pipe:1'],input=data,capture_output=True,check=True).stdout
    a=np.frombuffer(raw,dtype='<f4').copy()
    active=np.flatnonzero(np.abs(a)>.002)
    if len(active):a=a[max(0,active[0]-440):min(len(a),active[-1]+2205)]
    a*=min(.5/max(np.abs(a).max(),.001),level/max(np.sqrt(np.mean(a*a)),.001))
    fade=min(660,len(a)//6);a[:fade]*=np.linspace(0,1,fade);a[-fade:]*=np.linspace(1,0,fade)
    subprocess.run(['ffmpeg','-v','error','-y','-f','f32le','-ar','44100','-ac','1','-i','pipe:0','-c:a','libvorbis','-q:a','4',str(OUT/f'{name}.ogg')],input=a.astype('<f4').tobytes(),check=True)
    print(name,round(len(a)/44100,2))
def metal(file):return kenney.read('Audio/'+file+'.ogg')
for name,file,speed in [
 ('pulse','impactPlate_light_002',.58),('anchor','impactGlass_light_001',.75),
 ('ability','impactBell_heavy_000',.55),('resonator','impactBell_heavy_001',.38),
 ('secret','impactGlass_medium_003',.65),('death','impactSoft_heavy_002',.6),
 ('bell','impactBell_heavy_000',.95),('wake','impactPlate_heavy_000',.42),('ending','impactBell_heavy_001',.32),
 ('begin','impactWood_light_001',.8)]:
 render(metal(file),name,speed,2200,True,.10)
render(metal('impactPunch_heavy_002'),'hurt',.78,3600,False,.20)
render(metal('impactMetal_heavy_002'),'distress',.55,1700,False,.12)
listener=(ROOT/'art/audio-source/creatures/silent_beast_growl.mp3').read_bytes()
large=ghosts.read('qubodup-GhostMoans/wav/qubodup-GhostMoan02.wav')
still=ghosts.read('qubodup-GhostMoans/wav/qubodup-GhostMoan05.wav')
grazer=(ROOT/'art/audio-source/creatures/silent_beast_growl_2.mp3').read_bytes()
for i,(data,speed) in enumerate([(listener,.86),(still,.65),(grazer,1.2),(large,.55)]):
 for muffled in [False,True]:render(data,f'creature-{i+4*int(muffled)}',speed,650 if muffled else 2300,True,.085)
# More urgent version of the same inhalation identifies detection without a click.
render(listener,'warning',.95,3200,False,.14)
