//! **The tree a reader finds, kept on the bus while the machine runs.**
//!
//! `crate::access_nodes` builds the tree and `crate::access_bus` puts it on the
//! accessibility bus, and until now **nothing called either of them outside a
//! test**. The tree was correct, tested against a real at-spi2 bus, and served to
//! nobody: a screen reader on a running alo machine would have found no
//! application at all.
//!
//! That is the gap this file closes, and it is worth saying plainly what it was,
//! because four EN 301 549 clauses were marked *met* against tests of a tree that
//! production never published. A test that proves the tree is right proves nothing
//! about whether anybody can read it.
//!
//! # Why a holder rather than a call in the loop
//!
//! What a reader is told has to follow what is open, and *what is open* changes
//! when a window is mapped or closed — not on a schedule. Rebuilding and
//! re-serving the whole tree once a frame would put a D-Bus round trip in the
//! middle of compositing sixty times a second; asking it once at start-up would
//! leave a reader holding a list of the windows that happened to be open then.
//!
//! So this holds the bus and the **names it last published**, and re-serves only
//! when those change. A frame being dragged, focused, zoomed or redrawn changes
//! nothing a reader is told, so it costs one comparison of a short list.
//!
//! # A machine with no reader is not a failure
//!
//! ADR 0063 — *a machine that cannot run the engine says so, rather than failing*.
//! A session with no accessibility bus is the ordinary case on a machine nobody
//! has turned a reader on for, so [`TheReaderIsTold::opened`] answers `None` and
//! the desktop runs exactly as before. What it does **not** do is pretend: the
//! refusal is returned, so a caller that wants to say *the reader could not be
//! reached* has something to say it about.

use crate::{ReadAloudBus, Server};
use alo_access::{Surface, TurnedOn};
use alo_strings::Strings;

/// The tree on the bus, and what it last said was open.
#[derive(Debug)]
pub struct TheReaderIsTold {
    /// The connection the tree is answered on.
    bus: ReadAloudBus,
    /// The window names last published, in the order they were published.
    ///
    /// The comparison that decides whether to re-serve. Names rather than
    /// surfaces, because a window that changes its title changes what a reader
    /// should hear just as much as one that opens.
    windows: Vec<String>,
    /// Which surfaces were up when the tree was last built.
    showing: Vec<Surface>,
}

impl TheReaderIsTold {
    /// Put this machine's tree on the accessibility bus, if there is one.
    ///
    /// `None` where no bus answered, which is a machine nobody is reading and
    /// not a fault. The refusal is carried so a caller can say which it was.
    ///
    /// # Errors
    /// Whatever [`ReadAloudBus::where_the_reader_is`] refuses, and
    /// [`crate::NotRead::NoRegistry`] when the tree is served but the registry
    /// will not embed it — a tree nothing embedded is one a reader walking the
    /// session never reaches, so it is reported rather than left looking served.
    pub fn opened(
        server: &Server,
        strings: &Strings,
        showing: &[Surface],
        turned_on: &TurnedOn,
    ) -> Result<Self, crate::NotRead> {
        let tree = server.read_aloud_with_the_frames_open(strings, showing, turned_on);
        let bus = ReadAloudBus::where_the_reader_is(&tree)?;
        bus.embedded()?;
        Ok(Self {
            windows: tree
                .the_windows_open_as_read()
                .into_iter()
                .map(str::to_owned)
                .collect(),
            showing: showing.to_vec(),
            bus,
        })
    }

    /// Follow what is open now, and say whether anything was republished.
    ///
    /// Cheap to call every frame and meant to be: it builds the tree, compares
    /// the window names and the surfaces up against what was last published, and
    /// only touches the bus when they differ.
    ///
    /// # Errors
    /// [`crate::NotRead::NotServed`] when the new tree could not be put on the
    /// bus. The old paths are already off by then, so a caller that ignores this
    /// leaves a reader with less than it had rather than a mixture of two
    /// moments — see [`ReadAloudBus::now_showing`].
    pub fn following(
        &mut self,
        server: &Server,
        strings: &Strings,
        showing: &[Surface],
        turned_on: &TurnedOn,
    ) -> Result<bool, crate::NotRead> {
        let tree = server.read_aloud_with_the_frames_open(strings, showing, turned_on);
        let windows: Vec<String> = tree
            .the_windows_open_as_read()
            .into_iter()
            .map(str::to_owned)
            .collect();
        if windows == self.windows && showing == self.showing.as_slice() {
            return Ok(false);
        }
        self.bus.now_showing(&tree)?;
        self.windows = windows;
        self.showing = showing.to_vec();
        Ok(true)
    }

    /// What a reader is currently told is open, in the order it hears them.
    #[must_use]
    pub fn windows(&self) -> &[String] {
        &self.windows
    }

    /// The name this machine's tree answers as on the bus.
    #[must_use]
    pub fn answers_as(&self) -> &str {
        self.bus.answers_as()
    }
}
