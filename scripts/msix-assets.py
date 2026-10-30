"""Renders the MSIX package's logos into src-tauri/gen/windows/Assets/.

As DESIGN.md sets out: up to 48 px the mark is the simplified drawing `small.svg`, from 64 px the
rendered master `source.png`. Edge renders `small.svg` at 1024 px on a transparent background,
and each size is a Lanczos downscale of the render or the master. Windows draws the
`targetsize` icons on the taskbar, Start and Alt+Tab; the `unplated` forms with no plate behind
them. Run again whenever either source changes:

    python scripts/msix-assets.py

Needs Pillow and Microsoft Edge.
"""

import subprocess
import tempfile
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
ICONS = ROOT / "src-tauri" / "icons"
ASSETS = ROOT / "src-tauri" / "gen" / "windows" / "Assets"
EDGE = Path(r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe")
TARGET_SIZES = [16, 20, 24, 30, 32, 36, 40, 48, 60, 64, 72, 80, 96, 256]


def render_small() -> Image.Image:
    with tempfile.TemporaryDirectory() as tmp:
        out = Path(tmp) / "small.png"
        subprocess.run(
            [
                str(EDGE),
                "--headless",
                "--disable-gpu",
                "--hide-scrollbars",
                "--default-background-color=00000000",
                "--window-size=1024,1024",
                f"--screenshot={out}",
                (ICONS / "small.svg").as_uri(),
            ],
            check=True,
            capture_output=True,
        )
        return Image.open(out).convert("RGBA").copy()


def main() -> None:
    ASSETS.mkdir(parents=True, exist_ok=True)
    small = render_small()
    # Square, with the tile centred and even margins, as generated.
    large = Image.open(ICONS / "source.png").convert("RGBA")

    def at(size: int) -> Image.Image:
        return (small if size <= 48 else large).resize((size, size), Image.LANCZOS)

    at(44).save(ASSETS / "Square44x44Logo.png")
    at(150).save(ASSETS / "Square150x150Logo.png")
    at(50).save(ASSETS / "StoreLogo.png")
    wide = Image.new("RGBA", (310, 150))
    wide.paste(at(120), ((310 - 120) // 2, 15))
    wide.save(ASSETS / "Wide310x150Logo.png")
    for size in TARGET_SIZES:
        icon = at(size)
        for suffix in ("", "_altform-unplated", "_altform-lightunplated"):
            icon.save(ASSETS / f"Square44x44Logo.targetsize-{size}{suffix}.png")


if __name__ == "__main__":
    main()
