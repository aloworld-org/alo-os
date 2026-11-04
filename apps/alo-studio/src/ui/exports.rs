//! Native asynchronous export and deliberate removal of a completed video.

use super::*;
pub(super) fn save(own: Rc<State>) {
    let Some(clip) = own.clip.borrow().clone() else {
        return;
    };
    let Some(library) = &own.library else {
        return;
    };
    let Ok(path) = library.video(&clip.id) else {
        return;
    };
    let name = format!(
        "{}.webm",
        clip.title
            .chars()
            .filter(|character| !character.is_control() && !matches!(character, '/' | '\\'))
            .take(100)
            .collect::<String>()
    );
    glib::MainContext::default().spawn_local(async move {
        let filter = gtk::FileFilter::new();
        filter.set_name(Some(&t("WebM video")));
        filter.add_mime_type("video/webm");
        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);
        let dialog = gtk::FileDialog::builder()
            .title(t("Save your recording"))
            .initial_name(name)
            .filters(&filters)
            .default_filter(&filter)
            .build();
        let destination = match dialog.save_future(Some(&own.view.window)).await {
            Ok(destination) => destination,
            Err(error) if error.matches(gtk::DialogError::Dismissed) || error.matches(gtk::DialogError::Cancelled) => return,
            Err(_) => {
                own.notice(&t("The save window could not open. Your recording is safe in your library. Please try again."));
                return;
            },
        };
        own.view.review.save.set_sensitive(false);
        let source = gio::File::for_path(path);
        let callback = own.clone();
        // A large export continues if the person closes the last window.
        let hold = own
            .view
            .window
            .application()
            .map(|application| application.hold());
        source.copy_async(
            &destination,
            gio::FileCopyFlags::OVERWRITE,
            glib::Priority::DEFAULT,
            gio::Cancellable::NONE,
            None,
            move |result| {
                let _keep_running = hold;
                callback.view.review.save.set_sensitive(true);
                match result {
                    Ok(()) => callback.notice(&t(
                        "Your copy is saved. The original is still in your library.",
                    )),
                    Err(error) => callback.notice(&format!(
                        "{} {error}",
                        t("The copy could not be saved. Your original is safe in the library.")
                    )),
                }
            },
        );
    });
}
pub(super) fn delete(own: Rc<State>) {
    let Some(clip) = own.clip.borrow().clone() else {
        return;
    };
    glib::MainContext::default().spawn_local(async move {
        let answer = gtk::AlertDialog::builder().message(t("Delete this recording?"))
            .detail(t("It will be removed from your library. Any copies you saved elsewhere stay where they are."))
            .buttons([t("Keep video"), t("Delete")]).cancel_button(0).default_button(0).build()
            .choose_future(Some(&own.view.window)).await;
        if answer != Ok(1) { return; }
        own.view.review.video.set_media_stream(gtk::MediaStream::NONE);
        if let Some(library) = &own.library {
            match library.remove(&clip.id) {
                Ok(()) => { own.clip.borrow_mut().take(); own.show_library(); },
                Err(error) => own.notice(&format!("{} {error}", t("The recording could not be removed."))),
            }
        }
    });
}
