//! Drawing every display, and what happens when one refuses.

use super::*;
use crate::OutputMetadata;
use crate::direct_protocol_client as client;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Physical, Size};

/// A display that draws, or refuses at submission.
struct Screen {
    named: &'static str,
    size: (i32, i32),
    refuses: bool,
    /// What this display reports having put on a screen.
    ///
    /// Empty for the tests that only ask *was it drawn*; a real surface for
    /// the one that asks **which display's presentation sent the callback**,
    /// which cannot be answered without a surface to send it to.
    submits: Vec<WlSurface>,
}

impl FrameTarget for Screen {
    fn size(&self) -> Size<i32, Physical> {
        self.size.into()
    }
    fn submit(&mut self, _: &[WlSurface]) -> Result<Vec<WlSurface>, RenderError> {
        if self.refuses {
            return Err(RenderError::EmptySize);
        }
        Ok(self.submits.clone())
    }
    fn metadata(&self) -> Result<OutputMetadata, RenderError> {
        Ok(OutputMetadata {
            name: self.named.to_owned(),
            make: "alo".to_owned(),
            model: "a monitor".to_owned(),
            physical_size: (310, 170),
            refresh: 60_000,
        })
    }
}

/// A server on a private socket.
fn server(named: &str) -> (tempfile::TempDir, Server) {
    use std::os::unix::fs::PermissionsExt as _;
    let directory = tempfile::tempdir().expect("a private directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = Server::bind(directory.path(), named).expect("a bound socket");
    (directory, server)
}

/// **Both displays paint, and each is named in what came back.**
#[test]
fn every_display_is_drawn() {
    let (_directory, mut server) = server("every");
    let mut left = Screen {
        named: "DP-1",
        size: (1280, 720),
        refuses: false,
        submits: vec![],
    };
    let mut right = Screen {
        named: "HDMI-A-1",
        size: (1920, 1080),
        refuses: false,
        submits: vec![],
    };

    let became = server.render_each_display(&mut [&mut left, &mut right], 1);

    let named: Vec<&str> = became.drawn().iter().map(|(it, _)| it.as_str()).collect();
    assert_eq!(named, vec!["DP-1", "HDMI-A-1"], "a display was not drawn");
    assert!(became.refusals().is_empty());
    assert!(became.anything_drawn());
}

/// **A display that refuses does not stop the one after it**, which is the
/// acceptance's third clause and the whole reason this is not a `?` in a loop.
///
/// The refusing display is **first**, deliberately: a loop written with `?`
/// passes this test when the failure is last and fails it when the failure is
/// first, so putting it last would be a test that cannot catch the fault it
/// is named for.
#[test]
fn a_refusing_display_does_not_stop_the_next_one() {
    let (_directory, mut server) = server("refusing");
    let mut broken = Screen {
        named: "DP-1",
        size: (0, 0),
        refuses: true,
        submits: vec![],
    };
    let mut working = Screen {
        named: "HDMI-A-1",
        size: (1920, 1080),
        refuses: false,
        submits: vec![],
    };

    let became = server.render_each_display(&mut [&mut broken, &mut working], 1);

    assert!(
        became.anything_drawn(),
        "one broken display turned the working one off"
    );
    let drew: Vec<&str> = became.drawn().iter().map(|(it, _)| it.as_str()).collect();
    assert_eq!(drew, vec!["HDMI-A-1"]);
}

/// **And the refusal says which display made it.**
///
/// *A display failed* sends somebody to look at both. The name is the whole
/// difference between a diagnosis and a hunt.
#[test]
fn a_refusal_names_the_display_that_made_it() {
    let (_directory, mut server) = server("named");
    let mut broken = Screen {
        named: "DP-1",
        size: (0, 0),
        refuses: true,
        submits: vec![],
    };
    let mut working = Screen {
        named: "HDMI-A-1",
        size: (1920, 1080),
        refuses: false,
        submits: vec![],
    };

    let became = server.render_each_display(&mut [&mut broken, &mut working], 1);

    let refused: Vec<&str> = became
        .refusals()
        .iter()
        .map(|(it, _)| it.as_str())
        .collect();
    assert_eq!(
        refused,
        vec!["DP-1"],
        "the refusal did not name the display that made it"
    );
}

/// **Every display refusing is nothing drawn**, which is the one case a
/// caller has to treat as a session showing nothing.
#[test]
fn every_display_refusing_is_nothing_drawn() {
    let (_directory, mut server) = server("all-refuse");
    let mut one = Screen {
        named: "DP-1",
        size: (0, 0),
        refuses: true,
        submits: vec![],
    };
    let mut two = Screen {
        named: "HDMI-A-1",
        size: (0, 0),
        refuses: true,
        submits: vec![],
    };

    let became = server.render_each_display(&mut [&mut one, &mut two], 1);

    assert!(!became.anything_drawn());
    assert_eq!(became.refusals().len(), 2, "a refusal was lost");
}

/// **One display is the same answer it always was.**
///
/// Every caller today hands over one, and this must not have moved: a single
/// display drawing is `anything_drawn`, no refusals, and its own name.
#[test]
fn one_display_behaves_exactly_as_before() {
    let (_directory, mut server) = server("just-one");
    let mut only = Screen {
        named: "DP-1",
        size: (1280, 720),
        refuses: false,
        submits: vec![],
    };

    let became = server.render_each_display(&mut [&mut only], 1);

    assert!(became.anything_drawn());
    assert!(became.refusals().is_empty());
    assert_eq!(became.drawn().len(), 1);
}

/// A display that refuses to be disabled, and remembers being asked.
struct Dying {
    named: &'static str,
    refuses: bool,
    asked: bool,
}

impl FrameTarget for Dying {
    fn size(&self) -> Size<i32, Physical> {
        (1280, 720).into()
    }
    fn submit(&mut self, _: &[WlSurface]) -> Result<Vec<WlSurface>, RenderError> {
        Ok(vec![])
    }
    fn retire(&mut self) -> Result<(), RenderError> {
        self.asked = true;
        if self.refuses {
            return Err(RenderError::Closed);
        }
        Ok(())
    }
    fn metadata(&self) -> Result<OutputMetadata, RenderError> {
        Ok(OutputMetadata {
            name: self.named.to_owned(),
            make: "alo".to_owned(),
            model: "a monitor".to_owned(),
            physical_size: (310, 170),
            refresh: 60_000,
        })
    }
}

/// **A display that will not retire does not leave the next one scanning
/// out**, and the refusal names it.
///
/// The refusing display is **first** for the same reason it is first in the
/// drawing test: a `?` in a loop passes when the failure is last.
#[test]
fn a_display_refusing_to_retire_does_not_leave_the_next_one_lit() {
    let mut stuck = Dying {
        named: "DP-1",
        refuses: true,
        asked: false,
    };
    let mut ordinary = Dying {
        named: "HDMI-A-1",
        refuses: false,
        asked: false,
    };

    let refusals = crate::retire_each_display(&mut [&mut stuck, &mut ordinary]);

    assert!(
        ordinary.asked,
        "a display was left lit because an earlier one refused"
    );
    let named: Vec<&str> = refusals.iter().map(|(it, _)| it.as_str()).collect();
    assert_eq!(named, vec!["DP-1"]);
}

/// **Displays of different kinds go down one road.**
///
/// The desktop lane's own display is a `Layered` wrapper and the ones beyond
/// it are plain targets, so the list this road takes cannot be of one
/// concrete type. A road that compiled only for a uniform list would make
/// the second display's type the first display's business.
#[test]
fn displays_of_different_kinds_go_down_one_road() {
    let (_directory, mut server) = server("mixed");
    let mut wrapped = Screen {
        named: "DP-1",
        size: (1280, 720),
        refuses: false,
        submits: vec![],
    };
    let mut plain = Dying {
        named: "HDMI-A-1",
        refuses: false,
        asked: false,
    };
    let mut displays: Vec<&mut dyn FrameTarget> = vec![&mut wrapped, &mut plain];

    let became = server.render_each_display(&mut displays, 1);

    let drew: Vec<&str> = became.drawn().iter().map(|(it, _)| it.as_str()).collect();
    assert_eq!(drew, vec!["DP-1", "HDMI-A-1"]);
}

/// **A refusal can be taken, so a one-display caller may raise it whole.**
///
/// Not a convenience: [`RenderError`] is not `Clone`, so a caller that could
/// only borrow would have to rebuild the error and would lose its source.
#[test]
fn a_refusal_can_be_taken_so_a_caller_may_raise_it() {
    let (_directory, mut server) = server("taken");
    let mut only = Screen {
        named: "DP-1",
        size: (0, 0),
        refuses: true,
        submits: vec![],
    };

    let became = server.render_each_display(&mut [&mut only], 1);

    assert!(!became.anything_drawn());
    let mut taken = became.into_refusals();
    let (named, why) = taken.pop().expect("the refusal was lost");
    assert_eq!(named, "DP-1");
    assert!(matches!(why, RenderError::EmptySize));
}

/// **A client gets its frame callback from the display that drew it, and
/// from that display only.**
///
/// The acceptance's second clause, and the constraint underneath it:
/// `render_frame`'s standing promise — every client that was drawn gets its
/// callback — holds **per display**. A desktop that drew the displays around
/// `render_frame` would be a compositor whose clients slowly stopped drawing,
/// and nothing about the picture on either screen would say so.
///
/// Asymmetric on purpose: the second display reports submitting nothing, so
/// one callback is the right answer and two is the fault. A test where both
/// displays drew the surface could not tell a correct compositor from one
/// that sends a callback per display regardless of who drew.
#[test]
fn a_callback_comes_from_the_display_that_drew_and_from_no_other()
-> Result<(), Box<dyn std::error::Error>> {
    use std::{
        os::unix::net::UnixStream,
        sync::{Arc, mpsc},
        thread,
        time::{Duration, Instant},
    };

    let (_directory, mut server) = server("callbacks");
    let mut handle = server.display_handle();
    let (socket, peer) = UnixStream::pair()?;
    let client = handle.insert_client(socket, Arc::new(crate::surfaces::ClientState::default()))?;
    let (send, receive) = mpsc::channel();
    let (reply, responses) = mpsc::channel();
    let drawn_at = 90;
    let wire = thread::spawn(move || client::one_of_two_displays(peer, send, responses, drawn_at));

    let start = Instant::now();
    let mut stages = 0;
    while !wire.is_finished() {
        if start.elapsed() > Duration::from_secs(15) {
            return Err("two-display client deadline".into());
        }
        server.dispatch()?;
        if let Ok((_stage, id)) = receive.try_recv() {
            let root = client.object_from_protocol_id::<WlSurface>(&handle, id)?;
            // The first display reports this surface drawn; the second
            // reports nothing. Which display *could* have drawn it is not the
            // question — the question is who the callback comes from.
            let mut drew = Screen {
                named: "DP-1",
                size: (1280, 720),
                refuses: false,
                submits: vec![],
            };
            let mut drew_nothing = Screen {
                named: "HDMI-A-1",
                size: (1920, 1080),
                refuses: false,
                submits: vec![],
            };
            drew.submits = vec![root];
            let became = server.render_each_display(&mut [&mut drew, &mut drew_nothing], drawn_at);
            assert_eq!(
                became.drawn(),
                &[(String::from("DP-1"), 1), (String::from("HDMI-A-1"), 0)],
                "a submission was attributed to the wrong display"
            );
            reply.send(())?;
            stages += 1;
        }
        thread::sleep(Duration::from_millis(1));
    }
    wire.join()
        .map_err(|_| "two-display client assertions failed")??;
    assert_eq!(stages, 1);
    Ok(())
}
