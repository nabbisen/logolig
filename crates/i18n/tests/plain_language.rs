//! The plain-language texts of the Customize page (RFC 034, job J23).
//!
//! The page says what each option gives the user, in the user's words, with the technical name or
//! the file name beside it, in brackets. This file pins the confirmed text of every string of the
//! page by key, in both languages (a regression is caught by name), and the rules the text follows:
//!
//! * no jargon outside brackets in a title or a label (the list below names what was removed, and why);
//! * a blurb is at most two sentences;
//! * `favicon` and `ICO` are each explained once: the first in the PNG-sizes blurb, the second in the
//!   grayscale blurb, and `ICO` is named nowhere else on the page.
//!
//! Layout is not tested here: the matrix of `ui_layout_tests` lays every one of these strings out at
//! seven window sizes in both languages.

use logolig::MessageKey;
use logolig_i18n::{Locale, Translator};

/// Every string of the Customize page and its Advanced block, as confirmed:
/// key, English, Japanese.
const CONFIRMED: &[(MessageKey, &str, &str)] = &[
    (
        MessageKey::SectionPngOutputSizes,
        "Icon sizes (PNG files)",
        "アイコンのサイズ (PNG ファイル)",
    ),
    (
        MessageKey::SectionPngOutputSizesBlurb,
        "A favicon is the small icon shown in a browser tab. Each size you tick is saved as its own image file, such as favicon-32.png.",
        "favicon とは、 ブラウザのタブに表示される小さなアイコンのことです。 チェックを入れたサイズごとに、 favicon-32.png のような画像ファイルが 1 つずつ作られます。",
    ),
    (
        MessageKey::AddCustomSize,
        "+ Add another size",
        "+ ほかのサイズを追加",
    ),
    (MessageKey::CustomSizePlaceholder, "e.g. 256", "例: 256"),
    (
        MessageKey::SectionSvgConversion,
        "Turning a picture into SVG",
        "画像を SVG に変換する方法",
    ),
    (
        MessageKey::SectionSvgConversionBlurb,
        "These choices apply when a PNG, WebP or JPEG is turned into SVG, which happens only if \"Turn a PNG, WebP or JPEG into SVG\" under Advanced settings is on (it is on at first). An SVG file you load is copied as it is.",
        "ここでの選択は、 「上級設定」の「PNG・WebP・JPEG を SVG に変換する」がオンのとき (最初はオン) に、 PNG・WebP・JPEG を SVG に変換する方法として使われます。 読み込んだ SVG ファイルは、 そのままコピーされます。",
    ),
    (
        MessageKey::VtracerPresetSharp,
        "Logos and icons, with crisp edges (Sharp)",
        "ロゴやアイコン向け・輪郭くっきり (Sharp)",
    ),
    (
        MessageKey::VtracerPresetDefault,
        "A balance for most pictures (Default, recommended)",
        "ほとんどの画像向けのバランス型 (Default・推奨)",
    ),
    (
        MessageKey::VtracerPresetPhotoRich,
        "Photos and gradients, keeps fine color differences (Photo-rich)",
        "写真やグラデーション向け・色の細かな違いを残す (Photo-rich)",
    ),
    (MessageKey::SectionMisc, "Transparent areas", "透明な部分"),
    (
        MessageKey::KeepTransparency,
        "Keep transparent areas (alpha)",
        "透明な部分をそのまま残す (アルファ)",
    ),
    (
        MessageKey::KeepTransparencyBlurb,
        "If you turn this off, transparent areas are filled with white in all PNG images and in favicon.ico. SVG files are not changed.",
        "オフにすると、 すべての PNG 画像と favicon.ico で、 透明な部分が白になります。 SVG ファイルは変わりません。",
    ),
    (
        MessageKey::AdvancedExtrasSection,
        "Advanced settings",
        "上級設定",
    ),
    (
        MessageKey::IncludeAppleTouchLabel,
        "Icon for iPhone and iPad home screens (apple-touch-icon.png)",
        "iPhone・iPad のホーム画面用アイコン (apple-touch-icon.png)",
    ),
    (
        MessageKey::IncludeHtmlSnippetLabel,
        "Ready-to-paste lines for your web page (favicon-snippet.html)",
        "Web ページに貼り付けるだけの行 (favicon-snippet.html)",
    ),
    (
        MessageKey::IncludeSvgLabel,
        "Icon that stays sharp at any size (favicon.svg)",
        "どんな大きさでもくっきり見えるアイコン (favicon.svg)",
    ),
    (
        MessageKey::VectorizeOnRasterLabel,
        "Turn a PNG, WebP or JPEG into SVG (vtracer)",
        "PNG・WebP・JPEG を SVG に変換する (vtracer)",
    ),
    (
        MessageKey::SectionWebManifest,
        "Details for installing as an app (web manifest)",
        "アプリとしてインストールするための情報 (Web マニフェスト)",
    ),
    (
        MessageKey::SectionWebManifestBlurb,
        "Writes manifest.webmanifest, which gives a browser the name, colors and icons to use when your website is installed like an app; it lists the PNG sizes you ticked. Leave this off if you only need favicons.",
        "Web サイトをアプリのようにインストールするときにブラウザが使う、 名前・色・アイコンの情報 (manifest.webmanifest) を作り、 その中にチェックを入れた PNG のサイズを載せます。 favicon だけでよければオフのままで構いません。",
    ),
    (
        MessageKey::IncludeWebManifestLabel,
        "Save the app details (manifest.webmanifest)",
        "アプリの情報を出力 (manifest.webmanifest)",
    ),
    (MessageKey::WebManifestNameLabel, "App name", "アプリ名"),
    (
        MessageKey::WebManifestShortNameLabel,
        "Short name (for small spaces)",
        "短い名前 (表示場所が狭いとき用)",
    ),
    (
        MessageKey::WebManifestThemeColorLabel,
        "Theme color",
        "テーマカラー",
    ),
    (
        MessageKey::WebManifestBackgroundColorLabel,
        "Background color",
        "背景色",
    ),
    (
        MessageKey::WebManifestNamePlaceholder,
        "e.g. My App",
        "例: My App",
    ),
    (
        MessageKey::WebManifestShortNamePlaceholder,
        "e.g. App",
        "例: App",
    ),
    (
        MessageKey::WebManifestNameError,
        "Enter a name",
        "名前を入力してください",
    ),
    (
        MessageKey::WebManifestColorError,
        "Use a color like #1A2B3C or #1AB",
        "#1A2B3C や #1AB のような色を指定してください",
    ),
    (
        MessageKey::SectionMicrosoftAppLogos,
        "Logos for a Windows app (Microsoft Store)",
        "Windows アプリ用のロゴ (Microsoft Store)",
    ),
    (
        MessageKey::SectionMicrosoftAppLogosBlurb,
        "Four logo images for packaging a Windows app: StoreLogo.png, Square44x44Logo.png, Square150x150Logo.png and Wide310x150Logo.png. A website does not need them.",
        "Windows アプリを作るときに使う 4 つのロゴ画像です: StoreLogo.png, Square44x44Logo.png, Square150x150Logo.png, Wide310x150Logo.png。 Web サイトには必要ありません。",
    ),
    (
        MessageKey::IncludeMicrosoftAppLogosLabel,
        "Save the four Windows app logos",
        "Windows アプリ用のロゴ 4 つを出力",
    ),
    (
        MessageKey::SectionMonochrome,
        "Grayscale copies",
        "グレースケール版",
    ),
    (
        MessageKey::SectionMonochromeBlurb,
        "Saves grayscale copies of the PNG sizes and the ICO file (ICO is the classic favicon.ico file format) in a folder named mono, for one-color printing and similar uses. SVG files are not included.",
        "PNG の各サイズと ICO ファイル (ICO は favicon.ico の従来からある形式) のグレースケール版を mono フォルダーに保存し、 単色の印刷などに使えます。 SVG は含まれません。",
    ),
    (
        MessageKey::IncludeMonochromeLabel,
        "Also save grayscale copies (in the mono folder)",
        "グレースケール版も出力 (mono フォルダー)",
    ),
    (
        MessageKey::SectionResize,
        "How the picture is resized",
        "拡大・縮小のしかた",
    ),
    (
        MessageKey::SectionResizeBlurb,
        "How a PNG, WebP or JPEG is scaled to each icon size (an SVG is drawn directly at each size). The recommended choice suits most logos.",
        "PNG・WebP・JPEG を各アイコンのサイズに拡大・縮小する方法です (SVG は各サイズで直接描画されます)。 おすすめの選択肢は、 ほとんどのロゴに向いています。",
    ),
    (MessageKey::AlgorithmLabel, "Method:", "方法:"),
    (
        MessageKey::AlgorithmLanczos3,
        "Crisp and detailed, for most logos (Lanczos3, recommended)",
        "くっきり細かく仕上げる・ほとんどのロゴ向け (Lanczos3・推奨)",
    ),
    (
        MessageKey::AlgorithmMitchellNetravali,
        "Softer, with fewer halos around edges (Mitchell-Netravali)",
        "やわらかめ・輪郭の縁取りが出にくい (Mitchell-Netravali)",
    ),
    (
        MessageKey::AlgorithmCatmullRom,
        "Crisp, with slightly fewer halos than Lanczos3 (Catmull-Rom)",
        "くっきり・Lanczos3 より縁取りがやや少なめ (Catmull-Rom)",
    ),
    (
        MessageKey::AlgorithmBilinear,
        "Softest, with no halos around edges (Bilinear)",
        "いちばんやわらかく・縁取りが出ない (Bilinear)",
    ),
    (
        MessageKey::AlgorithmNearest,
        "Keeps hard pixel edges, for pixel art (Nearest)",
        "ピクセルの境目をそのまま残す・ピクセルアート向け (Nearest)",
    ),
    (MessageKey::ResetButton, "Reset to defaults", "既定値に戻す"),
];

