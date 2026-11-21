# Application logo (master)

`logolig.png` is the master application logo: PNG, 1254 × 1254, 8-bit RGBA, transparent outside the
rounded square. Every other size is derived from it (see "How every derived file is produced").

## Provenance and licence

**Author.** The logo was created by nabbisen (the author and owner of the Logolig repository), in 2026.

**AI assistance.** AI tools were used in the early phase of exploring ideas for the logo. The final artwork was created by the author.

**Copyright.** © 2026 nabbisen. The logo is **not** covered by the Apache License of the source code.

**Terms of use.** Permission is granted to copy and redistribute the logo **unmodified**, as part of Logolig or its packages and in material that refers to Logolig, in order to identify the application. Any other use, including modified versions and use for a different product or a fork, needs the author's permission. All other rights are reserved.

## Technical derivation

The owner's file `logolig-app-logo-base.png` (sha256 `53718618b22ec1754e9d0b539a33d0762a39e1f170ce25bce42db4be1680c323`)
is RGB, 1254 × 1254, with **opaque black** outside the rounded square. The architect derived the alpha
channel with the method documented in the header of `derive-alpha.sh` (ImageMagick 7); the header,
copied here:

```text
# Derives an alpha channel for logolig-app-logo-base.png (RGB, opaque black outside the rounded square).
# Method (architect, 2026-11-10): the base file's background is near-black with faint noise (max channel <= 8 of 255).
#  core   = pixels whose brightest channel is above 8 % (the rounded square plus its 1-pixel anti-aliased rim)
#  solid3 = core eroded by a disc of radius 3 (certainly inside): alpha 1, colour untouched
#  rim    = core minus solid3 (at most 3 px deep): colour = the nearest inside colour (dilated solid colours, radius 5),
#           alpha = brightest channel / brightest channel of that inside colour (snapped to 1 above 0.9)
#  outside core: alpha 0.
# Every pixel with alpha 1 is byte-identical to the input (checked). Needs ImageMagick 7 (magick).
```

**Every pixel with alpha 1 is byte-identical to the owner's file**; only the anti-aliased rim and the
outside differ. The result is `logolig.png`, sha256
`987eb24a74be9212fa9deea2748a9f6d0953ebe6a87d4d8c8a71182658c1163f`.

## How every derived file is produced

From `logolig.png` only, with **Logolig's own exporter** (the same resampling the product uses):
a small scratch program outside the repository (not committed) calls
`logolig::services::exporter::run_in_memory` on the master with this plan, everything else off:

```text
png_sizes                    = [32, 48, 64, 128, 256, 512]
include_microsoft_app_logos  = true
include_ico = include_svg = include_apple_touch = include_html_snippet = false
algorithm                    = Lanczos3 (the default)     keep_transparency = true (the default)
```

and writes the artifacts unchanged: `favicon-<N>.png` and the four Microsoft logo files.

| File | Size | Made from |
|---|---|---|
| `packaging/linux/icons/hicolor/{32x32,48x48,64x64,128x128,256x256,512x512}/apps/io.github.nabbisen.Logolig.png` | the directory's size, square | `favicon-<N>.png` |
| `packaging/windows/Assets/StoreLogo.png` | 50 × 50 | `StoreLogo.png` |
| `packaging/windows/Assets/Square44x44Logo.png` | 44 × 44 | `Square44x44Logo.png` |
| `packaging/windows/Assets/Square150x150Logo.png` | 150 × 150 | `Square150x150Logo.png` |
| `packaging/windows/Assets/Wide310x150Logo.png` | 310 × 150 (the logo contain-fit and centred on a transparent canvas) | `Wide310x150Logo.png` |
| `docs/assets/logolig-logo.png` | 256 × 256 (the README) | `favicon-256.png` |

All are RGBA with transparent corners.

## History

The earlier stand-in icon (an AI-derived flat re-interpretation of the old Store tile: the SVG master
and the icon theme's vector copy) was removed when the real logo was adopted; the repository history
keeps it.
