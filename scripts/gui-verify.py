#!/usr/bin/env python3
"""gui-verify.py: the pixel checks of the real-window runs, on FILES (a live capture or a saved one alike).

Boxes are in LOGICAL pixels; --scale is capture pixels per logical pixel (1 on X11, 1.2 or so on a scaled
Wayland output: see capture_scale in gui-verify.sh). Each command prints one number.

  gui-verify.py [--scale S] diff   A.png B.png x0 y0 x1 y1       pixels that differ (luma > 40) in the box
  gui-verify.py [--scale S] accent IMG.png x0 y0 x1 y1 R G B     pixels within colour distance 40 of (R,G,B)
  gui-verify.py [--scale S] mean   IMG.png x0 y0 x1 y1           mean luminance 0..255
  gui-verify.py scale IMG.png LOGICAL_WIDTH                      capture width / logical width

Run it with `python3 -I` (the callers do). A size mismatch in `diff` prints 99999999.
"""
import sys
from PIL import Image, ImageChops


def box_of(args, scale):
    return tuple(int(round(float(v) * scale)) for v in args[:4])


def main(argv):
    scale = 1.0
    if argv and argv[0] == "--scale":
        scale = float(argv[1])
        argv = argv[2:]
    cmd, rest = argv[0], argv[1:]
    if cmd == "scale":
        print(round(Image.open(rest[0]).width / float(rest[1]), 4))
    elif cmd == "diff":
        a, b = Image.open(rest[0]).convert("RGB"), Image.open(rest[1]).convert("RGB")
        if a.size != b.size:
            print(99999999)
            return
        box = box_of(rest[2:], scale)
        d = ImageChops.difference(a.crop(box), b.crop(box)).convert("L").point(lambda v: 255 if v > 40 else 0)
        print(sum(1 for v in d.get_flattened_data() if v) if hasattr(d, "get_flattened_data") else sum(1 for v in d.getdata() if v))
    elif cmd == "accent":
        im = Image.open(rest[0]).convert("RGB").crop(box_of(rest[1:], scale))
        r, g, b = (int(v) for v in rest[5:8])
        px = im.get_flattened_data() if hasattr(im, "get_flattened_data") else im.getdata()
        print(sum(1 for p in px if ((p[0] - r) ** 2 + (p[1] - g) ** 2 + (p[2] - b) ** 2) ** 0.5 < 40))
    elif cmd == "mean":
        im = Image.open(rest[0]).convert("RGB").crop(box_of(rest[1:], scale))
        px = list(im.get_flattened_data() if hasattr(im, "get_flattened_data") else im.getdata())
        print(round(sum(0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2] for p in px) / len(px)))
    else:
        sys.exit("unknown command " + cmd)


main(sys.argv[1:])
