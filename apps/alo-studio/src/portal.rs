//! A screen is selected by the system picker, never enumerated behind it.

use std::os::fd::{AsRawFd, OwnedFd, RawFd};

use ashpd::desktop::{
    PersistMode,
    screencast::{CursorMode, Screencast, SelectSourcesOptions, SourceType},
};

use crate::model::SourceKind;

/// A granted source. Its remote and permission session outlive every frame read.
pub struct Capture {
    #[cfg(feature = "verification")]
    pub(crate) verification_display: Option<String>,
    permission: Option<crate::permission_lifetime::Permission>,
    remote: OwnedFd,
    /// The one selected stream on the permission-scoped PipeWire remote.
    pub node: u32,
    /// Reported source width; stream negotiation handles a portal that omits it.
    pub width: i32,
    /// Reported source height.
    pub height: i32,
}

impl Capture {
    /// A closed source permission stops all live inputs before finalization.
    pub fn ended(&self) -> bool {
        self.permission
            .as_ref()
            .is_some_and(|permission| permission.ended())
    }
    /// The descriptor is borrowed, not transferred to the encoder.
    pub fn fd(&self) -> RawFd {
        self.remote.as_raw_fd()
    }
}

/// Ask anew for every recording. No restore token or ambient screen permission.
pub async fn choose(
    kind: SourceKind,
    window: Option<&gtk::Window>,
) -> Result<Capture, ashpd::Error> {
    // Only explicit verification builds can select the isolated Xvfb fixture.
    // This branch and its environment variable do not exist in a Flatpak build.
    #[cfg(feature = "verification")]
    if let Ok(display) = std::env::var("ALO_STUDIO_VERIFY_DISPLAY") {
        let remote = std::fs::File::open("/dev/null")
            .map_err(ashpd::Error::IO)?
            .into();
        return Ok(Capture {
            permission: None,
            remote,
            node: 0,
            width: 1280,
            height: 960,
            verification_display: Some(display),
        });
    }
    let parent = match window {
        Some(window) => crate::parent_window::Parent::export(window).await,
        None => None,
    };
    let portal = Screencast::new().await?;
    let session = portal.create_session(Default::default()).await?;
    let permission = crate::permission_lifetime::Permission::watch(session).await?;
    let session = permission.session();
    let source = match kind {
        SourceKind::Screen => SourceType::Monitor,
        SourceKind::Window => SourceType::Window,
    };
    let granted = async {
        // Waiting for SelectSources' response is essential: Start may not race it.
        portal
            .select_sources(
                session,
                SelectSourcesOptions::default()
                    .set_sources(Some(enumflags2::BitFlags::from(source)))
                    .set_multiple(false)
                    .set_cursor_mode(CursorMode::Embedded)
                    .set_persist_mode(PersistMode::DoNot),
            )
            .await?
            .response()?;
        let answer = portal
            .start(
                session,
                parent.as_ref().map(|parent| parent.identifier()),
                Default::default(),
            )
            .await?
            .response()?;
        let stream = answer
            .streams()
            .first()
            .ok_or_else(|| ashpd::Error::NoResponse)?;
        // Unknown is not Full HD. The negotiated preview caps supply the actual size.
        let (width, height) = stream.size().unwrap_or((0, 0));
        let remote = portal
            .open_pipe_wire_remote(session, Default::default())
            .await?;
        Ok::<_, ashpd::Error>((remote, stream.pipe_wire_node_id(), width, height))
    }
    .await;
    let (remote, node, width, height) = match granted {
        Ok(granted) => granted,
        Err(error) => {
            return Err(error);
        }
    };
    Ok(Capture {
        #[cfg(feature = "verification")]
        verification_display: None,
        permission: Some(permission),
        remote,
        node,
        width,
        height,
    })
}
