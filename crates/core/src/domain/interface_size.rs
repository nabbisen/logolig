//! The interface size (RFC 034, job J24): how large the whole application is drawn.
//!
//! Three steps, named in words and not in percentages: **Normal** (100 %), **Large** (125 %) and
//! **Extra large** (150 %). The application multiplies the system's display scaling by
//! [`InterfaceSize::factor`], so text, icons and spacing grow together. There is no 200 % step and no
//! free value: a user who needs more uses the operating system's scaling as well.
//!
//! ## The stored value is read leniently
//!
//! The settings file is hand-editable and may come from another version. An unknown name, an empty
//! string, the wrong case, a number, `null` or any other JSON value is read as [`InterfaceSize::Normal`];
//! reading never fails because of this field, so the application always starts. Only the three canonical
//! lowercase names (`"normal"`, `"large"`, `"extra_large"`) select a step. The value is written in that
//! canonical form.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// How large the interface is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum InterfaceSize {
    /// 100 %: the size the application was designed at.
    #[default]
    Normal,
    /// 125 %.
    Large,
    /// 150 %.
    ExtraLarge,
}

impl InterfaceSize {
    /// The three steps, in the order the Settings page shows them.
    pub const fn all() -> [Self; 3] {
        [Self::Normal, Self::Large, Self::ExtraLarge]
    }

    /// The factor the interface is scaled by, on top of the operating system's own scaling.
    pub const fn factor(self) -> f32 {
        match self {
            Self::Normal => 1.0,
            Self::Large => 1.25,
            Self::ExtraLarge => 1.5,
        }
    }

    /// The canonical stored name.
    const fn name(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Large => "large",
            Self::ExtraLarge => "extra_large",
        }
    }

    /// The step a stored name selects; `None` for anything that is not one of the three names.
    fn from_name(name: &str) -> Option<Self> {
        Self::all().into_iter().find(|step| step.name() == name)
    }
}

impl Serialize for InterfaceSize {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.name())
    }
}

impl<'de> Deserialize<'de> for InterfaceSize {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        /// A string, or anything else (consumed and ignored).
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Stored {
            Name(String),
            Other(serde::de::IgnoredAny),
        }
        Ok(match Stored::deserialize(deserializer)? {
            Stored::Name(name) => Self::from_name(&name).unwrap_or_default(),
            Stored::Other(_) => Self::default(),
        })
    }
}
