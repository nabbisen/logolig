//! State cues that are not selections (RFC 021, D7).
//!
//! Selection is drawn by `snora`'s checkbox and radio group (`ui/choice.rs`) and the active page
//! by the navigation bar (`ui/sidebar.rs`). What is left: *busy* and *success*. Both are drawn
//! from the bundled icon font (lucide, registered at start-up), never from a text glyph that
//! depends on the machine's fonts (the old `⏳` showed as a boxed "?" where no font had it).
//! One meaning per cue: the check mark means success or ready and nothing else.

use iced::widget::{row, text};
use iced::{Alignment, Element};

use snora::Icon;
use snora::lucide;
use snora::widget::icon_element_sized;

use crate::app::Message;

/// "In progress": the `Loader` icon and the message, at the message's size.
pub fn busy<'a>(message: String, size: f32) -> Element<'a, Message> {
    row![
        icon_element_sized::<Message>(&Icon::Lucide(lucide::Loader), size),
        text(message).size(size),
    ]
    .spacing(size * 0.5)
    .align_y(Alignment::Center)
    .into()
}

/// "Done": the `Check` icon, for the success headline of the Result screen.
pub fn success<'a>(size: f32) -> Element<'a, Message> {
    icon_element_sized::<Message>(&Icon::Lucide(lucide::Check), size)
}
