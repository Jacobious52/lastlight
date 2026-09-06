"""Pack CC0 Poly Haven scans as isotropic grayscale surface swatches."""
from pathlib import Path
from PIL import Image, ImageOps
import numpy as np
root=Path(__file__).resolve().parent.parent
names=['gravel_floor','forrest_ground_01','embedded_rock_floor','brown_mud_02']
atlas=Image.new('RGB',(2048,2048))
for i,name in enumerate(names):
 im=Image.open(root/'art/texture-source'/f'{name}.jpg').convert('L').resize((1024,1024),Image.Resampling.LANCZOS)
 a=np.array(im,dtype=float);lo,hi=np.percentile(a,[3,97]);a=np.uint8(np.clip((a-lo)/(hi-lo)*148+44,28,214))
 atlas.paste(Image.fromarray(a).convert('RGB'),((i%2)*1024,(i//2)*1024))
atlas.save(root/'assets/art/surfaces.png',optimize=True)
