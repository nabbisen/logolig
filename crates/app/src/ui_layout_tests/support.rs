//! The probe, the states and the matrix of the headless layout tests (RFC 035).
//!
//! `Probe` lays an interface out on the CPU renderer (no GPU, no display) and answers two
//! questions about it: where is every piece of text, container, scrollable and text input
//! (`iced_selector` walks the widget tree), and where does the pointer find something to
//! press (the mouse interaction `iced` reports for a hover, which is also how buttons,
//! checkboxes, pick lists and the slider, none of which the selector reports, are found).

use std::sync::{Arc, Mutex, Once};

use iced::{Element, Font, Pixels, Point, Rectangle, Renderer, Size, Theme, mouse};
use iced_test::core::widget::{self, Operation as _};
use iced_test::core::{Event, clipboard};
use iced_test::runtime::{UserInterface, user_interface};
use iced_test::selector::{Candidate, Selector as _};
use logolig::{ExportPlan, WebManifestSettings};
use logolig_i18n::{Locale, Translator};

use crate::app::test_support::result_assets_of_cards;
use crate::app::{AppState, Message, NavPage, Screen};

/// What a piece of the widget tree is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Text,
    TextInput,
    Container,
    Scrollable,
    Other,
}

/// One piece of the widget tree: its layout bounds and the part of it a viewport shows.
#[derive(Debug, Clone)]
pub(super) struct Seen {
    pub(super) kind: Kind,
    /// The text of a `Text` or `TextInput`, empty for the rest.
    pub(super) text: String,
    pub(super) bounds: Rectangle,
    pub(super) visible: Option<Rectangle>,
}

/// An interface laid out on the CPU renderer.
pub(super) struct Probe<'a, M = Message> {
    renderer: Renderer,
    ui: UserInterface<'a, M, Theme, Renderer>,
    pub(super) size: Size,
}

impl<'a, M> Probe<'a, M> {
    pub(super) fn new(element: Element<'a, M>, size: Size) -> Self {
        static FONTS: Once = Once::new();
        let backend = std::env::var("ICED_TEST_BACKEND").ok();
        let mut renderer = iced_test::futures::futures::executor::block_on(
            <Renderer as iced_test::core::renderer::Headless>::new(
                Font::with_name("Fira Sans"),
                Pixels(16.0),
                backend.as_deref(),
            ),
        )
        .expect("a headless renderer");
        // The application loads the Lucide icon font at start-up; do the same, once.
        FONTS.call_once(|| {
            iced_test::renderer::graphics::text::font_system()
                .write()
                .expect("the font system")
                .load_font(std::borrow::Cow::Borrowed(snora::lucide::LUCIDE_FONT_BYTES));
        });
        let ui = UserInterface::build(
            element,
            size,
            user_interface::Cache::default(),
            &mut renderer,
        );
        Self { renderer, ui, size }
    }

