# RFC 0004: Simplified Chinese (zh-CN) locale

- **Status**: Draft
- **Target version**: v1.22.0+ option
- **Author**: logolig maintainers
- **Created**: 2026-05-05

## Summary

Add Simplified Chinese as a supported UI locale alongside English and
Japanese. The work parallels v1.6.0 (which added Japanese after the
v1.5.0 i18n base): add a `Locale::ZhCn` variant, ship a `zh-CN.toml`
dictionary covering all `MessageKey` variants, extend BCP-47 detection,
add the locale to the picker UI, and add tests that prove the
dictionary loads and substitutes placeholders correctly.

## Background

The v1.18.0 picker overlay design (`ui::picker_overlay::locale_view`)
shows three options today: 日本語, English, システム設定に従う. The
PNG mock for the redesign explicitly listed 简体中文 as a fourth option
but we deferred it as Q4-c at the time, on the grounds that adding a
locale is mechanical work that benefits from being its own version
boundary.

The i18n architecture from v1.5.0 makes this addition genuinely small:
because every UI string goes through `MessageKey` and the dictionary
must satisfy the enum exhaustively (the `match` in `locale_message_key`
fails to compile if any variant is missing), translating is a fill-in-
the-blanks exercise. The number of variants today is roughly 100 (UI
labels + error messages + toasts).

## External design

### Picker UI

The locale picker overlay (opened via the sidebar / bottom-nav language
icon) gains a fourth row, ordered for cultural / visual neutrality:

```
🌐 Language
   ✓ 日本語
     English
     简体中文
     システム設定に従う / Follow system / 跟随系统
```

The "follow system" row's label is the localised string for whichever
locale is *currently active* — same behaviour as v1.18.0. When
zh-CN is active the row reads "跟随系统".

### OS auto-detection

`sys-locale` is consulted at startup; if the OS reports `zh-CN`,
`zh_CN`, `zh-Hans`, or `zh_Hans_CN`, logolig launches in zh-CN.

If the OS reports `zh-TW` or any traditional-Chinese variant, logolig
**does not** automatically pick zh-CN. We treat zh-Hant as a separate
locale not yet supported; the user falls through to English (the v1.5.0
fallback). This is deliberate: Simplified and Traditional Chinese are
not mutually substitutable at the UI-string level (vocabulary and
characters differ) and showing one when the user expects the other is
worse than showing English.

### What is *not* translated

