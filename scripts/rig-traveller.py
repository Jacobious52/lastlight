"""Bake opaque walking and settling poses from rigid painted limb pieces.
No bone changes length and no leg is sheared. The coat is a clean neutral pose,
not fragments left behind by moving a different walking silhouette.
"""
from pathlib import Path
import cv2, numpy as np
ROOT=Path(__file__).resolve().parent.parent
H,W=256,192
y,x=np.mgrid[:H,:W].astype(np.float32)

def over(a,b):
 aa=a[:,:,3:]/255.;ba=b[:,:,3:]/255.;alpha=ba+aa*(1-ba)
 rgb=(b[:,:,:3]*ba+a[:,:,:3]*aa*(1-ba))/np.maximum(alpha,.00001)
 return np.concatenate([rgb,alpha*255],axis=2)

def rigid(part,a,b,c,d):
 # Rotation and translation only: both singular values are exactly one.
 angle=np.arctan2(*(d-c)[::-1])-np.arctan2(*(b-a)[::-1])
 rot=np.array([[np.cos(angle),-np.sin(angle)],[np.sin(angle),np.cos(angle)]])
 mat=np.column_stack([rot,c-rot@a]).astype(np.float32)
 return cv2.warpAffine(part,mat,(W,H),flags=cv2.INTER_LINEAR)

def articulate(part,hip,knee,ankle,foot,target_hip,target_foot,bend=1):
 h,k,a,f,th,tf=map(lambda v:np.array(v,dtype=float),(hip,knee,ankle,foot,target_hip,target_foot))
 ta=tf+(a-f);delta=ta-th;distance=np.linalg.norm(delta)
 l1,l2=np.linalg.norm(k-h),np.linalg.norm(a-k)
 distance=np.clip(distance,abs(l1-l2)+.1,l1+l2-.1)
 axis=delta/max(np.linalg.norm(delta),.001);ta=th+axis*distance;tf=ta+(f-a)
 along=(l1*l1-l2*l2+distance*distance)/(2*distance)
 tk=th+axis*along+np.array([axis[1],-axis[0]])*np.sqrt(max(0,l1*l1-along*along))*bend
 upper=part.copy();upper[y>k[1]+5,3]=0
 lower=part.copy();lower[(y<k[1]-5)|(y>a[1]+4),3]=0
 boot=part.copy();boot[y<a[1]-2,3]=0
 canvas=np.zeros((H,W,4),float)
 # Continuous cloth underpainting closes tiny cutout seams at the bent knee.
 cv2.line(canvas,tuple(np.int32(th)),tuple(np.int32(tk)),(28.,28.,28.,255.),13,cv2.LINE_AA)
 cv2.line(canvas,tuple(np.int32(tk)),tuple(np.int32(ta)),(30.,30.,30.,255.),12,cv2.LINE_AA)
 canvas=over(canvas,rigid(upper,h,k,th,tk))
 canvas=over(canvas,rigid(lower,k,a,tk,ta))
 boot=cv2.warpAffine(boot,np.float32([[1,0,tf[0]-f[0]],[0,1,tf[1]-f[1]]]),(W,H))
 return over(canvas,boot)

side_sheet=cv2.imread(str(ROOT/'assets/art/traveller.png'),-1)
vertical_sheet=cv2.imread(str(ROOT/'assets/art/traveller-vertical.png'),-1)

def cycle(view=0,idle=None):
 if view==0:
  source=side_sheet[:256,192:384].astype(float).copy()
  body=source.copy()
  body[y>=198,3]=0
  # Use the intact neutral boot and shin. The old contact frame contains coat
  # strips between its legs; those must never become moving limb pieces.
  source[(x<82)|(x>130)|(y<188),3]=0
  joints=((99,169),(97,197),(96,220),(107,233))
 else:
  source=vertical_sheet[(view-1)*256:view*256,:192].astype(float)
  body=source.copy();body[y>=194,3]=0
  source[(x<72)|(x>112)|(y<172),3]=0
  joints=((91,173),(94,198),(95,219),(96,232))
 poses=[]
 for i in range(48 if idle is None else 24):
  phase=i/48. if idle is None else idle*.5
  bob=-1.0*np.cos(phase*np.pi*4)
  canvas=np.zeros((H,W,4),float)
  for leg in range(2):
   t=(phase+leg*.5)%1
   stride=23-92*t if t<.5 else -23+46*(.5-.5*np.cos((t-.5)*2*np.pi))
   lift=0 if t<.5 else 13*np.sin((t-.5)*2*np.pi)
   hip=(104.+leg*6.,167.+bob) if view==0 else (92.+leg*24.,174.+bob)
   foot=np.array((108.+stride,233.-lift) if view==0 else (94.+leg*24.,229.-lift))
   if idle is not None:
    rest=np.array((101.+leg*15.,233.) if view==0 else (94.+leg*24.,229.))
    foot=foot*(1-i/23.)+rest*i/23.
   part=source.copy();part[:,:,:3]*=.80 if leg==0 else 1.
   canvas=over(canvas,articulate(part,*joints,hip,foot,bend=1 if view==0 else (-1 if leg==0 else 1)))
  torso=cv2.warpAffine(body,np.float32([[1,0,0],[0,1,bob]]),(W,H))
  poses.append(np.uint8(np.clip(over(canvas,torso),0,255)))
 return poses

def atlas(poses,name):
 image=np.concatenate([np.concatenate(poses[i:i+8],axis=1) for i in range(0,len(poses),8)],axis=0)
 cv2.imwrite(str(ROOT/'assets/art'/name),image,[cv2.IMWRITE_PNG_COMPRESSION,9])
 return image
atlas(cycle(),'traveller-motion.png')
atlas(cycle(1)+cycle(2),'traveller-vertical-motion.png')
poses=cycle(0,0)+cycle(0,1)+cycle(1,0)+cycle(2,0)
atlas(poses,'traveller-idle.png')
cv2.imwrite('/tmp/lastlight-idle.png',np.concatenate([poses[23],poses[47],poses[71],poses[95]],axis=1))
cv2.imwrite('/tmp/lastlight-rig.png',np.concatenate([np.concatenate(cycle()[i:i+8],axis=1) for i in range(0,48,8)],axis=0))
