"""Prepare complete painted traveller poses and bake the browser animation.

Run scripts/setup-animation.sh first. No runtime model or skeletal deformation.
Source sheets and prompts are in art/source/traveller-v2. The RGB generator
returned a light checkerboard despite the alpha request; remove that matte
before resizing, preserving the enclosed bright lantern glass.
"""
from pathlib import Path
import sys
import subprocess
import torch
import cv2
import numpy as np
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / 'art/source/traveller-v2'
OUTPUT = ROOT / 'assets/art'
W, H, FRAMES = 192, 256, 48
Y, X = np.mgrid[:H,:W].astype(np.float32)

RIFE = ROOT / '.tools/rife/upstream'
sys.path.insert(0, str(RIFE))
from train_log.IFNet_HDv3 import IFNet
NET = IFNet().eval()
weights = torch.load(RIFE / 'train_log/flownet.pkl', map_location='cpu', weights_only=True)
NET.load_state_dict({k.removeprefix('module.'):v for k,v in weights.items()
                    if k.removeprefix('module.') in NET.state_dict()})
torch.set_num_threads(4)


def cutout(rgb):
    if rgb.shape[2] == 4:
        return rgb
    dark = np.uint8(rgb.min(axis=2) < 205)
    count, labels, stats, _ = cv2.connectedComponentsWithStats(dark, 8)
    selected = np.argmax(stats[1:, cv2.CC_STAT_AREA]) + 1
    mask = np.uint8(labels == selected)
    # Fill tiny enclosed highlights, including the actual lamp glass. Keep
    # large gaps between the arm and body, or between legs, transparent.
    n, holes, hs, _ = cv2.connectedComponentsWithStats(1-mask, 8)
    for i in range(1, n):
        region = holes == i
        if hs[i, cv2.CC_STAT_AREA] < 500 and np.mean((rgb[:,:,0].astype(float)-rgb[:,:,2])[region] > 3) > .08:
            mask[region] = 1
    alpha = mask.astype(np.float32)
    boundary = mask-cv2.erode(mask, np.ones((3,3),np.uint8))
    alpha[boundary > 0] = np.clip((240-rgb.min(axis=2)[boundary>0])/100, 0, 1)
    clean = np.clip((rgb.astype(float)-240*(1-alpha[:,:,None])) / np.maximum(alpha[:,:,None], .01), 0, 255)
    return np.uint8(np.dstack([clean, alpha*255]))


def sheet(name, columns=4, rows=2):
    src = np.array(Image.open(SOURCE / f'{name}.png'))
    ch, cw = src.shape[0]//rows, src.shape[1]//columns
    result = []
    for row in range(rows):
        for col in range(columns):
            part = cutout(src[row*ch:(row+1)*ch, col*cw:(col+1)*cw])
            ys, xs = np.where(part[:,:,3] > 128)
            part = part[ys.min():ys.max()+1, xs.min():xs.max()+1]
            # Uniform scaling (never independent X/Y stretching). Register the
            # torso, rather than centring the changing silhouette of the coat.
            scale = 216 / part.shape[0]
            torso = part[int(part.shape[0]*.20):int(part.shape[0]*.42),:,3]
            centre = np.average(np.arange(part.shape[1]), weights=torso.sum(axis=0))
            resized = Image.fromarray(part).resize((round(part.shape[1]*scale),216), Image.Resampling.LANCZOS)
            canvas = Image.new('RGBA',(W,H))
            canvas.paste(resized,(round(100-centre*scale),24))
            result.append(np.array(canvas))
    return result


def save_atlas(poses, name, columns=8):
    atlas = np.concatenate([np.concatenate(poses[i:i+columns],axis=1) for i in range(0,len(poses),columns)],axis=0)
    assert max(atlas.shape[:2]) <= 4096, 'Atlas exceeds the WebGL 2 minimum texture limit'
    Image.fromarray(atlas).save(OUTPUT / name, optimize=True)


def contacts(views):
    result = Image.new('RGB',(8*W,3*(H+24)),(65,65,65))
    draw = ImageDraw.Draw(result)
    for row,(name,poses) in enumerate(views.items()):
        for i,pose in enumerate(poses):
            result.paste(Image.fromarray(pose),(i*W,row*(H+24)),Image.fromarray(pose[:,:,3]))
            draw.text((i*W+5,row*(H+24)+H),f'{name} {i}',fill='white')
    result.save(ROOT / '.tools/traveller-keyframes.jpg')


def lamp_centre(pose):
    rgb = pose[:,:,:3].astype(float)
    # Glass is the only bright warm region in these charcoal silhouettes.
    weight = np.maximum(rgb[:,:,0]-rgb[:,:,2]-12,0) * (rgb[:,:,0]>195) * (pose[:,:,3]>180)
    assert weight.sum() > 1, 'Missing lantern glass in animation frame'
    peak_y,peak_x = np.unravel_index(weight.argmax(),weight.shape)
    weight *= (X-peak_x)**2+(Y-peak_y)**2 < 144
    centre = np.array([(weight*X).sum(),(weight*Y).sum()])/weight.sum()
    xx,yy = np.rint(centre).astype(int)
    if pose[yy,xx,3] < 180:
        py,px = np.where(weight > 0)
        near = np.argmin((px-centre[0])**2+(py-centre[1])**2)
        centre = np.array([px[near],py[near]],dtype=float)
    return centre


