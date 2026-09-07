//! Read-only DRM ioctl transport for direct-output discovery.

use crate::direct_output::{DirectOutputError, Port};
use drm::control::{Device, connector};
use std::os::fd::{AsFd, BorrowedFd};

/// Borrow a session descriptor without reopening its path or taking ownership.
#[derive(Clone)]
pub(crate) struct Inventory<'a>(pub BorrowedFd<'a>);

impl AsFd for Inventory<'_> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0
    }
}

impl drm::Device for Inventory<'_> {}
impl Device for Inventory<'_> {}

impl crate::direct_output::Inventory for Inventory<'_> {
    fn ports(&self) -> Result<Vec<Port>, DirectOutputError> {
        let resources = self
            .resource_handles()
            .map_err(|source| DirectOutputError::Query {
                stage: "resources",
                source,
            })?;
        // drm-rs shifts one bit per resource index when applying encoder masks.
        if resources.crtcs().len() > u32::BITS as usize {
            return Err(DirectOutputError::InvalidTopology);
        }
        let mut ports = Vec::new();
        for handle in resources.connectors() {
            let info =
                self.get_connector(*handle, false)
                    .map_err(|source| DirectOutputError::Query {
                        stage: "connector",
                        source,
                    })?;
            let connected = info.state() == connector::State::Connected;
            let display = info.interface() != connector::Interface::Writeback;
            let mut crtcs = Vec::new();
            if connected && display {
                for encoder in info.encoders() {
                    let encoder =
                        self.get_encoder(*encoder)
                            .map_err(|source| DirectOutputError::Query {
                                stage: "encoder",
                                source,
                            })?;
                    crtcs.extend(resources.filter_crtcs(encoder.possible_crtcs()));
                }
            }
            ports.push(Port {
                handle: *handle,
                connected,
                internal: matches!(
                    info.interface(),
                    connector::Interface::EmbeddedDisplayPort
                        | connector::Interface::LVDS
                        | connector::Interface::DSI
                ),
                display,
                modes: info.modes().to_vec(),
                crtcs,
            });
        }
        Ok(ports)
    }
}