/// The titles of the page's sections.
const TITLES: &[MessageKey] = &[
    MessageKey::SectionPngOutputSizes,
    MessageKey::SectionSvgConversion,
    MessageKey::SectionMisc,
    MessageKey::SectionWebManifest,
    MessageKey::SectionMicrosoftAppLogos,
    MessageKey::SectionMonochrome,
    MessageKey::SectionResize,
];

/// The labels: checkboxes, field labels, the options of the two choice groups.
const LABELS: &[MessageKey] = &[
    MessageKey::AddCustomSize,
    MessageKey::KeepTransparency,
    MessageKey::IncludeAppleTouchLabel,
    MessageKey::IncludeHtmlSnippetLabel,
    MessageKey::IncludeSvgLabel,
    MessageKey::VectorizeOnRasterLabel,
    MessageKey::IncludeWebManifestLabel,
    MessageKey::WebManifestNameLabel,
    MessageKey::WebManifestShortNameLabel,
    MessageKey::IncludeMicrosoftAppLogosLabel,
    MessageKey::IncludeMonochromeLabel,
    MessageKey::AlgorithmLabel,
    MessageKey::AlgorithmLanczos3,
    MessageKey::AlgorithmMitchellNetravali,
    MessageKey::AlgorithmCatmullRom,
    MessageKey::AlgorithmBilinear,
    MessageKey::AlgorithmNearest,
    MessageKey::VtracerPresetSharp,
    MessageKey::VtracerPresetDefault,
    MessageKey::VtracerPresetPhotoRich,
];