    /// Every text, container, scrollable and text input of the tree, depth first.
    pub(super) fn walk(&mut self) -> Vec<Seen> {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let sink = seen.clone();
        let selector = move |candidate: Candidate<'_>| -> Option<()> {
            let mut out = sink.lock().expect("the walk's result");
            out.push(match candidate {
                Candidate::Text {
                    bounds,
                    visible_bounds,
                    content,
                    ..
                } => Seen {
                    kind: Kind::Text,
                    text: content.to_string(),
                    bounds,
                    visible: visible_bounds,
                },
                Candidate::TextInput {
                    bounds,
                    visible_bounds,
                    state,
                    ..
                } => Seen {
                    kind: Kind::TextInput,
                    text: state.text().to_string(),
                    bounds,
                    visible: visible_bounds,
                },
                Candidate::Container {
                    bounds,
                    visible_bounds,
                    ..
                } => Seen {
                    kind: Kind::Container,
                    text: String::new(),
                    bounds,
                    visible: visible_bounds,
                },
                Candidate::Scrollable {
                    bounds,
                    visible_bounds,
                    ..
                } => Seen {
                    kind: Kind::Scrollable,
                    text: String::new(),
                    bounds,
                    visible: visible_bounds,
                },
                Candidate::Focusable {
                    bounds,
                    visible_bounds,
                    ..
                }
                | Candidate::Custom {
                    bounds,
                    visible_bounds,
                    ..
                } => Seen {
                    kind: Kind::Other,
                    text: String::new(),
                    bounds,
                    visible: visible_bounds,
                },
            });
            // Never "found": the walk visits the whole tree.
            None
        };
        let mut operation = selector.find();
        self.ui.operate(
            &self.renderer,
            &mut widget::operation::black_box(&mut operation),
        );
        let _ = operation.finish();
        seen.lock().expect("the walk's result").clone()
    }

    /// The mouse interaction the interface reports with the pointer at `p` (a hover: it changes
    /// nothing in the interface).
    fn interaction_at(&mut self, p: Point) -> mouse::Interaction {
        let mut messages: Vec<M> = Vec::new();
        let events = [Event::Mouse(mouse::Event::CursorMoved { position: p })];
        let (state, _) = self.ui.update(
            &events,
            mouse::Cursor::Available(p),
            &mut self.renderer,
            &mut clipboard::Null,
            &mut messages,
        );
        match state {
            user_interface::State::Updated {
                mouse_interaction, ..
            } => mouse_interaction,
            user_interface::State::Outdated => mouse::Interaction::None,
        }
    }

    /// Does the pointer find something to press at `p`?
    pub(super) fn is_interactive_at(&mut self, p: Point) -> bool {
        !matches!(
            self.interaction_at(p),
            mouse::Interaction::None | mouse::Interaction::Idle | mouse::Interaction::Hidden
        )
    }

    /// The rectangle around `p` in which the pointer finds something to press: the contiguous
    /// run of interactive points in each of the four directions, scanned pixel by pixel (a
    /// binary search would jump over a gap between two controls) and refined to a quarter pixel.
    fn extent_around(&mut self, p: Point) -> Rectangle {
        let size = self.size;
        let mut reach = |dx: f32, dy: f32, max: f32| -> f32 {
            let mut near = 0.0_f32;
            while near + 1.0 <= max
                && self
                    .is_interactive_at(Point::new(p.x + dx * (near + 1.0), p.y + dy * (near + 1.0)))
            {
                near += 1.0;
            }
            // The edge lies between `near` and `near + 1`.
            let (mut lo, mut hi) = (near, (near + 1.0).min(max));
            while hi - lo > 0.25 {
                let mid = (lo + hi) / 2.0;
                if self.is_interactive_at(Point::new(p.x + dx * mid, p.y + dy * mid)) {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            lo
        };
        let left = reach(-1.0, 0.0, p.x);
        let right = reach(1.0, 0.0, size.width - p.x);
        let up = reach(0.0, -1.0, p.y);
        let down = reach(0.0, 1.0, size.height - p.y);
        Rectangle::new(
            Point::new(p.x - left, p.y - up),
            Size::new(left + right, up + down),
        )
    }

    /// The interactive areas of the interface: for every visible piece of the tree, five
    /// points (centre and the four quarter points on the axes) are tried; where the pointer
    /// finds something to press, the area around that point is measured. A point inside an area
    /// already found is skipped.
    pub(super) fn interactive_areas(&mut self, seen: &[Seen]) -> Vec<Rectangle> {
        let mut areas: Vec<Rectangle> = Vec::new();
        for piece in seen {
            let Some(v) = piece.visible else { continue };
            // A 3 x 3 grid inside the piece: a control (the slider is 16 px high inside a
            // container four times as tall) need not lie on the centre or on the axes.
            let points = [0.15_f32, 0.5, 0.85].into_iter().flat_map(|fy| {
                [0.15_f32, 0.5, 0.85]
                    .into_iter()
                    .map(move |fx| Point::new(v.x + v.width * fx, v.y + v.height * fy))
            });
            for p in points {
                if areas.iter().any(|a| a.contains(p)) || !self.is_interactive_at(p) {
                    continue;
                }
                // Measure again from the centre of the first measurement, so the result does
                // not depend on which point of the control the search happened to start from.
                let first = self.extent_around(p);
                areas.push(self.extent_around(first.center()));
            }
        }
        areas
    }
}

// ---------------------------------------------------------------------------------------------
// The matrix
// ---------------------------------------------------------------------------------------------

/// The screens of the application that are checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Case {
    Home,
    Converting,
    /// The result of the default plan: 7 cards.
    ResultDefault,
    /// The result with every option on: 16 cards.
    ResultAllOptions,
    /// Customize with the advanced settings open and every option on (the widest set of controls).
    CustomizeAdvanced,
    Settings,
}

