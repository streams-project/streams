#!/usr/bin/env python3
import sys
from PIL import Image

def measure(shot_path, x0, y0, x1, y1, t):
    im = Image.open(shot_path).convert('L')
    xs, ys = [], []
    for y in range(y0, y1):
        for x in range(x0, x1):
            if im.getpixel((x, y)) >= t:
                xs.append(x)
                ys.append(y)
    if not xs:
        print('box px None dp None')
        return None
    min_x, min_y, max_x, max_y = min(xs), min(ys), max(xs), max(ys)
    dp_vals = [round(v / 2.5625, 1) for v in (min_x, min_y, max_x, max_y)]
    width_px = max_x - min_x + 1
    height_px = max_y - min_y + 1
    width_dp = round(width_px / 2.5625, 1)
    height_dp = round(height_px / 2.5625, 1)
    print(f'box px {min_x} {min_y} {max_x} {max_y} (size {width_px}x{height_px}) dp {dp_vals} (size {width_dp}x{height_dp})')
    return (min_x, min_y, max_x, max_y)

if __name__ == '__main__':
    if len(sys.argv) < 7:
        print('Usage: box.py shot.png x0 y0 x1 y1 threshold')
        sys.exit(1)
    measure(sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4]), int(sys.argv[5]), int(sys.argv[6]))
