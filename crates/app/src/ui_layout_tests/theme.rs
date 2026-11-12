//! The layout does not depend on the theme (RFC 035, job JH2).
//!
//! The hover scan and the property checks run on the Light theme only, which halves their time.
//! That is sound only where Light and Dark lay the interface out identically, and this test proves
//! it over the whole matrix: for every (screen, size, language) the walk of the widget tree (kind,
//! text, bounds and visible bounds of every piece) is the same on both themes. **One screen is
//! the exception, found by this proof: Settings.** Its theme list marks the selected theme with a
//! check mark, so the row of "Dark" is wider when Dark is selected; that page is therefore run on
//! both themes (`layout_matrix`). (Colour contrast is a different question and belongs to RFC
//! 034's test of the colour functions.)

use super::support::{Case, Combo, layout_matrix, matrix, probe_of};

#[test]
fn the_layout_does_not_depend_on_the_theme() {
    let all = matrix();
    let mut compared = 0;
    for light in all.iter().filter(|c| !c.dark && c.case != Case::Settings) {
        let dark = Combo {
            dark: true,
            ..*light
        };
        let walk = |combo: &Combo| {
            let state = combo.state();
            probe_of(&state, combo.size).walk()
        };
        let (a, b) = (walk(light), walk(&dark));
        assert_eq!(
            a.len(),
            b.len(),
            "{}: another number of pieces on Dark",
            light.describe()
        );
        for (x, y) in a.iter().zip(&b) {
            assert_eq!(
                x,
                y,
                "{}: a piece is laid out differently on Dark",
                light.describe()
            );
        }
        compared += 1;
    }
    // Every combination is either compared here or run on both themes.
    let settings = all.iter().filter(|c| c.case == Case::Settings).count();
    assert_eq!(compared * 2 + settings, all.len());
    assert_eq!(
        layout_matrix().len(),
        compared + settings,
        "the properties run on every compared combination once and on Settings twice"
    );
}
