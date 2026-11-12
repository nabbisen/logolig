#!/bin/sh
# Regenerates the test fonts of the headless layout tests (RFC 035, job JH2) from the system's
# Noto Sans CJK (Japanese text) and Adwaita Sans (the few symbols the CJK font lacks, such as the
# reload arrow). Run by hand, never by the tests; the results are committed.
#
#   scripts/make-test-font.sh [NotoSansCJK-Regular.ttc [AdwaitaSans-Regular.ttf]]
#
# Needs python3 with fonttools (`pip install fonttools`: `pyftsubset` and the `fontTools` module).
# The subsets hold every character of the locale files and of the Rust sources of `crates/`, plus
# ASCII, the kana, the CJK and full-width punctuation, so a changed string is covered after one
# run of this script. Both source fonts are under the SIL Open Font License 1.1.
set -eu
cd "$(dirname "$0")/.."
CJK="${1:-/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc}"
SYM="${2:-/usr/share/fonts/Adwaita/AdwaitaSans-Regular.ttf}"
DIR=crates/app/test-assets/fonts
OUT_CJK="$DIR/NotoSansJP-subset.otf"
OUT_SYM="$DIR/AdwaitaSans-symbols-subset.ttf"
TEXT="$(mktemp)"
MISSING="$(mktemp)"
trap 'rm -f "$TEXT" "$MISSING"' EXIT

# All wanted characters; and those of them the Japanese face of the CJK font does not have.
python3 - "$CJK" "$TEXT" "$MISSING" <<'PY'
import glob, sys
from fontTools.ttLib import TTCollection
chars = set(chr(c) for c in range(0x20, 0x7F))
for lo, hi in [(0x3000, 0x303F), (0x3040, 0x309F), (0x30A0, 0x30FF), (0xFF01, 0xFF5E), (0xFF61, 0xFF9F)]:
    chars |= set(chr(c) for c in range(lo, hi + 1))
files = glob.glob("crates/i18n/locales/*.toml") + glob.glob("crates/**/*.rs", recursive=True)
for path in sorted(files):
    chars |= set(open(path, encoding="utf-8").read())
chars -= set("\n\r\t")
cmap = TTCollection(sys.argv[1]).fonts[0].getBestCmap()
open(sys.argv[2], "w", encoding="utf-8").write("".join(sorted(chars)))
open(sys.argv[3], "w", encoding="utf-8").write("".join(sorted(c for c in chars if ord(c) not in cmap)))
PY

# Face 0 of the collection is the Japanese face. No hinting, kerning only, no timestamp: the
# same input gives the same bytes.
pyftsubset "$CJK" --font-number=0 --text-file="$TEXT" --layout-features='kern' \
  --no-hinting --desubroutinize --notdef-outline --no-recommended-glyphs \
  --name-IDs='*' --no-legacy-cmap --output-file="$OUT_CJK"
pyftsubset "$SYM" --text-file="$MISSING" --layout-features='' \
  --no-hinting --notdef-outline --no-recommended-glyphs \
  --name-IDs='*' --no-legacy-cmap --output-file="$OUT_SYM"
printf 'characters the CJK face lacks, taken from Adwaita Sans: %s\n' "$(cat "$MISSING")"
sha256sum "$CJK" "$SYM" "$OUT_CJK" "$OUT_SYM"
ls -l "$OUT_CJK" "$OUT_SYM"
