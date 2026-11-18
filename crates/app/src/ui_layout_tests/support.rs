//! The probe, the states and the matrix of the headless layout tests (RFC 035).
//!
//! `Probe` lays an interface out on the CPU renderer (no GPU, no display) and answers two
//! questions about it: where is every piece of text, container, scrollable and text input
//! (`iced_selector` walks the widget tree), and where does the pointer find something to
//! press (the mouse interaction `iced` reports for a hover, which is also how buttons,
//! checkboxes, pick lists and the slider, none of which the selector reports, are found).

use std::sync::{Arc, Mutex};

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
#[derive(Debug, Clone, PartialEq)]
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
        // The font system first: it is process-global and must be the hermetic one before any
        // text is measured.
        super::fonts::ensure();
        let mut renderer = iced_test::futures::futures::executor::block_on(
            <Renderer as iced_test::core::renderer::Headless>::new(
                Font::with_name("Fira Sans"),
                Pixels(16.0),
                // The CPU renderer, named here and not through the environment: no GPU, the
                // same result on every machine, and far faster than probing for an adapter.
                Some("tiny-skia"),
            ),
        )
        .expect("a headless renderer");
        let ui = UserInterface::build(
            element,
            size,
            user_interface::Cache::default(),
            &mut renderer,
        );
        Self { renderer, ui, size }
    }

    /// Runs an operation on the tree the way the runtime does, following `Outcome::Chain`
    /// (iced's `focus_next` is a `count` pass chained to a second pass).
    pub(super) fn run_chained(&mut self, op: impl iced_test::core::widget::Operation + 'static) {
        use iced_test::core::widget::operation::Outcome;
        let mut current: Box<dyn iced_test::core::widget::Operation> = Box::new(op);
        loop {
            self.ui.operate(&self.renderer, current.as_mut());
            match current.finish() {
                Outcome::Chain(next) => current = next,
                _ => break,
            }
        }
    }

    /// Every focus target of the tree in the order Tab visits them (tree order): where it is,
    /// whether it is a text input, and whether it holds the focus now.
    pub(super) fn focus_targets(&mut self) -> Vec<FocusTarget> {
        let found = Arc::new(Mutex::new((Vec::new(), false)));
        self.ui.operate(
            &self.renderer,
            &mut FocusScan {
                found: found.clone(),
            },
        );
        let (targets, _) = std::mem::take(&mut *found.lock().expect("the focus scan"));
        targets
    }

    /// Focuses the target at `index` of [`Probe::focus_targets`] **by identity**, not by counting
    /// Tab presses in the caller: nothing is focused first, so `index + 1` steps of `focus_next`
    /// reach it, and the result is checked against a second scan.
    pub(super) fn focus_target(&mut self, index: usize) {
        for _ in 0..=index {
            self.run_chained(widget::operation::focusable::focus_next::<()>());
        }
        let now: Vec<bool> = self.focus_targets().iter().map(|t| t.focused).collect();
        assert_eq!(
            now.iter().position(|f| *f),
            Some(index),
            "the focus is not on target {index}: {now:?}"
        );
    }

    /// Focuses the `k`-th text input (0-based) of the tree, whatever precedes it in the Tab order.
    pub(super) fn focus_text_input(&mut self, k: usize) {
        let index = self
            .focus_targets()
            .iter()
            .enumerate()
            .filter(|(_, t)| t.text_input)
            .nth(k)
            .map(|(i, _)| i)
            .unwrap_or_else(|| panic!("there is no text input number {k}"));
        self.focus_target(index);
    }

    /// The interface as pixels (RGBA, `scale` pixels per logical pixel), drawn with `theme` on the
    /// CPU renderer, with whatever holds the focus drawn as focused.
    pub(super) fn rgba(&mut self, theme: &Theme, scale: f32) -> Vec<u8> {
        use iced_test::core::renderer::Headless as _;
        let mut messages: Vec<M> = Vec::new();
        let _ = self.ui.update(
            &[Event::Window(iced::window::Event::RedrawRequested(
                iced::time::Instant::now(),
            ))],
            mouse::Cursor::Unavailable,
            &mut self.renderer,
            &mut clipboard::Null,
            &mut messages,
        );
        self.ui.draw(
            &mut self.renderer,
            theme,
            &iced_test::core::renderer::Style {
                text_color: iced::Color::BLACK,
            },
            mouse::Cursor::Unavailable,
        );
        let background = iced_test::core::theme::Base::base(theme).background_color;
        self.renderer.screenshot(
            Size::new(
                (self.size.width * scale) as u32,
                (self.size.height * scale) as u32,
            ),
            scale,
            background,
        )
    }

    /// Runs what a `Task` asks of the widget tree, as the runtime does: every widget operation it
    /// yields, in order (a task that yields none, or `Task::none()`, changes nothing).
    pub(super) fn run_task<T: 'static>(&mut self, task: iced::Task<T>) {
        use iced_test::futures::futures::StreamExt as _;
        let Some(mut stream) = iced_test::runtime::task::into_stream(task) else {
            return;
        };
        while let Some(action) = iced_test::futures::futures::executor::block_on(stream.next()) {
            if let iced_test::runtime::Action::Widget(operation) = action {
                self.run_chained(operation);
            }
        }
    }

    /// Where the focused control is on the screen and which viewports show it, if a control has
    /// the focus. The focus scan reports layout positions; each scroll area that holds the
    /// control (its content contains the control's centre) moves it by its current offset.
    pub(super) fn focused_view(&mut self) -> Option<FocusedView> {
        let found = Arc::new(Mutex::new(ViewScan::default()));
        self.ui.operate(
            &self.renderer,
            &mut ViewScanOp {
                found: found.clone(),
            },
        );
        let scan = std::mem::take(&mut *found.lock().expect("the view scan"));
        let layout = scan.focused?;
        let mut on_screen = layout;
        let mut viewports = Vec::new();
        for (bounds, content, translation) in &scan.scrollables {
            if content.contains(layout.center()) {
                on_screen.x -= translation.x;
                on_screen.y -= translation.y;
                viewports.push(*bounds);
            }
        }
        Some(FocusedView {
            on_screen,
            viewports,
            window: Rectangle::new(Point::ORIGIN, self.size),
        })
    }

    /// Delivers events as the window would: the statuses the tree answers and the messages its
    /// widgets published.
    pub(super) fn send(
        &mut self,
        events: &[Event],
        cursor: mouse::Cursor,
    ) -> (Vec<iced_test::core::event::Status>, Vec<M>) {
        let mut messages: Vec<M> = Vec::new();
        let (_state, statuses) = self.ui.update(
            events,
            cursor,
            &mut self.renderer,
            &mut clipboard::Null,
            &mut messages,
        );
        (statuses, messages)
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
    /// The result of the default plan with the preview panel open (the "View as" and "Surface"
    /// pickers, enabled).
    ResultPreview,
    /// The same with the transparency checker as the context: the Surface picker is disabled.
    ResultPreviewChecker,
}

