//! **The order a desktop's task windows are painted in**: each window whole,
//! its panel and then its words, before the next window starts.
//!
//! Until 2026-10-27 every task window's panel was painted first and every
//! window's words after them all. With one window open, nobody could tell. With
//! two, the first window's words were painted **over** the second window's
//! panel, so the window in front showed the one behind it through itself: two
//! conversations' lines in one pile, unreadable. The fifth Demo 1 walk
//! photographed exactly that, with three tasks open.
//!
//! This is the order alone, as a list a test can read. `crate::desktop_paint`
//! follows it with the textures it imported. Whether the pixels then land
//! where this says is a picture's question, and a test cannot ask it.

/// One thing to paint, for one task window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TaskLayer<'a, T> {
    /// The window's panel and every rounded shape on it, as an imported texture.
    Panel(&'a T),
    /// The window's words, inked onto the panel painted just before.
    Words(&'a [crate::painted::Inked]),
}

/// **Each window's panel, then its words, window after window**, oldest
/// first, so the window opened last is painted last and is the one in front.
///
/// `panels` is the textures imported for these windows, in the same order. A
/// window whose texture is missing is left out whole, and never painted as words
/// with no panel under them, which is the same fault in a smaller form.
pub(crate) fn in_painting_order<'a, T>(
    words: impl IntoIterator<Item = &'a [crate::painted::Inked]>,
    panels: impl IntoIterator<Item = &'a T>,
) -> Vec<TaskLayer<'a, T>> {
    words
        .into_iter()
        .zip(panels)
        .flat_map(|(words, panel)| [TaskLayer::Panel(panel), TaskLayer::Words(words)])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{TaskLayer, in_painting_order};

    /// **The second window's panel comes after the first window's words**, so
    /// it covers them. Painted the old way it came before them, and the words
    /// of the window behind showed through the window in front.
    #[test]
    fn each_window_is_painted_whole_before_the_next() {
        let words: [&[crate::painted::Inked]; 2] = [&[], &[]];
        let panels = ["first", "second"];
        let order = in_painting_order(words, &panels);
        assert_eq!(
            order,
            vec![
                TaskLayer::Panel(&"first"),
                TaskLayer::Words(words[0]),
                TaskLayer::Panel(&"second"),
                TaskLayer::Words(words[1]),
            ]
        );
    }

    /// **A window with no texture is left out whole**: never its words alone,
    /// floating over whatever is behind it.
    #[test]
    fn a_window_without_its_panel_paints_nothing() {
        let words: [&[crate::painted::Inked]; 2] = [&[], &[]];
        let panels = ["only the first"];
        let order = in_painting_order(words, &panels);
        assert_eq!(
            order,
            vec![
                TaskLayer::Panel(&"only the first"),
                TaskLayer::Words(words[0])
            ]
        );
    }
}