/// The one-line blurbs under a heading.
const BLURBS: &[MessageKey] = &[
    MessageKey::SectionPngOutputSizesBlurb,
    MessageKey::SectionSvgConversionBlurb,
    MessageKey::KeepTransparencyBlurb,
    MessageKey::SectionWebManifestBlurb,
    MessageKey::SectionMicrosoftAppLogosBlurb,
    MessageKey::SectionMonochromeBlurb,
    MessageKey::SectionResizeBlurb,
];

/// Words removed from the page's titles and labels because a reader who wants a favicon does not know
/// them (RFC 034 requirement 5). They may stand inside brackets, as the technical name of an option.
const JARGON_EN: &[(&str, &str)] = &[
    (
        "PWA",
        "a developer's acronym; the blurb says 'installed like an app'",
    ),
    (
        "BT.709",
        "a colour-conversion standard the user cannot act on",
    ),
    (
        "markup",
        "said of the pasted lines; the file name says the rest",
    ),
    ("raster", "'a PNG, WebP or JPEG' says it"),
    (
        "ectoriz",
        "'Vectorize' / 'vectorization': 'turn ... into SVG' says it",
    ),
    ("lgorithm", "'Algorithm': the group is 'Method'"),
    (
        "monochrome",
        "it is grayscale, which is the word a layperson knows",
    ),
    (
        "Monochrome",
        "it is grayscale, which is the word a layperson knows",
    ),
    ("custom", "'custom size': 'another size'"),
];

const JARGON_JA: &[(&str, &str)] = &[
    ("ラスタ", "'PNG・WebP・JPEG' says it"),
    ("ベクトル化", "'SVG に変換する' says it"),
    ("アルゴリズム", "the group is '方法'"),
    (
        "モノクローム",
        "グレースケール is the word a layperson knows",
    ),
    ("マークアップ", "the file name says the rest"),
    ("PWA", "a developer's acronym"),
    (
        "BT.709",
        "a colour-conversion standard the user cannot act on",
    ),
    ("カスタム", "'ほかのサイズ'"),
    ("方式", "'方法'"),
];

fn en() -> Translator {
    Translator::for_locale(Locale::En)
}

fn ja() -> Translator {
    Translator::for_locale(Locale::Ja)
}

