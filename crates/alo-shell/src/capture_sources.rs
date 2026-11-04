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
            let dimensions = (extent(geometry.size.w), extent(geometry.size.h));
            let Some(mapping) = server.surfaces.window_visibility(root) else {
                continue;
            };
            if let Some(index) = previous.iter().position(|w| {
                w.root == *root
                    && Arc::ptr_eq(&w.mapping, &mapping)
                    && (w.feed.source.width, w.feed.source.height) == dimensions
            }) {
                let window = previous.remove(index);
                window.feed.named(
                    server
                        .the_name_of(root)
                        .its_own_words()
                        .unwrap_or("")
                        .chars()
                        .filter(|c| !c.is_control())
                        .take(160)
                        .collect(),
                );
                self.windows.push(window);
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
            // Embed only the compositor's arrow; no other app's cursor tree enters.
            let looking = if server.screens.is_some() { named } else { "" };
            let cursor = server
                .surfaces
                .pointer
                .as_ref()
                .and_then(|pointer| {
                    let at = server.surfaces.camera_of(looking).plane_of((
                        pointer.location.x.floor() as i32,
                        pointer.location.y.floor() as i32,
                    ))?;
                    let x = f64::from(at.x) - origin.x;
                    let y = f64::from(at.y) - origin.y;
                    (x >= 0.0
                        && y >= 0.0
                        && x < f64::from(window.feed.source.width)
                        && y < f64::from(window.feed.source.height))
                    .then_some(crate::Cursor::Arrow {
                        location: (x, y).into(),
                    })
                })
                .unwrap_or(crate::Cursor::Default);
            match target.capture_window(
                size,
                &window.root,
                &server.popup_surfaces(),
                &cursor,
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

/// Validate fractional logical geometry before rounding the private pixel extent.
fn extent(side: f64) -> u32 {
    if side.is_finite() && side > 0.0 && side <= 16_384.0 {
        side.ceil() as u32
    } else {
        0
    }
}

impl Server {
    /// Expose capture only after sign-in; a failed endpoint leaves the desktop usable.
    pub(super) fn enable_capture(&mut self) -> io::Result<()> {
        self.capture = Some(CaptureSources::bind(&self.socket.path)?);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    /// Fractional logical extents round outwards; non-finite or huge sizes refuse.
    #[test]
    fn private_capture_extents_are_bounded_before_allocation() {
        assert_eq!(super::extent(640.25), 641);
        for side in [0.0, -1.0, f64::NAN, f64::INFINITY, 16_384.01] {
            assert_eq!(super::extent(side), 0);
        }
    }
}
