#!/usr/bin/env python3
"""Generate NotchKu app icons (PNG + ICO) from scratch.

Run locally:  python3 scripts/generate_icon.py
In CI: the release workflow runs this before `tauri build`, so the
icon files never need to be committed (they are gitignored).
"""
import os
from PIL import Image, ImageDraw, ImageFont

OUT_DIR = os.path.join(os.path.dirname(__file__), "..", "src-tauri", "icons")
SIZE = 512

img = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
d = ImageDraw.Draw(img)

# Dark rounded square background
d.rounded_rectangle([8, 8, SIZE - 8, SIZE - 8], radius=110, fill=(17, 17, 17, 255))

# The notch pill: white pill, centered, with a purple dot
d.rounded_rectangle([76, 196, SIZE - 76, 316], radius=60, fill=(255, 255, 255, 255))
d.ellipse([SIZE // 2 - 28, 228, SIZE // 2 + 28, 284], fill=(124, 58, 237, 255))

# Wordmark
try:
    font = ImageFont.truetype("DejaVuSans-Bold.ttf", 64)
except OSError:
    font = ImageFont.load_default()
text = "NotchKu"
bbox = d.textbbox((0, 0), text, font=font)
d.text(((SIZE - (bbox[2] - bbox[0])) / 2, 348), text, font=font, fill=(255, 255, 255, 255))

os.makedirs(OUT_DIR, exist_ok=True)
png_path = os.path.join(OUT_DIR, "icon.png")
ico_path = os.path.join(OUT_DIR, "icon.ico")
img.save(png_path, "PNG")
img.save(ico_path, "ICO", sizes=[(16, 16), (32, 32), (48, 48), (256, 256)])
print(f"Wrote {png_path} and {ico_path}")
