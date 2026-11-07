#!/usr/bin/env python3
"""Regenerate src/web_assets/favicon.ico from the site's star mark.

    ./scripts/make-favicon.py [OUT.ico]

The mark is the gold star already inlined as an SVG data: URI in `head()` (src/web.rs); this
renders that same path at icon sizes, so the .ico and the SVG favicon stay the same drawing. Edit
STAR/GOLD here and in that data: URI together. Pure standard library on purpose — the project has
no image dependency, and an icon is not worth adding one for.
"""
import struct, zlib, sys

GOLD = (0xE6, 0xA2, 0x3C)          # --primary (dark theme), as in the SVG favicon
# The SVG path "M16 3l3.4 8.6 9.2.5-7.1 5.9 2.3 8.9L16 25.9 8.2 30.9l2.3-8.9-7.1-5.9 9.2-.5z"
# resolved to absolute points on its 32x32 viewBox.
STAR = [(16,3), (19.4,11.6), (28.6,12.1), (21.5,18.0), (23.8,26.9),
        (16,25.9), (8.2,30.9), (10.5,22.0), (3.4,16.1), (12.6,15.6)]
VIEWBOX = 32.0
SS = 8                              # supersampling factor per axis -> SS*SS coverage samples

def inside(x, y, poly):
    """Even-odd ray cast; the outline is simple, so this matches SVG's nonzero rule."""
    hit = False
    n = len(poly)
    for i in range(n):
        x0, y0 = poly[i]
        x1, y1 = poly[(i + 1) % n]
        if (y0 > y) != (y1 > y):
            if x < x0 + (y - y0) * (x1 - x0) / (y1 - y0):
                hit = not hit
    return hit

def render(size):
    """RGBA bytes, size x size, the star in GOLD on transparent, antialiased by supersampling."""
    scale = VIEWBOX / (size * SS)
    rows = []
    for py in range(size):
        row = bytearray()
        for px in range(size):
            covered = 0
            for sy in range(SS):
                y = (py * SS + sy + 0.5) * scale
                for sx in range(SS):
                    x = (px * SS + sx + 0.5) * scale
                    if inside(x, y, STAR):
                        covered += 1
            a = covered * 255 // (SS * SS)
            row += bytes((GOLD[0], GOLD[1], GOLD[2], a))
        rows.append(bytes(row))
    return rows

def png(size, rows):
    def chunk(tag, data):
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data))
    ihdr = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)   # 8-bit RGBA, no interlace
    raw = b"".join(b"\x00" + r for r in rows)                   # filter type 0 per scanline
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", ihdr)
            + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))

def ico(sizes):
    """ICO with PNG-compressed entries (understood by every current browser and Windows Vista+)."""
    images = [png(s, render(s)) for s in sizes]
    header = struct.pack("<HHH", 0, 1, len(images))             # reserved, type 1 = icon, count
    offset = len(header) + 16 * len(images)
    entries = b""
    for s, img in zip(sizes, images):
        entries += struct.pack("<BBBBHHII", s & 0xFF, s & 0xFF, 0, 0, 1, 32, len(img), offset)
        offset += len(img)
    return header + entries + b"".join(images)

out = sys.argv[1] if len(sys.argv) > 1 else "src/web_assets/favicon.ico"
data = ico([16, 32, 48, 64])
open(out, "wb").write(data)
print("wrote", out, len(data), "bytes")
