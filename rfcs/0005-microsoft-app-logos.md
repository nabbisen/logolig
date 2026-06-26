# RFC 0005: Microsoft app logo advanced output set

- **Status**: Accepted
- **Target version**: v1.26.0
- **Author**: logolig maintainers
- **Created**: 2026-06-26

## Summary

Add an advanced, opt-in output group that generates four Microsoft app logo
PNGs from the same single source image used by the existing favicon flow:

- `StoreLogo.png` — 50×50
- `Square44x44Logo.png` — 44×44
- `Square150x150Logo.png` — 150×150
- `Wide310x150Logo.png` — 310×150

The feature must preserve the current minimal startup and result-screen UX.
It is not a full Windows asset-matrix generator.

## Requirements

1. The default export plan remains unchanged; Microsoft app logos are off by
   default.
2. The setting is exposed only under Advanced settings.
3. The setting is persisted as part of `ExportPlan`.
4. Enabling it adds exactly the four requested PNG files to in-memory and disk
   export paths.
5. The wide 310×150 output must preserve source aspect ratio and centre the
   logo on a transparent canvas instead of cropping or stretching.
6. The existing `keep_transparency` setting must apply to these PNG outputs.
7. The feature must work for PNG, WebP, JPEG, and SVG input sources.

## Design

Add `ExportPlan::include_microsoft_app_logos: bool` with serde default
compatibility. The exporter reads a small domain-level spec table rather than
hardcoding file names throughout the UI.

A new canvas helper performs contain-fit placement for raster inputs. SVG input
uses a rectangular SVG rasterisation path that applies the same contain-fit
logic directly from the SVG tree.

## Non-goals

- Scale-qualified Windows package assets such as `.scale-200.png` variants
- `targetsize-*` icon matrix generation
- Store screenshots or marketing image generation
- Multiple source images or batch mode
