#!/usr/bin/env python3
"""Render the Lullaby brand raster icons from the canonical geometry.

Draws the filled app icon (the lavender "L cradling a crescent moon" in cream on
a soft pastel tile) at high resolution, then downsamples to every size a Windows
app/installer/favicon needs and writes a multi-size ``.ico`` plus PNGs.

Geometry matches ``lullaby-icon.svg`` (a 120-unit design space). Run from anywhere:

    python assets/brand/render_icons.py

Outputs (next to this script):
    lullaby.ico            multi-size 16/24/32/48/64/128/256 (app + installer + favicon)
    lullaby-icon-256.png   256px app icon
    lullaby-icon-512.png   512px app icon
"""
from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw

HERE = Path(__file__).resolve().parent
UNIT = 120           # design space
SS = 1024            # supersample canvas (rendered, then downsampled)
S = SS / UNIT        # scale factor design -> supersample

CREAM = (255, 248, 239, 255)
# tile gradient stops (top-left -> mid -> bottom-right)
G0 = (217, 204, 255)   # #d9ccff
G1 = (196, 181, 253)   # #c4b5fd
G2 = (191, 230, 251)   # #bfe6fb


def lerp(a: tuple[int, int, int], b: tuple[int, int, int], t: float) -> tuple[int, int, int]:
    return tuple(round(a[i] + (b[i] - a[i]) * t) for i in range(3))


def diagonal_gradient(size: int) -> Image.Image:
    """A smooth 3-stop diagonal gradient, built small and upscaled (cheap + smooth)."""
    small = 128
    g = Image.new("RGB", (small, small))
    px = g.load()
    for y in range(small):
        for x in range(small):
            t = (x + y) / (2 * (small - 1))
            px[x, y] = lerp(G0, G1, t * 2) if t < 0.5 else lerp(G1, G2, (t - 0.5) * 2)
    return g.resize((size, size), Image.LANCZOS)


def sc(v: float) -> float:
    return v * S


def render_master() -> Image.Image:
    img = Image.new("RGBA", (SS, SS), (0, 0, 0, 0))

    # --- tile: rounded-rect gradient with a soft inner hairline ---
    tile_mask = Image.new("L", (SS, SS), 0)
    ImageDraw.Draw(tile_mask).rounded_rectangle(
        [sc(4), sc(4), sc(116), sc(116)], radius=sc(28), fill=255
    )
    grad = diagonal_gradient(SS).convert("RGBA")
    img.paste(grad, (0, 0), tile_mask)
    ImageDraw.Draw(img).rounded_rectangle(
        [sc(4.75), sc(4.75), sc(115.25), sc(115.25)],
        radius=sc(27.25), outline=(255, 255, 255, 140), width=max(1, round(sc(1.5))),
    )

    # --- mark layer (cream): the L, then the crescent, then the star ---
    mark = Image.new("RGBA", (SS, SS), (0, 0, 0, 0))
    d = ImageDraw.Draw(mark)

    # L: rounded polyline (matches stroke-linecap/linejoin = round)
    lw = round(sc(13))
    pts = [(sc(43), sc(30)), (sc(43), sc(78)), (sc(74), sc(78))]
    d.line(pts, fill=CREAM, width=lw, joint="curve")
    r = lw / 2
    for (cx, cy) in (pts[0], pts[2]):          # round the two open ends
        d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=CREAM)

    # crescent moon = full disc minus an offset disc (erase via the alpha band).
    # Lifted into the crook so it clears the L's stroke and reads as a crescent.
    moon = Image.new("RGBA", (SS, SS), (0, 0, 0, 0))
    md = ImageDraw.Draw(moon)
    md.ellipse([sc(71 - 18), sc(55 - 18), sc(71 + 18), sc(55 + 18)], fill=CREAM)
    alpha = moon.split()[3]
    ImageDraw.Draw(alpha).ellipse(
        [sc(80 - 15.5), sc(47 - 15.5), sc(80 + 15.5), sc(47 + 15.5)], fill=0
    )
    moon.putalpha(alpha)
    mark = Image.alpha_composite(mark, moon)

    # four-point twinkle star near the moon
    d2 = ImageDraw.Draw(mark)
    cxp, cyp, a, b = sc(94), sc(40), sc(6.5), sc(2.2)
    d2.polygon(
        [(cxp, cyp - a), (cxp + b, cyp - b), (cxp + a, cyp), (cxp + b, cyp + b),
         (cxp, cyp + a), (cxp - b, cyp + b), (cxp - a, cyp), (cxp - b, cyp - b)],
        fill=CREAM,
    )

    return Image.alpha_composite(img, mark)


def main() -> None:
    master = render_master()
    sizes = [16, 24, 32, 48, 64, 128, 256]
    frames = [master.resize((s, s), Image.LANCZOS) for s in sizes]
    ico_path = HERE / "lullaby.ico"
    frames[-1].save(ico_path, format="ICO", sizes=[(s, s) for s in sizes])
    master.resize((256, 256), Image.LANCZOS).save(HERE / "lullaby-icon-256.png")
    master.resize((512, 512), Image.LANCZOS).save(HERE / "lullaby-icon-512.png")
    print(f"wrote {ico_path.name} ({', '.join(str(s) for s in sizes)}), "
          f"lullaby-icon-256.png, lullaby-icon-512.png")


if __name__ == "__main__":
    main()