pub(super) const CASES: [Case; 8] = [
    Case::Home,
    Case::Converting,
    Case::ResultDefault,
    Case::ResultAllOptions,
    Case::CustomizeAdvanced,
    Case::Settings,
    Case::ResultPreview,
    Case::ResultPreviewChecker,
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
            Case::ResultPreview | Case::ResultPreviewChecker => {
                state.screen = Screen::Result;
                state.result_assets = Some(result_assets_of_cards(false));
                state.result_preview_open = true;
                if self.case == Case::ResultPreviewChecker {
                    state.preview = Some(logolig::PreviewProfile {
                        context: logolig::PreviewContext::TransparencyChecker,
                        background: logolig::ThemeMode::Light,
                    });
                }
            }
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

/// The combinations whose layout is measured and hovered: the whole matrix on the Light theme,
/// and the Settings page on both themes. Elsewhere the layout does not depend on the theme, so
/// Dark would repeat every answer; Settings marks the selected theme with a check mark and so
/// lays out differently (both proved over the whole matrix by `theme`).
pub(super) fn layout_matrix() -> Vec<Combo> {
    matrix()
        .into_iter()
        .filter(|combo| !combo.dark || combo.case == Case::Settings)
        .collect()
}

/// The interface of `state` laid out at `size`.
pub(super) fn probe_of(state: &AppState, size: Size) -> Probe<'_, Message> {
    Probe::new(crate::shell::view(state), size)
}

