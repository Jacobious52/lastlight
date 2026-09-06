"""Extract disconnected RGBA sprites, then align ground contacts in GPU atlases.

The image model does not reliably respect grid cell boundaries. Connected-alpha
extraction prevents clipping limbs or including part of the neighbouring prop.
This is mechanical extraction/resampling, not repainting; sources stay intact.
"""
from pathlib import Path
from PIL import Image, ImageChops, ImageFilter
import numpy as np

ROOT = Path(__file__).resolve().parent.parent


def extract(source, count, rows):
    alpha = source.getchannel('A')
    pixels = np.array(alpha) > 64
    height, width = pixels.shape
    labels = np.zeros((height, width), dtype=np.int32)
    components = []
    label = 0
    for y, x in np.argwhere(pixels):
        if not pixels[y, x]:
            continue
        label += 1
        queue = [(int(x), int(y))]
        pixels[y, x] = False
        lo = [int(x), int(y)]
        hi = lo.copy()
        size = 0
        while queue:
            xx, yy = queue.pop()
            labels[yy, xx] = label
            size += 1
            lo[0], lo[1] = min(lo[0], xx), min(lo[1], yy)
            hi[0], hi[1] = max(hi[0], xx), max(hi[1], yy)
            for nx, ny in [(xx-1, yy), (xx+1, yy), (xx, yy-1), (xx, yy+1)]:
                if 0 <= nx < width and 0 <= ny < height and pixels[ny, nx]:
                    pixels[ny, nx] = False
                    queue.append((nx, ny))
        if size > 2000:
            components.append((size, label, (max(0,lo[0]-3),max(0,lo[1]-3),min(width,hi[0]+4),min(height,hi[1]+4))))
    components = sorted(components, reverse=True)[:count]
    assert len(components) == count, 'Missing sprite silhouettes'
    components.sort(key=lambda c: (int(((c[2][1]+c[2][3])*0.5)/height*rows),c[2][0]))
    for _, label, bounds in components:
        mask = Image.fromarray(np.uint8(labels == label)*255).filter(ImageFilter.MaxFilter(7))
        sprite = source.copy()
        sprite.putalpha(ImageChops.multiply(alpha,mask))
        yield sprite.crop(bounds)


def pack(name, columns, rows, cell, fit, baseline=16):
    source = Image.open(ROOT / 'art/source' / (name+'.png')).convert('RGBA')
    atlas = Image.new('RGBA',(columns*cell[0],rows*cell[1]))
    for index, frame in enumerate(extract(source,columns*rows,rows)):
        factor = min(fit[0]/frame.width,fit[1]/frame.height)
        frame = frame.resize((round(frame.width*factor),round(frame.height*factor)),Image.Resampling.LANCZOS)
        x, y = index%columns,index//columns
        atlas.paste(frame,(x*cell[0]+(cell[0]-frame.width)//2,y*cell[1]+cell[1]-baseline-frame.height))
    atlas.save(ROOT / 'assets/art' / (name+'.png'),optimize=True)


if __name__ == "__main__":
    pack('traveller',4,2,(192,256),(144,220))
    pack('traveller-vertical',4,2,(192,256),(144,220))
    pack('hauler',4,2,(512,512),(460,436),baseline=32)
    pack('terrain',2,2,(512,512),(478,456))
    pack('ecosystem',4,2,(256,256),(230,218))
    Image.open(ROOT / 'art/source/ground.png').resize((1024,1024),Image.Resampling.LANCZOS).save(ROOT / 'assets/art/ground.png',optimize=True)

    pack('objects',4,2,(384,384),(344,336),baseline=24)
    pack('pickups',2,1,(384,384),(315,330),baseline=24)
    objects=Image.new('RGBA',(1536,1152))
    objects.paste(Image.open(ROOT/'assets/art/objects.png'),(0,0))
    objects.paste(Image.open(ROOT/'assets/art/pickups.png'),(0,768))
    objects.save(ROOT/'assets/art/objects.png',optimize=True)
