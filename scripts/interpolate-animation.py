"""Bake 48-pose cycles using bidirectional motion and explicit boot tracking.

Requires numpy and opencv-python-headless. Input artwork remains unchanged.
RGBA stores forward XY and backward XY, normalized to cell size. A motion of
half a cell maps from the midpoint to either end of the UNORM range.
"""
from pathlib import Path
import cv2
import numpy as np

ROOT = Path(__file__).resolve().parent.parent

BOOTS = np.array([
    [[145,224],[53,222]], [[110,228],[94,204]],
    [[120,228],[79,213]], [[84,228],[134,210]],
    [[50,225],[149,225]], [[94,203],[110,232]],
    [[69,223],[139,225]], [[129,218],[80,232]],
], dtype=np.float32)


def match_boots(flow, index, adjacent):
    # Optical flow alone can match a passing boot to the wrong leg. Explicit
    # anatomical correspondences keep each boot attached through the crossing.
    points, offsets = [], []
    for y in [100, 140, 165]:
        for x in [0, 48, 96, 144, 191]:
            points.append([x,y]); offsets.append([0,0])
    for source, target in zip(BOOTS[index], BOOTS[adjacent]):
        for delta in [[0,0],[-5,0],[5,0],[0,-5],[0,5]]:
            points.append(source+delta); offsets.append(target-source)
    points=np.array(points)/256.
    offsets=np.array(offsets)
    def kernel(a,b):
        d=np.sum((a[:,None,:]-b[None,:,:])**2,axis=2)
        return d*np.log(d+1e-8)
    k=kernel(points,points)+np.eye(len(points))*1e-6
    p=np.column_stack([np.ones(len(points)),points])
    system=np.block([[k,p],[p.T,np.zeros((3,3))]])
    weights=np.linalg.solve(system,np.vstack([offsets,np.zeros((3,2))]))
    h,w=flow.shape[:2]
    y,x=np.mgrid[:h,:w]
    coords=np.column_stack([x.ravel(),y.ravel()])/256.
    guided=(kernel(coords,points)@weights[:-3]+np.column_stack([np.ones(len(coords)),coords])@weights[-3:]).reshape(h,w,2)
    blend=np.clip((y-157)/25.,0,1)[:,:,None]
    return flow*(1-blend)+guided*blend


def splat(frame, flow, blend):
    # Forward warping avoids the duplicate limbs produced by approximately
    # inverting a flow field where two legs occlude each other.
    h,w=frame.shape[:2]
    y,x=np.mgrid[:h,:w]
    tx=x+flow[:,:,0]*w*blend;ty=y+flow[:,:,1]*h*blend
    ix=np.floor(tx).astype(int);iy=np.floor(ty).astype(int)
    fx=tx-ix;fy=ty-iy
    buffer=np.zeros((h*w,4),np.float32)
    values=np.concatenate([frame[:,:,:3]*frame[:,:,3:],frame[:,:,3:]],axis=2)
    for dx,dy,weight in [(0,0,(1-fx)*(1-fy)),(1,0,fx*(1-fy)),(0,1,(1-fx)*fy),(1,1,fx*fy)]:
        xx=ix+dx;yy=iy+dy
        valid=(xx>=0)&(xx<w)&(yy>=0)&(yy<h)
        np.add.at(buffer,yy[valid]*w+xx[valid],values[valid]*weight[valid,None])
    buffer=buffer.reshape(h,w,4)
    buffer[:,:,:3]/=np.maximum(buffer[:,:,3:],0.0001)
    buffer[:,:,3]=np.minimum(buffer[:,:,3],1)
    return buffer


def prepare(name, cycles):
    source = cv2.imread(str(ROOT / 'assets/art' / f'{name}.png'), cv2.IMREAD_UNCHANGED)
    h, w = source.shape[:2]
    ch, cw = h // 2, w // 4
    frames = [source[y*ch:(y+1)*ch, x*cw:(x+1)*cw] for y in range(2) for x in range(4)]
    # Alpha gives optical flow a stable silhouette even in almost-black cloth.
    def feature(frame):
        grey = cv2.cvtColor(frame[:, :, :3], cv2.COLOR_BGR2GRAY).astype(np.float32)
        return np.uint8(np.clip((grey * 0.45 + 140.) * (frame[:, :, 3] / 255.), 0, 255))
    features = [feature(frame) for frame in frames]
    fields = [None] * 8
    raw_fields = [None] * 8
    for cycle in cycles:
        for i, index in enumerate(cycle):
            pair = []
            for adjacent in [cycle[(i+1) % len(cycle)], cycle[(i-1) % len(cycle)]]:
                estimator = cv2.DISOpticalFlow_create(cv2.DISOPTICAL_FLOW_PRESET_MEDIUM)
                estimator.setFinestScale(0)
                flow = estimator.calc(features[index], features[adjacent], None)
                if name == 'traveller':
                    flow = match_boots(flow,index,adjacent)
                flow = cv2.GaussianBlur(flow, (5, 5), 0.8)
                pair.append(flow / np.array([cw, ch], dtype=np.float32))
            fields[index] = np.uint8(np.clip(np.concatenate(pair, axis=2) + 0.5, 0., 1.) * 255. + 0.5)
            raw_fields[index] = pair
    atlas = np.concatenate([np.concatenate(fields[:4], axis=1), np.concatenate(fields[4:], axis=1)], axis=0)
    # OpenCV writes BGRA; the shader expects encoded RGBA channels unchanged.
    (ROOT / 'art/motion-fields').mkdir(exist_ok=True)
    path = ROOT / 'art/motion-fields' / f'{name}-flow.png'
    cv2.imwrite(str(path), cv2.cvtColor(atlas, cv2.COLOR_RGBA2BGRA), [cv2.IMWRITE_PNG_COMPRESSION, 9])
    animated=[]
    for cycle in cycles:
        subdivisions=48//len(cycle)
        for i,index in enumerate(cycle):
            adjacent=cycle[(i+1)%len(cycle)]
            a=frames[index].astype(np.float32)/255.
            b=frames[adjacent].astype(np.float32)/255.
            for sub in range(subdivisions):
                t=sub/subdivisions
                if sub==0:
                    pose=frames[index]
                else:
                    aa=splat(a,raw_fields[index][0],t)
                    bb=splat(b,raw_fields[adjacent][1],1-t)
                    alpha=aa[:,:,3:]*(1-t)+bb[:,:,3:]*t
                    rgb=(aa[:,:,:3]*aa[:,:,3:]*(1-t)+bb[:,:,:3]*bb[:,:,3:]*t)/np.maximum(alpha,.0001)
                    pose=np.uint8(np.clip(np.concatenate([rgb,alpha],axis=2),0,1)*255.+.5)
                if name=='hauler':
                    pose=cv2.resize(pose,(384,384),interpolation=cv2.INTER_AREA)
                animated.append(pose)
    atlas=np.concatenate([np.concatenate(animated[i:i+8],axis=1) for i in range(0,len(animated),8)],axis=0)
    path=ROOT / 'assets/art' / f'{name}-motion.png'
    cv2.imwrite(str(path),atlas,[cv2.IMWRITE_PNG_COMPRESSION,9])
    print(f'{path.name}: {atlas.shape[1]}x{atlas.shape[0]}, {len(cycles)} cycles of 48 poses')


# Traveller assets have a separate full-body pipeline: animate-traveller.py.
prepare('hauler', [list(range(8))])
prepare('ecosystem', [list(range(4)), list(range(4, 8))])
