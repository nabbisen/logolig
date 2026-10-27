# Application icon (master)

`logolig.svg` is the master application icon. Every platform size is generated from it (Linux hicolor sizes, the Microsoft Store tiles), ideally with Logolig itself.

**Status: approved by the owner as the master application icon.** If the owner supplies original vector artwork, it replaces this file; nothing else changes.

## Provenance (please keep; Flathub asks for disclosure of AI-assisted material)

- The only artwork available in the repository was the 150 × 150 px Microsoft Store tile `packaging/windows/Assets/Square150x150Logo.png` (the hexagon mark on a blue rounded square). No larger or vector original was found in the repository or its history.
- This SVG was **derived from that tile by an AI agent** (the architect role), with Logolig's own services and `vtracer`: upscale to 1024 px (Lanczos3), snap the pixels to the two flat mark colours (orange, cream), trace them as polygons (`vtracer`, polygon mode, speckle filter), and place them on a rounded rectangle with a two-stop gradient measured from the original's lighting (about `#6b82a1` top-left to `#516989` bottom-right).
- It is a **flat re-interpretation**: the soft relief shading of the original mark is not reproduced. Shapes, proportions and colours follow the tile. The file has 13 paths and about 7 KB.
- No third-party artwork or font is used.

## Checks still to do

- Legibility at 16, 24, 32, 48 and 64 px (the dev job that installs the icons renders and reviews each size).
