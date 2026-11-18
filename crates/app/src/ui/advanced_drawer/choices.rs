//! The message keys of the resize algorithms and the trace presets (the labels of the radio
//! groups of the Customize page, RFC 021).
//!
//! The enums themselves must not know their translated names (core must not depend on i18n), so
//! the UI layer maps each value to its `MessageKey`; the groups translate at view time.

use logolig::{MessageKey, ResizeAlgorithm, VtracerPreset};

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
