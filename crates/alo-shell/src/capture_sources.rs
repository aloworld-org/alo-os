//! Source identities follow actual output and mapping lifetimes, never titles.

use super::{
    Server,
    capture_transport::{Endpoint, Feed},
};
use alo_capture_protocol::{MAX_SOURCES, Source, SourceKind, frame_bytes};
use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Size},
};
use std::{collections::BTreeMap, io, sync::Arc};

/// A mapped window's private source and the mapping lifetime it was chosen in.
struct Window {
    /// The selected root, never a rectangle sampled from the desktop.
    root: WlSurface,
    /// The compositor changes this identity on unmap, hide or a Place change.
    mapping: Arc<()>,
    /// Fixed dimensions and capture interest.
    feed: Arc<Feed>,
}

/// Owned only by the signed-in desktop's compositor, never its greeter.
pub(super) struct CaptureSources {
    /// Worker endpoint; it carries no Wayland objects across threads.
    endpoint: Endpoint,
    /// Random per-compositor generation; a restart cannot recreate an old handle.
    generation: String,
    /// Monotonic identities within this generation, independent of app titles.
    next: u64,
    /// Submitted displays, removed before a disconnected display can return.
    monitors: BTreeMap<String, Arc<Feed>>,
    /// Currently capturable application mappings only.
    windows: Vec<Window>,
}

impl CaptureSources {
    /// Capture is explicitly enabled at the person's desktop entry point.
    pub(super) fn bind(wayland: &std::path::Path) -> io::Result<Self> {
        let mut random = [0; 24];
        getrandom::fill(&mut random).map_err(io::Error::other)?;
        let generation = random.iter().map(|b| format!("{b:02x}")).collect();
        Ok(Self {
            endpoint: Endpoint::bind(wayland)?,
            generation,
            next: 0,
            monitors: BTreeMap::new(),
            windows: Vec::new(),
        })
    }

    /// Assign an unrepeatable handle, refusing invalid dimensions before pixels.
    fn source(
        &mut self,
        name: String,
        kind: SourceKind,
        width: u32,
        height: u32,
    ) -> Option<Arc<Feed>> {
        frame_bytes(width, height).ok()?;
        self.next = self.next.checked_add(1)?;
        Some(Feed::new(Source {
            id: format!("{}-{}", self.generation, self.next),
            name,
            kind,
            width,
            height,
        }))
    }

    /// Snapshot sources and set output copy interest before submission.
    pub(super) fn prepare(
        &mut self,
        server: &Server,
        named: &str,
        size: Size<i32, Physical>,
        target: &mut (impl crate::FrameTarget + ?Sized),
    ) {
        let dimensions = (
            u32::try_from(size.w).unwrap_or(0),
            u32::try_from(size.h).unwrap_or(0),
        );
        if self
            .monitors
            .get(named)
            .is_some_and(|m| (m.source.width, m.source.height) != dimensions)
        {
            self.retire(named);
        }
        if !self.monitors.contains_key(named)
            && self.monitors.len() + self.windows.len() < MAX_SOURCES
            && let Some(feed) = self.source(
                named.to_owned(),
                SourceKind::Monitor,
                dimensions.0,
                dimensions.1,
            )
        {
            self.monitors.insert(named.to_owned(), feed);
        }
        let mut previous = std::mem::take(&mut self.windows);
        for root in server
            .mapped_surfaces()
            .take(MAX_SOURCES.saturating_sub(self.monitors.len()))
        {
            let geometry = crate::scene::geometry(root);
            let dimensions = (
                u32::try_from(geometry.size.w).unwrap_or(0),
                u32::try_from(geometry.size.h).unwrap_or(0),
            );
            let Some(mapping) = server.surfaces.window_visibility(root) else {
                continue;
            };
            if let Some(index) = previous.iter().position(|w| {
                w.root == *root
                    && Arc::ptr_eq(&w.mapping, &mapping)
                    && (w.feed.source.width, w.feed.source.height) == dimensions
            }) {
                self.windows.push(previous.remove(index));
            } else if let Some(feed) = self.source(
                server
                    .the_name_of(root)
                    .its_own_words()
                    .unwrap_or("")
                    .chars()
                    .filter(|c| !c.is_control())
                    .take(160)
                    .collect(),
                SourceKind::Window,
                dimensions.0,
                dimensions.1,
            ) {
                self.windows.push(Window {
                    root: root.clone(),
                    mapping,
                    feed,
                });
            }
        }
        for stale in previous {
            stale.feed.stop();
        }
        target.capture_output(self.monitors.get(named).is_some_and(|feed| feed.wanted()));
        self.endpoint.catalogue(
            self.monitors
                .values()
                .cloned()
                .chain(self.windows.iter().map(|w| w.feed.clone())),
        );
    }

    /// Publish only successful output frames, and render selected roots separately.
    pub(super) fn submitted(
        &self,
        server: &Server,
        named: &str,
        target: &mut (impl crate::FrameTarget + ?Sized),
    ) {
        if let Some(pixels) = target.captured_output()
            && let Some(feed) = self.monitors.get(named)
        {
            feed.publish(pixels);
        }
        for window in self.windows.iter().filter(|w| w.feed.wanted()) {
            let geometry = crate::scene::geometry(&window.root);
            let origin = crate::window_buffer_origin(&window.root) + geometry.loc.to_f64();
            let camera = alo_canvas::At::checked(origin.x.round() as i32, origin.y.round() as i32)
                .and_then(|at| alo_canvas::Camera::new().looking_at(at));
            let Some(camera) = camera else {
                window.feed.stop();
                continue;
            };
            let size = (
                window.feed.source.width as i32,
                window.feed.source.height as i32,
            )
                .into();
            // A window's private target contains no cursor tree owned by another app.
            match target.capture_window(
                size,
                &window.root,
                &server.popup_surfaces(),
                &crate::Cursor::Default,
                camera,
            ) {
                Ok(pixels) => window.feed.publish(pixels),
                Err(_) => window.feed.stop(),
            }
        }
    }

    /// End a display's stream before a same-named replacement can be catalogued.
    pub(super) fn retire(&mut self, named: &str) {
        if let Some(feed) = self.monitors.remove(named) {
            feed.stop();
        }
    }

    /// End every source before seat retirement or any exclusive screen is drawn.
    pub(super) fn stop(&mut self) {
        for feed in self
            .monitors
            .values()
            .chain(self.windows.iter().map(|w| &w.feed))
        {
            feed.stop();
        }
        self.monitors.clear();
        self.windows.clear();
        self.endpoint.catalogue(std::iter::empty());
    }
}

impl Server {
    /// Expose capture only after sign-in; a failed endpoint leaves the desktop usable.
    pub(super) fn enable_capture(&mut self) -> io::Result<()> {
        self.capture = Some(CaptureSources::bind(&self.socket.path)?);
        Ok(())
    }
}