pub(super) const CASES: [Case; 6] = [
    Case::Home,
    Case::Converting,
    Case::ResultDefault,
    Case::ResultAllOptions,
    Case::CustomizeAdvanced,
    Case::Settings,
];

/// The window sizes, in logical pixels. 767 and 768 straddle the mobile breakpoint.
pub(super) const SIZES: [(f32, f32); 7] = [
    (1280.0, 720.0),
    (1024.0, 600.0),
    (800.0, 600.0),
    (767.0, 480.0),
    (768.0, 480.0),
    (640.0, 480.0),
    (360.0, 640.0),
];

/// One combination of the matrix.
#[derive(Debug, Clone, Copy)]
pub(super) struct Combo {
    pub(super) case: Case,
    pub(super) size: Size,
    pub(super) locale: Locale,
    pub(super) dark: bool,
}

impl Combo {
    pub(super) fn describe(&self) -> String {
        format!(
            "{:?} {}x{} {} {}",
            self.case,
            self.size.width,
            self.size.height,
            self.locale.as_bcp47(),
            if self.dark { "Dark" } else { "Light" }
        )
    }

    /// The state the application is in for this combination.
    pub(super) fn state(&self) -> AppState {
        let all_options = matches!(self.case, Case::ResultAllOptions | Case::CustomizeAdvanced);
        let export_plan = if all_options {
            ExportPlan {
                monochrome: true,
                include_microsoft_app_logos: true,
                web_manifest: Some(WebManifestSettings::default()),
                ..ExportPlan::default()
            }
        } else {
            ExportPlan::default()
        };
        let mut state = AppState {
            window_size: self.size,
            theme: if self.dark {
                logolig::ThemeMode::Dark
            } else {
                logolig::ThemeMode::Light
            },
            translator: Translator::for_locale(self.locale),
            locale_override: Some(self.locale),
            export_plan,
            ..AppState::default()
        };
        match self.case {
            Case::Home => {}
            Case::Converting => state.screen = Screen::Converting,
            Case::ResultDefault | Case::ResultAllOptions => {
                state.screen = Screen::Result;
                state.result_assets = Some(result_assets_of_cards(all_options));
            }
            Case::CustomizeAdvanced => {
                state.nav_page = NavPage::Customize;
                state.advanced_extras_open = true;
            }
            Case::Settings => state.nav_page = NavPage::Settings,
        }
        state
    }
}

/// Every (screen, size, language, theme).
pub(super) fn matrix() -> Vec<Combo> {
    let mut all = Vec::new();
    for case in CASES {
        for (w, h) in SIZES {
            for locale in Locale::all() {
                for dark in [false, true] {
                    all.push(Combo {
                        case,
                        size: Size::new(w, h),
                        locale,
                        dark,
                    });
                }
            }
        }
    }
    all
}

/// The interface of `state` laid out at `size`.
pub(super) fn probe_of(state: &AppState, size: Size) -> Probe<'_, Message> {
    Probe::new(crate::shell::view(state), size)
}

/// A finding: one property failing at one place.
#[derive(Debug, Clone)]
pub(super) struct Finding {
    pub(super) combo: String,
    pub(super) what: String,
}

impl std::fmt::Display for Finding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.combo, self.what)
    }
}

/// The label of the first text that lies inside `area`, for naming a control in a finding.
pub(super) fn label_in(seen: &[Seen], area: Rectangle) -> String {
    seen.iter()
        .find(|s| {
            s.kind == Kind::Text && !s.text.trim().is_empty() && area.contains(s.bounds.center())
        })
        .map(|s| s.text.clone())
        .unwrap_or_else(|| "(no label)".to_string())
}
