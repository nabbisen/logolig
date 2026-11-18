//! Single-choice groups (RFC 021): a value with its translated label, for `snora::radio_group`.
//!
//! `radio_group` takes options that are `ToString + PartialEq + Clone` and returns the chosen
//! option to a handler. Our values (a locale, a theme, a preview context, a resize algorithm)
//! must not know their translated names, and the names change with the language, so the label is
//! attached here, at view time, and the equality of two options is the equality of their values.
//!
//! The group is vertical (it has no orientation setting), one Tab stop, and ↑ ↓ ← → move **and
//! select**; a click on the option that is already selected publishes nothing.

use snora::LayoutDirection;

use crate::app::Message;

/// A value with the label the user sees. Equal when the values are equal, whatever the labels.
#[derive(Debug, Clone)]
pub(crate) struct Choice<T> {
    pub(crate) value: T,
    pub(crate) label: String,
}

impl<T: PartialEq> PartialEq for Choice<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> std::fmt::Display for Choice<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

/// A radio group over `values`, `selected` marked, each shown as `label(value)`; choosing one
/// (a click or an arrow key) makes `on_select(value)`. The application is left to right.
pub(crate) fn radio_group<'a, T>(
    values: impl IntoIterator<Item = T>,
    selected: &T,
    label: impl Fn(&T) -> String,
    on_select: impl Fn(T) -> Message + 'a,
) -> snora::RadioGroup<'a, Choice<T>, Message>
where
    T: Clone + PartialEq + 'a,
{
    let options: Vec<Choice<T>> = values
        .into_iter()
        .map(|value| Choice {
            label: label(&value),
            value,
        })
        .collect();
    // Equality is by value, so the label of the marker is irrelevant.
    let current = Choice {
        value: selected.clone(),
        label: String::new(),
    };
    snora::radio_group(
        options,
        Some(&current),
        move |choice| on_select(choice.value),
        LayoutDirection::Ltr,
    )
}

/// A checkbox for an independent option (RFC 021): one Tab stop, Enter or Space toggles, the mark
/// is a box with a tick (a shape), and the label is inside the target. The caller adds
/// `.label(..)` and `.on_toggle(..)`. The application is left to right.
pub(crate) fn checkbox<'a>(checked: bool) -> snora::Checkbox<'a, Message> {
    snora::checkbox(checked, LayoutDirection::Ltr)
}
