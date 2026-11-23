//! The edit buffers of the four web manifest fields (RFC 012, job K2).
//!
//! The Customize page edits **text**; the plan (and so `settings.json`) only ever receives a valid,
//! canonical value. The buffer keeps what the user typed, valid or not, so a half-typed colour is
//! never saved and the field is never rewritten under the user's fingers: Enter shows the
//! canonical form. An error is shown only once the field has been typed in or submitted.

use logolig::WebManifestSettings;

/// One of the four fields of the web manifest form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestField {
    Name,
    ShortName,
    ThemeColor,
    BackgroundColor,
}

impl ManifestField {
    fn index(self) -> usize {
        match self {
            Self::Name => 0,
            Self::ShortName => 1,
            Self::ThemeColor => 2,
            Self::BackgroundColor => 3,
        }
    }

    pub fn is_color(self) -> bool {
        matches!(self, Self::ThemeColor | Self::BackgroundColor)
    }

    /// The canonical value of `text` for this field, or `None` when it is not valid: a colour is
    /// `#RRGGBB` or `#RGB` (canonical `#RRGGBB` upper case), a name is not empty (trimmed).
    pub fn normalize(self, text: &str) -> Option<String> {
        if self.is_color() {
            WebManifestSettings::normalize_color(text)
        } else {
            WebManifestSettings::normalize_text(text)
        }
    }
}

/// The buffers and which fields have been touched.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ManifestInputs {
    texts: [String; 4],
    touched: [bool; 4],
}

impl ManifestInputs {
    /// Buffers that show `settings` as they are, untouched (no error is shown).
    pub fn from_settings(settings: &WebManifestSettings) -> Self {
        Self {
            texts: [
                settings.name.clone(),
                settings.short_name.clone(),
                settings.theme_color.clone(),
                settings.background_color.clone(),
            ],
            touched: [false; 4],
        }
    }

    pub fn get(&self, field: ManifestField) -> &str {
        &self.texts[field.index()]
    }

    /// The user typed: the buffer holds the text, and the field is touched.
    pub fn set(&mut self, field: ManifestField, text: String) {
        self.texts[field.index()] = text;
        self.touched[field.index()] = true;
    }

    /// The field was submitted (Enter): touched, so an invalid value shows its error.
    pub fn touch(&mut self, field: ManifestField) {
        self.touched[field.index()] = true;
    }

    /// Whether the field shows its error: it was typed in or submitted, and its text is not valid.
    pub fn shows_error(&self, field: ManifestField) -> bool {
        self.touched[field.index()] && field.normalize(self.get(field)).is_none()
    }
}