def between(a, b, amounts):
    def tensor(pose):
        p = pose.astype(np.float32)/255
        # Estimate flow on a neutral white matte, carrying alpha through the
        # exact same learned warps and occlusion mask as the painted pixels.
        alpha = p[:,:,3:]
        color = p[:,:,:3][:,:,::-1]*alpha + (1-alpha)
        return torch.from_numpy(np.concatenate([color,alpha],axis=2).transpose(2,0,1).copy()).unsqueeze(0)
    pair = torch.cat([tensor(a),tensor(b)],1)
    frames = []
    with torch.inference_mode():
        for t in amounts:
            if t == 0:
                frames.append(a.copy()); continue
            if t == 1:
                frames.append(b.copy()); continue
            _,_,merged = NET(pair,float(t),[16,8,4,2,1])
            p = merged[-1][0].clamp(0,1).numpy().transpose(1,2,0)
            alpha = p[:,:,3:]
            color = (p[:,:,:3]-(1-alpha))/np.maximum(alpha,.001)
            # Recover solid cloth/boots from occlusion blending. Only the
            # antialiased outer boundary remains translucent in the atlas.
            coverage = np.clip((alpha-.06)/.30,0,1)
            result = np.concatenate([color[:,:,::-1],coverage],axis=2)
            frames.append(np.uint8(np.clip(result,0,1)*255+.5))
    return frames


def bake_cycle(keys):
    frames = []
    step = FRAMES//len(keys)
    for i,a in enumerate(keys):
        frames += between(a,keys[(i+1)%len(keys)],np.arange(step)/step)
    return frames


def write_lamps(walk, idle):
    # CPU and shader use exactly the same selected pose, including settling.
    for cycle in walk+idle:
        anchors = np.array([lamp_centre(pose) for pose in cycle])
        assert np.linalg.norm(np.diff(anchors,axis=0),axis=1).max() < 6, 'Flame jumps between disconnected painted lamps'
        for pose,lamp in zip(cycle,anchors):
            x,y = np.rint(lamp).astype(int)
            assert pose[y,x,3] >= 180, 'Flame is outside the physical lantern'
            assert all(edge.max()<10 for edge in [pose[:3,:,3],pose[-3:,:,3],pose[:,:3,3],pose[:,-3:,3]]), 'Clipped animation frame'
    lines = ['// Generated by scripts/animate-traveller.py. Pixel coordinates.',
             'pub const WALK_LAMPS: [[[f32; 2]; 48]; 3] = [']
    for cycle in walk:
        lines.append('['+', '.join(f'[{p[0]:.3f}, {p[1]:.3f}]' for p in map(lamp_centre,cycle))+'],')
    lines.append('];\npub const REST_LAMPS: [[[f32; 2]; 24]; 6] = [')
    for cycle in idle:
        lines.append('['+', '.join(f'[{p[0]:.3f}, {p[1]:.3f}]' for p in map(lamp_centre,cycle))+'],')
    lines.append('];')
    (ROOT / 'src/traveller_lamps.rs').write_text('\n'.join(lines)+'\n')


def preview(walk):
    frames = []
    for i in range(FRAMES):
        canvas = Image.new('RGB',(W*4,H),(45,45,45))
        for j,pose in enumerate([walk[0][i],walk[0][i][:,::-1],walk[1][i],walk[2][i]]):
            canvas.paste(Image.fromarray(pose),(j*W,0),Image.fromarray(pose[:,:,3]))
        frames.append(canvas)
    # 48 poses, shown at 24fps for close inspection (half normal walking rate).
    frames[0].save(ROOT / '.tools/traveller-walk.gif',save_all=True,append_images=frames[1:],duration=42,loop=0)


if __name__ == '__main__':
    views = {name:sheet(name) for name in ['side','back','front']}
    rest = sheet('rest',3,1)
    rest[0] = sheet('rest-side',1,1)[0]
    contacts(views)
    # The vertical generated sheets contain redundant contacts. Select the
    # opposing full-body contacts, instead of playing repeated steps in order.
    walk = [bake_cycle(views['side']),
            bake_cycle([views['back'][i] for i in [0,1,4,7]]),
            bake_cycle([views['front'][i] for i in [0,3,1,2]])]
    idle = [between(cycle[contact],rest[view],np.linspace(0,1,24))
            for view,cycle in enumerate(walk) for contact in [0,24]]
    write_lamps(walk,idle)
    save_atlas(walk[0],'traveller-motion.png')
    save_atlas(walk[1]+walk[2],'traveller-vertical-motion.png')
    save_atlas(sum(idle,[]),'traveller-idle.png',12)
    preview(walk)
    subprocess.run(['rustfmt',str(ROOT / 'src/traveller_lamps.rs')],check=True)
    print('Baked 144 complete walking poses and 144 settling poses; measured lantern anchors.')
