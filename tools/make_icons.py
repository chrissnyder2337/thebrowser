#!/usr/bin/env python3
"""Generate Lantern's app icons from the pixel art in ui/lantern.txt.

Writes:
  ui/appicon/lantern_{32,256,1024}.png   window icons (embedded) + master PNG
  resources/mac/lantern.icns              macOS bundle icon
  resources/win/lantern.ico, small.ico    Windows executable icons

Usage: python3 tools/make_icons.py   (requires Pillow)
"""

from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
FRAME = "lit0"


def load_sheet(path):
    palette, frames, current = {}, {}, None
    for line in path.read_text().splitlines():
        line = line.rstrip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("= "):
            _, key, color = line.split()
            palette[key] = tuple(int(color[i : i + 2], 16) for i in (1, 3, 5)) + (255,)
        elif line.startswith("frame "):
            current = line.split()[1]
            frames[current] = []
        elif current:
            frames[current].append(line)
    return palette, frames


def render(palette, rows, scale):
    width, height = max(map(len, rows)), len(rows)
    img = Image.new("RGBA", (width * scale, height * scale), (0, 0, 0, 0))
    for y, row in enumerate(rows):
        for x, cell in enumerate(row):
            if cell != ".":
                img.paste(palette[cell], (x * scale, y * scale, (x + 1) * scale, (y + 1) * scale))
    return img


def icon(palette, rows, size):
    """The lantern centered on a transparent square, crisp where possible."""
    height = len(rows)
    scale = int(size * 0.94) // height
    if scale >= 1:
        art = render(palette, rows, scale)
    else:
        # Too small for whole pixels: downsample a large rendering.
        big = render(palette, rows, 32)
        ratio = size * 0.94 / big.height
        art = big.resize((max(1, round(big.width * ratio)), max(1, round(big.height * ratio))), Image.LANCZOS)
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    canvas.paste(art, ((size - art.width) // 2, (size - art.height) // 2), art)
    return canvas


def main():
    palette, frames = load_sheet(ROOT / "ui" / "lantern.txt")
    rows = frames[FRAME]

    out_png = ROOT / "ui" / "appicon"
    out_png.mkdir(parents=True, exist_ok=True)
    for size in (32, 256, 1024):
        icon(palette, rows, size).save(out_png / f"lantern_{size}.png", optimize=True)

    master = icon(palette, rows, 1024)
    mac = ROOT / "resources" / "mac"
    mac.mkdir(parents=True, exist_ok=True)
    master.save(mac / "lantern.icns")

    win = ROOT / "resources" / "win"
    win.mkdir(parents=True, exist_ok=True)
    sizes = [16, 24, 32, 48, 64, 128, 256]
    images = [icon(palette, rows, s) for s in sizes]
    images[-1].save(win / "lantern.ico", sizes=[(s, s) for s in sizes], append_images=images[:-1])
    small = [icon(palette, rows, s) for s in (16, 32)]
    small[-1].save(win / "small.ico", sizes=[(16, 16), (32, 32)], append_images=small[:-1])
    print("icons written")


if __name__ == "__main__":
    main()
