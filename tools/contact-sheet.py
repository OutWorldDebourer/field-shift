"""Contact sheet: lays phone screenshots side by side (one row) so a whole scroll can be reviewed in one image.

    python tools/contact-sheet.py <out.jpg> <img1> <img2> ...   (each scaled to 300 px wide)
"""
import sys
from PIL import Image

out, *paths = sys.argv[1:]
W = 300
ims = [Image.open(p).convert('RGB') for p in paths]
ims = [im.resize((W, round(im.height * W / im.width))) for im in ims]
H = max(im.height for im in ims)
sheet = Image.new('RGB', (W * len(ims) + 8 * (len(ims) - 1), H), (255, 0, 255))
x = 0
for im in ims:
    sheet.paste(im, (x, 0))
    x += W + 8
sheet.save(out, quality=82)
print(out, sheet.size)
