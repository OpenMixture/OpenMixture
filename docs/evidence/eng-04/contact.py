from PIL import Image, ImageDraw
from pathlib import Path
root=Path('tmp/eng-04/visual')
weights=['0','0.25','0.5','1']
canvas=Image.new('RGB',(2080,1140),'#181c24');d=ImageDraw.Draw(canvas)
d.text((20,12),'ENG-04 / Scalar blend / 1024 x 1024 source / 2 x 2 tiling per panel',fill='white')
for col,weight in enumerate(weights):
 for row,channel in enumerate(['height','normal']):
  x=16+col*516;y=64+row*540
  d.text((x,y-22),f'weight {weight} / {channel}',fill='white')
  im=Image.open(root/weight/(channel+'.png')).convert('RGB').resize((256,256),Image.Resampling.LANCZOS)
  for ty in range(2):
   for tx in range(2):canvas.paste(im,(x+tx*256,y+ty*256))
canvas.save(root/'contact.png')