/// `text` without the parts in brackets (the technical names).
fn outside_brackets(text: &str) -> String {
    let mut out = String::new();
    let mut depth = 0;
    for c in text.chars() {
        match c {
            '(' => depth += 1,
            ')' if depth > 0 => depth -= 1,
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

#[test]
fn every_string_of_the_customize_page_is_the_confirmed_text_in_both_languages() {
    assert!(
        CONFIRMED.len() >= 43,
        "the table lost rows: {}",
        CONFIRMED.len()
    );
    for (key, english, japanese) in CONFIRMED {
        assert_eq!(en().t(*key), *english, "en {key:?}");
        assert_eq!(ja().t(*key), *japanese, "ja {key:?}");
    }
}

#[test]
fn every_listed_key_is_in_the_table() {
    for key in TITLES.iter().chain(LABELS).chain(BLURBS) {
        assert!(
            CONFIRMED.iter().any(|(k, _, _)| k == key),
            "{key:?} is not pinned"
        );
    }
}

#[test]
fn no_title_and_no_label_has_jargon_outside_brackets() {
    for (translator, list, locale) in [(en(), JARGON_EN, "en"), (ja(), JARGON_JA, "ja")] {
        for key in TITLES.iter().chain(LABELS) {
            let plain = outside_brackets(&translator.t(*key));
            for (word, why) in list {
                assert!(
                    !plain.contains(word),
                    "[{locale}] {key:?} has '{word}' outside brackets ({why}): {plain}"
                );
            }
        }
    }
}

/// The check can fail: a title with the old wording is caught, and the same word in brackets is not.
#[test]
fn the_jargon_check_can_fail() {
    assert!(outside_brackets("Resize algorithm").contains("lgorithm"));
    assert!(!outside_brackets("How it is resized (algorithm: Lanczos3)").contains("lgorithm"));
    assert!(outside_brackets("SVG 変換方式").contains("方式"));
}

/// The number of sentences of a blurb: its full stops (". " inside, "." at the end), in English;
/// "。" in Japanese. A file name such as `favicon-32.png` has no ". ".
fn sentences(text: &str, locale: &str) -> usize {
    if locale == "ja" {
        text.matches('。').count()
    } else {
        let t = text.trim_end();
        t.matches(". ").count() + usize::from(t.ends_with('.'))
    }
}

#[test]
fn a_blurb_is_at_most_two_sentences() {
    for (translator, locale) in [(en(), "en"), (ja(), "ja")] {
        for key in BLURBS {
            let text = translator.t(*key);
            let n = sentences(&text, locale);
            assert!(
                (1..=2).contains(&n),
                "[{locale}] {key:?} has {n} sentences: {text}"
            );
        }
    }
}

#[test]
fn favicon_and_ico_are_each_explained_once() {
    // favicon: explained in the PNG-sizes blurb, the first text of the page.
    assert!(
        en().t(MessageKey::SectionPngOutputSizesBlurb)
            .starts_with("A favicon is ")
    );
    assert!(
        ja().t(MessageKey::SectionPngOutputSizesBlurb)
            .starts_with("favicon とは")
    );
    // ICO: the word "ICO" (capitals) is named in exactly one string of the page, and that string
    // explains it. The file name `favicon.ico` is lower case and is not the word.
    for (translator, explains) in [(en(), "ICO is the classic"), (ja(), "ICO は")] {
        let naming: Vec<MessageKey> = CONFIRMED
            .iter()
            .map(|(k, _, _)| *k)
            .filter(|k| translator.t(*k).contains("ICO"))
            .collect();
        assert_eq!(naming, [MessageKey::SectionMonochromeBlurb], "{naming:?}");
        assert!(
            translator
                .t(MessageKey::SectionMonochromeBlurb)
                .contains(explains)
        );
    }
}

#[test]
fn the_svg_blurb_names_the_switch_it_depends_on_and_the_switch_is_on_at_first() {
    // The switch: the Advanced option "Turn a PNG, WebP or JPEG into SVG" (its label without the
    // bracketed technical name), under the "Advanced settings" header.
    for (translator, open, close) in [(en(), '"', '"'), (ja(), '「', '」')] {
        let label = translator.t(MessageKey::VectorizeOnRasterLabel);
        let switch = label.split(" (").next().unwrap();
        let blurb = translator.t(MessageKey::SectionSvgConversionBlurb);
        assert!(
            blurb.contains(&format!("{open}{switch}{close}")),
            "the blurb quotes the switch '{switch}': {blurb}"
        );
        let advanced = translator.t(MessageKey::AdvancedExtrasSection);
        assert!(
            blurb.contains(&advanced),
            "the blurb names '{advanced}': {blurb}"
        );
    }
    // "it is on at first" is true.
    assert!(logolig::ExportPlan::default().vectorize_on_raster);
    assert!(
        en().t(MessageKey::SectionSvgConversionBlurb)
            .contains("on at first")
    );
    assert!(
        ja().t(MessageKey::SectionSvgConversionBlurb)
            .contains("最初はオン")
    );
}

#[test]
fn the_transparency_blurb_says_what_is_filled_and_with_what() {
    assert!(en().t(MessageKey::KeepTransparencyBlurb).contains("white"));
    assert!(ja().t(MessageKey::KeepTransparencyBlurb).contains("白"));
}
