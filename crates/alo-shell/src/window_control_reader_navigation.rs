//! Worded reader navigation snapshots, without keyboard or pointer ownership.
use alo_strings::{Filling, Said, Strings};

use crate::window_control_reader_words::{
    READER_DISMISS, READER_NEXT, READER_POSITION, READER_PREVIOUS,
};
use crate::{Server, WindowControlLabelError, WindowControlReader, WindowControlReaderPage};

/// A reading request from the trusted host, never a window operation or binding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowControlReaderNavigation {
    /// Read the preceding page; refuse at the start without wrapping.
    Previous,
    /// Read the following page; refuse at the end without wrapping.
    Next,
}

/// Complete navigation wording and boundary state for a borrowed reader page.
/// Frozen data only: discard across host events. This is neither a submitted
/// frame nor input authority. Render all wording with its individual provenance.
#[derive(Debug)]
pub struct WindowControlReaderChrome {
    /// Externalized position, with both numeric gaps filled.
    pub position: Said,
    /// Previous-page name, retained even when the operation is unavailable.
    pub previous: Said,
    /// Next-page name, retained even when the operation is unavailable.
    pub next: Said,
    /// Dismiss-reader name; deliberately distinct from CloseWindow.
    pub dismiss: Said,
    /// Whether the current snapshot has a preceding page.
    pub can_previous: bool,
    /// Whether the current snapshot has a following page.
    pub can_next: bool,
}

impl WindowControlReaderPage<'_> {
    /// Prepare all reader wording atomically, including unavailable navigation.
    /// Invalid position/count returns Geometry; missing/unfilled vocabulary
    /// returns Vocabulary; empty or excessive wording returns Text. Translation
    /// fallback remains Said data, never a silently assembled English sentence.
    /// The 128-page bound is the page preparer's existing allocation bound.
    pub fn chrome(
        &self,
        strings: &Strings,
    ) -> Result<WindowControlReaderChrome, WindowControlLabelError> {
        if !(1..=128).contains(&self.total) || !(1..=self.total).contains(&self.number) {
            return Err(WindowControlLabelError::Geometry);
        }
        let chrome = WindowControlReaderChrome {
            position: strings.say(
                &READER_POSITION.key(),
                &Filling::of("page", self.number.to_string()).and("total", self.total.to_string()),
            ),
            previous: strings.say(&READER_PREVIOUS.key(), &Filling::nothing()),
            next: strings.say(&READER_NEXT.key(), &Filling::nothing()),
            dismiss: strings.say(&READER_DISMISS.key(), &Filling::nothing()),
            can_previous: self.number > 1,
            can_next: self.number < self.total,
        };
        for said in [
            &chrome.position,
            &chrome.previous,
            &chrome.next,
            &chrome.dismiss,
        ] {
            if said.is_a_bug() {
                return Err(WindowControlLabelError::Vocabulary);
            }
            if said.text().trim().is_empty() || said.text().len() > 4096 {
                return Err(WindowControlLabelError::Text);
            }
        }
        Ok(chrome)
    }
}

impl Server {
    /// Navigate a live reader by one page without wrapping, input capture or
    /// executing its window control. Validate lifetime before boundary refusal,
    /// so even a request at the first/last page retires an observed stale reader.
    pub fn navigate_window_control_reader<'a>(
        &mut self,
        reader: &'a mut WindowControlReader,
        navigation: WindowControlReaderNavigation,
    ) -> Option<WindowControlReaderPage<'a>> {
        let selected = reader.selected();
        let page = self.read_window_control_page(reader, selected)?;
        let index = match navigation {
            WindowControlReaderNavigation::Previous => selected.checked_sub(1)?,
            WindowControlReaderNavigation::Next if page.number < page.total => selected + 1,
            WindowControlReaderNavigation::Next => return None,
        };
        self.read_window_control_page(reader, index)
    }
}
