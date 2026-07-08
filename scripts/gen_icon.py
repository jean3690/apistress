"""从 icon.svg 生成所有平台图标 — 高质量降采样"""
import xml.etree.ElementTree as ET
import os, math
from PIL import Image, ImageDraw

SVG_PATH = os.path.join(os.path.dirname(__file__), "..", "src-tauri", "icons", "icon.svg")
OUT = os.path.join(os.path.dirname(__file__), "..", "src-tauri", "icons")

# All sizes needed by various platforms
PNG_SIZES = [32, 128, 256, 512]
WINDOWS_SQUARE = [30, 44, 71, 89, 107, 142, 150, 284, 310]
# ICO must contain these resolutions for crisp taskbar rendering
ICO_SIZES = [16, 24, 32, 48, 64, 128, 256]

def hex_to_rgb(h):
    h = h.lstrip("#")
    return tuple(int(h[i:i+2], 16) for i in (0, 2, 4))

def render_svg_to_png(size):
    """Render the 512x512 SVG at a target size using Pillow."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    s = size / 512.0
    def sv(v): return v * s

    margin = sv(16)
    radius = int(sv(96))

    # Background gradient
    bg_start, bg_end = hex_to_rgb("#1e1e2e"), hex_to_rgb("#181825")
    for y in range(size):
        t = y / max(size - 1, 1)
        r = int(bg_start[0] + (bg_end[0] - bg_start[0]) * t)
        g = int(bg_start[1] + (bg_end[1] - bg_start[1]) * t)
        b = int(bg_start[2] + (bg_end[2] - bg_start[2]) * t)
        draw.line([(0, y), (size, y)], fill=(r, g, b))

    # Rounded rect mask
    mask = Image.new("L", (size, size), 0)
    ImageDraw.Draw(mask).rounded_rectangle(
        (margin, margin, size - margin, size - margin), radius, fill=255)
    pix, mp = img.load(), mask.load()
    for y in range(size):
        for x in range(size):
            if mp[x, y] == 0:
                pix[x, y] = (0, 0, 0, 0)

    # Border
    bw = max(1, int(sv(4)))
    draw.rounded_rectangle(
        (margin, margin, size - margin, size - margin), radius,
        outline=hex_to_rgb("#45475a"), width=bw)

    # Grid
    grid_c = hex_to_rgb("#313244")
    cl, cr, ct, cb = sv(120), sv(420), sv(140), sv(380)

    for y_frac, w in [(0.0, max(1, int(sv(2)))), (0.33, max(1, int(sv(1.5)))),
                       (0.66, max(1, int(sv(1.5)))), (1.0, max(1, int(sv(1.5))))]:
        y = ct + (cb - ct) * y_frac
        draw.line([(cl, y), (cr, y)], fill=grid_c, width=w)

    vgrid_alpha = (*grid_c, 127)
    for x_frac in [0.25, 0.5, 0.75]:
        x = cl + (cr - cl) * x_frac
        overlay = Image.new("RGBA", (size, size), (0, 0, 0, 0))
        ImageDraw.Draw(overlay).line(
            [(x, ct), (x, cb)], fill=vgrid_alpha, width=max(1, int(sv(1))))
        img.paste(overlay, (0, 0), overlay)

    # Trend line
    accent, accent2, success = hex_to_rgb("#89b4fa"), hex_to_rgb("#74c7ec"), hex_to_rgb("#a6e3a1")
    pts_svg = [(120,340),(150,340),(180,320),(210,320),(240,280),
               (270,280),(300,240),(330,240),(360,180),(390,180),(420,150)]
    pts = [(sv(x), sv(y)) for x, y in pts_svg]

    for gw, ga in [(max(2, int(sv(14))), 50), (max(1, int(sv(8))), 90)]:
        ov = Image.new("RGBA", (size, size), (0, 0, 0, 0))
        ImageDraw.Draw(ov).line(pts, fill=(*accent, ga), width=gw, joint="curve")
        img.paste(ov, (0, 0), ov)

    draw.line(pts, fill=accent, width=max(2, int(sv(8))), joint="curve")

    for dx, dy, dr, dc in [(sv(150),sv(340),max(2,int(sv(7))),accent),
                            (sv(240),sv(280),max(2,int(sv(7))),accent),
                            (sv(330),sv(240),max(2,int(sv(7))),accent),
                            (sv(420),sv(150),max(2,int(sv(9))),success)]:
        draw.ellipse([dx-dr, dy-dr, dx+dr, dy+dr], fill=dc)

    # Arrows
    ay = sv(410)
    mute_c = hex_to_rgb("#a6adc8")
    for hw, lw, color in [(sv(60), max(2,int(sv(4))), accent),
                           (sv(50), max(1,int(sv(3))), accent2),
                           (sv(40), max(1,int(sv(2.5))), mute_c)]:
        x1, x2 = sv(256) - hw, sv(256) + hw
        draw.line([(x1, ay), (x2, ay)], fill=color, width=lw)
        h = max(2, int(sv(8)))
        draw.polygon([(x2-h, ay-h//2), (x2, ay), (x2-h, ay+h//2)], fill=color)
        ay += sv(16)

    return img


# === Generate from 1024px master for crisp downscaling ===
master = render_svg_to_png(1024)

for sz in PNG_SIZES:
    img = master.resize((sz, sz), Image.LANCZOS)
    name = f"{sz}x{sz}.png" if sz <= 128 else \
           f"{sz//2}x{sz//2}@2x.png" if sz == 256 else "icon.png"
    img.save(os.path.join(OUT, name))

for sz in WINDOWS_SQUARE:
    master.resize((sz, sz), Image.LANCZOS).save(os.path.join(OUT, f"Square{sz}x{sz}Logo.png"))

master.resize((100, 100), Image.LANCZOS).save(os.path.join(OUT, "StoreLogo.png"))

# ICO with multiple resolutions (hand-built, Pillow's ICO save is broken for multi-res)
import struct, io
png_data = []
for s in ICO_SIZES:
    buf = io.BytesIO()
    master.resize((s, s), Image.LANCZOS).save(buf, format="PNG")
    png_data.append(buf.getvalue())
header = struct.pack("<HHH", 0, 1, len(ICO_SIZES))
offset = 6 + 16 * len(ICO_SIZES)
dirs, blobs = b"", b""
for sz, data in zip(ICO_SIZES, png_data):
    w = sz if sz < 256 else 0
    h = sz if sz < 256 else 0
    dirs += struct.pack("<BBBBHHII", w, h, 0, 0, 1, 32, len(data), offset)
    blobs += data
    offset += len(data)
with open(os.path.join(OUT, "icon.ico"), "wb") as f:
    f.write(header + dirs + blobs)

# ICNS
master.resize((1024, 1024), Image.LANCZOS).save(os.path.join(OUT, "icon.icns"))

count = len(PNG_SIZES) + len(WINDOWS_SQUARE) + 3
print(f"OK — {count} icons + multi-res ICO ({ICO_SIZES}) in {OUT}")
for f in sorted(os.listdir(OUT)):
    path = os.path.join(OUT, f)
    if os.path.isfile(path) and f.endswith(('.png','.ico')):
        kb = os.path.getsize(path) / 1024
        print(f"  {f} ({kb:.1f}KB)")