/// A key press as the window delivers it, without text (arrows, Enter, Space, Tab).
pub(super) fn key_press(key: iced::keyboard::Key) -> Event {
    key_press_with(key, iced::keyboard::Modifiers::empty())
}

/// [`key_press`] with modifiers held.
pub(super) fn key_press_with(
    key: iced::keyboard::Key,
    modifiers: iced::keyboard::Modifiers,
) -> Event {
    Event::Keyboard(iced_test::core::keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: iced_test::core::keyboard::key::Physical::Code(
            iced_test::core::keyboard::key::Code::Escape,
        ),
        location: iced_test::core::keyboard::Location::Standard,
        modifiers,
        repeat: false,
        text: None,
    })
}

/// The events of a left click at `at`: the pointer moves there, presses and releases.
pub(super) fn click_events(at: Point) -> Vec<Event> {
    vec![
        Event::Mouse(mouse::Event::CursorMoved { position: at }),
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
    ]
}

/// Where the focused control is (see [`Probe::focused_view`]).
#[derive(Debug, Clone)]
pub(super) struct FocusedView {
    pub(super) on_screen: Rectangle,
    pub(super) viewports: Vec<Rectangle>,
    pub(super) window: Rectangle,
}

impl FocusedView {
    /// Is the control fully inside the window and inside every scroll area that holds it?
    pub(super) fn fully_visible(&self) -> bool {
        let inside = |r: &Rectangle, v: &Rectangle| {
            r.x >= v.x - 0.5
                && r.y >= v.y - 0.5
                && r.x + r.width <= v.x + v.width + 0.5
                && r.y + r.height <= v.y + v.height + 0.5
        };
        inside(&self.on_screen, &self.window)
            && self.viewports.iter().all(|v| inside(&self.on_screen, v))
    }
}

#[derive(Default)]
struct ViewScan {
    scrollables: Vec<(Rectangle, Rectangle, iced::Vector)>,
    focused: Option<Rectangle>,
}

struct ViewScanOp {
    found: Arc<Mutex<ViewScan>>,
}

impl widget::Operation for ViewScanOp {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn widget::Operation)) {
        operate(self);
    }

    fn scrollable(
        &mut self,
        _id: Option<&widget::Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        translation: iced::Vector,
        _state: &mut dyn widget::operation::Scrollable,
    ) {
        self.found.lock().expect("the view scan").scrollables.push((
            bounds,
            content_bounds,
            translation,
        ));
    }

    fn focusable(
        &mut self,
        _id: Option<&widget::Id>,
        bounds: Rectangle,
        state: &mut dyn widget::operation::Focusable,
    ) {
        if state.is_focused() {
            self.found.lock().expect("the view scan").focused = Some(bounds);
        }
    }
}

/// One focus target of the tree (see [`Probe::focus_targets`]).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct FocusTarget {
    pub(super) bounds: Rectangle,
    pub(super) text_input: bool,
    pub(super) focused: bool,
}

/// Collects the focus targets in tree order. `iced`'s text input announces itself as a text
/// input first and as a focus target second, so a pending flag marks the next target.
struct FocusScan {
    found: Arc<Mutex<(Vec<FocusTarget>, bool)>>,
}

impl widget::Operation for FocusScan {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn widget::Operation)) {
        operate(self);
    }

    fn text_input(
        &mut self,
        _id: Option<&widget::Id>,
        _bounds: Rectangle,
        _state: &mut dyn widget::operation::TextInput,
    ) {
        self.found.lock().expect("the focus scan").1 = true;
    }

    fn focusable(
        &mut self,
        _id: Option<&widget::Id>,
        bounds: Rectangle,
        state: &mut dyn widget::operation::Focusable,
    ) {
        let mut found = self.found.lock().expect("the focus scan");
        let text_input = std::mem::take(&mut found.1);
        found.0.push(FocusTarget {
            bounds,
            text_input,
            focused: state.is_focused(),
        });
    }
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
