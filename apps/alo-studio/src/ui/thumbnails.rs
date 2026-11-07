//! Load library previews only when shown, without blocking recorder controls.

use gtk::{gio, glib, prelude::*};
use std::{cell::RefCell, path::PathBuf, rc::Rc};

pub(super) fn load_when_shown(picture: &gtk::Picture, path: PathBuf) {
    let pending = Rc::new(RefCell::new(None::<gio::Cancellable>));
    let cancelled = pending.clone();
    picture.connect_unmap(move |_| {
        if let Some(task) = cancelled.borrow_mut().take() {
            task.cancel();
        }
    });
    picture.connect_map(move |picture| {
        if picture.paintable().is_some() || pending.borrow().is_some() {
            return;
        }
        let cancel = gio::Cancellable::new();
        *pending.borrow_mut() = Some(cancel.clone());
        let weak = picture.downgrade();
        let waiting = pending.clone();
        let token = cancel.clone();
        gio::File::for_path(&path).read_async(glib::Priority::LOW, Some(&cancel), move |opened| {
            let Ok(stream) = opened else {
                clear(&waiting, &token);
                return;
            };
            if token.is_cancelled() {
                clear(&waiting, &token);
                return;
            }
            let finishing = token.clone();
            gtk::gdk_pixbuf::Pixbuf::from_stream_at_scale_async(
                &stream,
                100,
                60,
                true,
                Some(&token),
                move |decoded| {
                    clear(&waiting, &finishing);
                    if finishing.is_cancelled() {
                        return;
                    }
                    if let (Some(picture), Ok(pixbuf)) = (weak.upgrade(), decoded)
                        && picture.is_mapped()
                    {
                        picture.set_paintable(Some(&gtk::gdk::Texture::for_pixbuf(&pixbuf)));
                    }
                },
            );
        });
    });
}

fn clear(pending: &RefCell<Option<gio::Cancellable>>, token: &gio::Cancellable) {
    let mut pending = pending.borrow_mut();
    if pending.as_ref() == Some(token) {
        pending.take();
    }
}
