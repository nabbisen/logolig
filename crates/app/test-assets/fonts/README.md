# Test fonts of the headless layout tests

**Test fixtures: not shipped.** They are read by `include_bytes!` inside `#[cfg(test)]` code
(`crates/app/src/ui_layout_tests/fonts.rs`), excluded from the published package
(`exclude = ["test-assets/"]` in `crates/app/Cargo.toml`) and not part of the binary. Their
reason: the width of a piece of text depends on the font that draws it, so the headless layout
tests (RFC 035) must not use the fonts of the machine they run on. The probe's font system
holds Fira Sans (bundled by `iced_graphics`), the Lucide icon font, and these two files.

| File | Source (system copy used) | Source sha256 | Subset sha256 |
|---|---|---|---|
| `NotoSansJP-subset.otf` | `/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc`, face 0 (Noto Sans CJK JP, Regular) | `b76b0433203017ca80401b2ee0dd69350349871c4b19d504c34dbdd80541690a` | `5b9802659ea6865f5ea9ef34317c47c147e0101ea38e2ae6742de17847237b79` |
| `AdwaitaSans-symbols-subset.ttf` | `/usr/share/fonts/Adwaita/AdwaitaSans-Regular.ttf` (Adwaita Sans, Regular) | `8381c33b9a44f066f2b99dba3d416a2342891e28c956a35dfd8d16ee2987e6d4` | `107b0b0bc8ef10f2c048d9bedc36c30aa163371444cf89d0b2ce67988abce541` |

Sizes: 108932 bytes and 3880 bytes.
Licence: SIL Open Font License 1.1, see `OFL.txt` (both source fonts; the copyright notices are there).

**Tool:** `pyftsubset` of fonttools 4.68.0, run by `scripts/make-test-font.sh` (by hand, never by the
tests; the output is deterministic, two runs give identical bytes):

```sh
pyftsubset NotoSansCJK-Regular.ttc --font-number=0 --text-file=<all characters> --layout-features='kern' \
  --no-hinting --desubroutinize --notdef-outline --no-recommended-glyphs \
  --name-IDs='*' --no-legacy-cmap --output-file=NotoSansJP-subset.otf
pyftsubset AdwaitaSans-Regular.ttf --text-file=<the characters the CJK face lacks> --layout-features='' \
  --no-hinting --notdef-outline --no-recommended-glyphs --name-IDs='*' --no-legacy-cmap \
  --output-file=AdwaitaSans-symbols-subset.ttf
```

The characters are every character of `crates/i18n/locales/*.toml` and of the Rust sources of
`crates/`, plus ASCII, the kana and the CJK and full-width punctuation. The test
`ui_layout_tests::fonts::the_font_set_covers_every_character_of_the_japanese_strings` fails, naming the
characters, when a Japanese string needs a glyph the fonts do not have; run the script and commit.
