//! The localised `pick_list` options (algorithm, trace preset) and their message keys.

use logolig::{MessageKey, ResizeAlgorithm, VtracerPreset};

// ---------------------------------------------------------------------------
// i18n wrappers for pick_list (ResizeAlgorithm / VtracerPreset)
// ---------------------------------------------------------------------------
//
// `pick_list` requires `Display` on options, but implementing it on
// ResizeAlgorithm / VtracerPreset directly would leak i18n into core.
// Instead: UI-layer wrappers that carry a translated label string.

#[derive(Debug, Clone)]
pub(super) struct LocalizedAlgorithm {
    pub(super) value: ResizeAlgorithm,
    pub(super) label: String,
}

impl PartialEq for LocalizedAlgorithm {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for LocalizedAlgorithm {}

impl std::fmt::Display for LocalizedAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

#[derive(Debug, Clone)]
pub(super) struct LocalizedPreset {
    pub(super) value: VtracerPreset,
    pub(super) label: String,
}

impl PartialEq for LocalizedPreset {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for LocalizedPreset {}

impl std::fmt::Display for LocalizedPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

// ---------------------------------------------------------------------------
// enum → MessageKey map (UI layer only, to keep core independent of i18n)
// ---------------------------------------------------------------------------

pub(super) fn algorithm_message_key(alg: ResizeAlgorithm) -> MessageKey {
    match alg {
        ResizeAlgorithm::Lanczos3 => MessageKey::AlgorithmLanczos3,
        ResizeAlgorithm::MitchellNetravali => MessageKey::AlgorithmMitchellNetravali,
        ResizeAlgorithm::CatmullRom => MessageKey::AlgorithmCatmullRom,
        ResizeAlgorithm::Bilinear => MessageKey::AlgorithmBilinear,
        ResizeAlgorithm::Nearest => MessageKey::AlgorithmNearest,
    }
}

pub(super) fn preset_message_key(preset: VtracerPreset) -> MessageKey {
    match preset {
        VtracerPreset::Sharp => MessageKey::VtracerPresetSharp,
        VtracerPreset::Default => MessageKey::VtracerPresetDefault,
        VtracerPreset::PhotoRich => MessageKey::VtracerPresetPhotoRich,
    }
}
