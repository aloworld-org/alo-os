//! A screen is selected by the system picker, never enumerated behind it.

use std::os::fd::{AsRawFd, OwnedFd, RawFd};

use ashpd::desktop::{
    PersistMode, Session,
    screencast::{CursorMode, Screencast, SelectSourcesOptions, SourceType},
};

use crate::model::SourceKind;

/// A granted source. Its remote and permission session outlive every frame read.
pub struct Capture {
    session: Option<Session<Screencast>>,
    remote: OwnedFd,
    /// The one selected stream on the permission-scoped PipeWire remote.
    pub node: u32,
    /// Reported source width; stream negotiation handles a portal that omits it.
    pub width: i32,
    /// Reported source height.
    pub height: i32,
}

impl Capture {
    /// The descriptor is borrowed, not transferred to the encoder.
    pub fn fd(&self) -> RawFd {
        self.remote.as_raw_fd()
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        if let Some(session) = self.session.take() {
            gtk::glib::MainContext::default().spawn_local(async move {
                if let Err(error) = session.close().await {
                    eprintln!("Closing the screen session: {error}");
                }
            });
        }
    }
}

/// Ask anew for every recording. No restore token or ambient screen permission.
pub async fn choose(kind: SourceKind) -> Result<Capture, ashpd::Error> {
    let portal = Screencast::new().await?;
    let session = portal.create_session(Default::default()).await?;
    let source = match kind {
        SourceKind::Screen => SourceType::Monitor,
        SourceKind::Window => SourceType::Window,
    };
    // Waiting for SelectSources' response is essential: Start may not race it.
    portal
        .select_sources(
            &session,
            SelectSourcesOptions::default()
                .set_sources(Some(enumflags2::BitFlags::from(source)))
                .set_multiple(false)
                .set_cursor_mode(CursorMode::Embedded)
                .set_persist_mode(PersistMode::DoNot),
        )
        .await?
        .response()?;
    let answer = portal
        .start(&session, None, Default::default())
        .await?
        .response()?;
    let stream = answer
        .streams()
        .first()
        .ok_or_else(|| ashpd::Error::NoResponse)?;
    let (width, height) = stream.size().unwrap_or((1920, 1080));
    let remote = portal
        .open_pipe_wire_remote(&session, Default::default())
        .await?;
    Ok(Capture {
        session: Some(session),
        remote,
        node: stream.pipe_wire_node_id(),
        width,
        height,
    })
}
