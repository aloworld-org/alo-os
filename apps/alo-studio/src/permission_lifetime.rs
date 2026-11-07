//! Observe permission closure before starting either the preview or microphone.

use ashpd::desktop::{Session, screencast::Screencast};
use futures_util::StreamExt;
use gtk::glib;
use std::{cell::Cell, rc::Rc};

/// The session and its already-subscribed closure listener share one lifetime.
pub(crate) struct Permission {
    session: Rc<Session<Screencast>>,
    ended: Rc<Cell<bool>>,
    listener: glib::JoinHandle<()>,
}

impl Permission {
    pub(crate) async fn watch(session: Session<Screencast>) -> Result<Self, ashpd::Error> {
        let session = Rc::new(session);
        let ended = Rc::new(Cell::new(false));
        let watching = session.clone();
        let closed = ended.clone();
        let (subscribed, ready) = futures_channel::oneshot::channel();
        let listener = glib::MainContext::default().spawn_local(async move {
            match watching.receive_closed().await {
                Ok(signals) => {
                    let _ = subscribed.send(Ok(()));
                    futures_util::pin_mut!(signals);
                    let _ = signals.next().await;
                    closed.set(true);
                }
                Err(why) => {
                    closed.set(true);
                    let _ = subscribed.send(Err(why));
                }
            }
        });
        let permission = Self {
            session,
            ended,
            listener,
        };
        ready.await.map_err(|_| ashpd::Error::NoResponse)??;
        Ok(permission)
    }
    pub(crate) fn session(&self) -> &Session<Screencast> {
        &self.session
    }
    pub(crate) fn ended(&self) -> bool {
        self.ended.get()
    }
}

impl Drop for Permission {
    fn drop(&mut self) {
        self.listener.abort();
        let session = self.session.clone();
        glib::MainContext::default().spawn_local(async move {
            if let Err(why) = session.close().await {
                eprintln!("Closing the screen session: {why}");
            }
        });
    }
}