- The app name "Logolig" itself stays untranslated (it's a brand).
- Filenames produced by the exporter (`favicon.ico`, `apple-touch-icon.png`,
  `mono/`) stay in Latin alphabet — they're filesystem identifiers, not
  UI text.
- Error messages from upstream crates (image, resvg, vtracer) come
  through as their original English text. logolig wraps them in a
  translated `MessageKey::ErrorTitle` envelope but the inner cause
  remains in English. This is an inherited limitation from v1.5.0 and
  is documented in `docs/src/architecture.md`.

## Requirements

1. A new `Locale::ZhCn` variant in `logolig-i18n::locale`, covering
   every existing `MessageKey` variant.
2. BCP-47 detection accepts `zh-CN`, `zh_CN`, `zh-Hans`, `zh_Hans_CN`,
   and POSIX-style `zh_CN.UTF-8` (the latter for Linux glibc compat).
3. zh-TW and other Traditional Chinese variants must **not** map to
   `Locale::ZhCn`; they fall through to `Locale::En`.
4. The picker overlay shows a "简体中文" row when the locale list is
   open. Selecting it calls `Message::LocalePicked(Some(Locale::ZhCn))`
   and persists.
5. The Settings drawer's "follow system" copy renders correctly in
   each of the three other locales (en, ja, zh-CN) using the existing
   `LocaleSystem` `MessageKey`.
6. Compile-time exhaustiveness: a missing key in `zh-CN.toml` must be a
   build error, not a runtime fallback. (This already holds because of
   v1.5.0's `match` on `MessageKey`.)
7. The placeholder substitution mechanism (`{count}`, `{path}`, etc.)
   must work for zh-CN strings — Chinese punctuation around placeholders
   is allowed.

## Design

### `crates/logolig-i18n` changes

#### 1. New `Locale::ZhCn` variant in `locale.rs`

```rust
pub enum Locale {
    En,
    Ja,
    ZhCn,  // NEW
}
```

Update `Locale::all()`, `Locale::label()`, and any `Display` impl. Add
to BCP-47 parsing:

```rust
pub fn from_bcp47(tag: &str) -> Option<Self> {
    let normalised = tag.to_lowercase().replace('_', "-");
    let primary = normalised.split('-').next().unwrap_or("");
    let suffix = &normalised[primary.len()..];

    match primary {
        "en" => Some(Locale::En),
        "ja" => Some(Locale::Ja),
        "zh" => {
            // zh-Hans-* and zh-CN are Simplified.
            // zh-Hant-*, zh-TW, zh-HK, zh-MO are Traditional → unsupported.
            if suffix.contains("hans") || suffix.contains("-cn") {
                Some(Locale::ZhCn)
            } else if suffix.contains("hant") || suffix.contains("-tw")
                   || suffix.contains("-hk") || suffix.contains("-mo") {
                None
            } else {
                // Bare "zh" with no region: ambiguous. Fall through to None
                // so callers default to English. (CLDR likelySubtags would
                // map to zh-Hans but we don't pull in unic-langid.)
                None
            }
        }
        _ => None,
    }
}
```

Detect at startup in `detect_system_locale`:

```rust
sys_locale::get_locale()
    .as_deref()
    .and_then(Locale::from_bcp47)
    .unwrap_or(Locale::En)
```

#### 2. New `locales/zh-CN.toml`

A complete dictionary mirroring `ja.toml`'s key set. Roughly 100 keys
in current state. The file uses TOML's bare-key form with double-quoted
string values.

```toml
# Application core
app_title = "Logolig"  # untranslated brand
app_tagline = "favicon 生成器"

# Drop zone
drop_zone_headline = "拖入 PNG / SVG / WebP / JPEG"
drop_zone_or = "或"
drop_zone_choose_button = "选择文件…"

# ... (full list in the actual implementation)
```

The translator should consult a native zh-CN reviewer before merging —
machine translation gets the gist but mistakes terminology in technical
contexts ("favicon" vs "网站图标" vs "网页图标" — pick one and use it
consistently).

#### 3. `dictionary.rs` changes

Add a new field to `Dictionary` for every existing one (the struct is
auto-generated via `match` from `MessageKey`, so this is mechanical).
Add `Locale::ZhCn => &ZH_CN_DICT,` arm to `for_locale`.

Use `include_str!("../locales/zh-CN.toml")` to embed the dictionary
into the binary at compile time, matching ja.toml's treatment.

### `crates/logolig-app` changes

#### Picker overlay

In `ui::picker_overlay::locale_view`:

```rust
let options = column![
    picker_row(current == Some(Locale::Ja), t.t(MessageKey::LocaleNameJa), Message::LocalePicked(Some(Locale::Ja)), &theme),
    picker_row(current == Some(Locale::En), t.t(MessageKey::LocaleNameEn), Message::LocalePicked(Some(Locale::En)), &theme),
    picker_row(current == Some(Locale::ZhCn), t.t(MessageKey::LocaleNameZhCn), Message::LocalePicked(Some(Locale::ZhCn)), &theme),  // NEW
    picker_row(current.is_none(), t.t(MessageKey::LocaleSystem), Message::LocalePicked(None), &theme),
];
```

#### New `MessageKey` variant

Add `MessageKey::LocaleNameZhCn` for the picker row label. Its
translations:

| Locale | String |
| --- | --- |
| en | "Simplified Chinese" |
| ja | "簡体中文" |
| zh-CN | "简体中文" |

The native form ("简体中文") is preferred over a translated form
("Simplified Chinese") in most i18n style guides — users scan the menu
visually for *their* language's word for itself. The English / Japanese
rows already follow this convention (English row says "English", not
"英語"; Japanese row says "日本語"). The zh-CN row says "简体中文" in
all three locale dictionaries for the same reason.

### Settings persistence

`PersistedSettings::locale_override: Option<Locale>` is already an
enum that derives `Serialize`/`Deserialize` via `serde_json`. Adding
`ZhCn` is forward-compatible automatically. Backward compatibility:
when v1.21.0 wrote `"locale_override": "Ja"` and v1.22.x reads it, the
value matches; when a v1.22.x user writes `"ZhCn"` and somehow
downgrades to v1.21.0, the older binary's `serde` will fail to parse
that variant. We accept the asymmetry — versions of logolig don't
downgrade in production.

## Test plan

### Dictionary tests in `crates/logolig-i18n/tests/`

Match the v1.6.0 pattern. Reference the existing `ja.toml` test for
the test shapes; copy them and replace `Locale::Ja` with `Locale::ZhCn`.

| Test | Verifies |
| --- | --- |
| `zh_cn_locale_loads` | `Translator::for_locale(ZhCn)` returns a non-empty translator without panicking. |
| `zh_cn_differs_from_english_on_ui_keys` | A representative key (e.g. `MessageKey::AppTagline`) returns a different string for `En` vs `ZhCn`. |
| `zh_cn_substitutes_placeholders` | A `t_args` call with `{count}` placeholder correctly interpolates into the zh-CN string. |
| `zh_cn_picker_label_is_native` | `MessageKey::LocaleNameZhCn` returns "简体中文" in all three dictionaries. |

### BCP-47 detection tests

| Input | Expected |
| --- | --- |
| `"zh-CN"` | `Some(ZhCn)` |
| `"zh_CN"` | `Some(ZhCn)` |
| `"zh-Hans"` | `Some(ZhCn)` |
| `"zh-Hans-CN"` | `Some(ZhCn)` |
| `"zh_CN.UTF-8"` | `Some(ZhCn)` |
| `"zh-TW"` | `None` (falls through to English) |
| `"zh-Hant"` | `None` |
| `"zh-HK"` | `None` |
| `"zh"` (bare) | `None` |
| `"ZH-cn"` (case mix) | `Some(ZhCn)` |

### Compile-time check

The existing `match` in `locale_message_key` already enforces this. As
a paranoia measure, add a CI step or `cargo test` that loads each
dictionary and calls `t(...)` for every `MessageKey` variant; missing
keys panic. (v1.5.0 may have shipped this already — check first.)

### Manual checks

| Check | Steps | Pass condition |
| --- | --- | --- |
| Picker shows zh-CN | Open locale picker. | Fourth row reads "简体中文". |
| Selection persists | Pick zh-CN, restart app. | App launches in zh-CN. |
| Auto-detect on Linux | `LANG=zh_CN.UTF-8 cargo run`. | App launches in zh-CN. |
| Auto-detect rejects zh-TW | `LANG=zh_TW.UTF-8 cargo run`. | App launches in English (not zh-CN). |
| All visible strings translated | Drive through every screen. | No English / Japanese leaks (except untranslated upstream errors). |

## Security considerations

N/A. The locale dictionary is compiled into the binary via
`include_str!`, so it cannot be modified at runtime by an attacker. No
filesystem reads, no network. The BCP-47 parser operates on a
short string from `sys_locale` and uses only safe Rust.

One micro-consideration: the `OsString` returned by `sys_locale` may
contain non-UTF-8 on some weird Linux configurations. The current code
calls `.as_deref()` after a lossless conversion; verify that path
doesn't panic on non-UTF-8 by adding a unit test that feeds in a
deliberately ill-formed string. (This is a v1.5.0-era concern, not
specific to zh-CN, but worth re-checking when touching the area.)

## Related ROADMAP entry

See `docs/src/architecture.md` ROADMAP row for `v1.22.0+ (option)`,
sub-bullet (d).
