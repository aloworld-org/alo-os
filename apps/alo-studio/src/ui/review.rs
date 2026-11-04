//! Previewing, naming, and saving a completed recording.

use super::layout::{icon_button, label};
use crate::words::t;
use gtk::prelude::*;

#[derive(Clone)]
pub struct Review {
    pub root: gtk::ScrolledWindow,
    pub video: gtk::Video,
    pub title: gtk::Entry,
    pub details: gtk::Label,
    pub save: gtk::Button,
    pub another: gtk::Button,
    pub delete: gtk::Button,
}

impl Review {
    pub fn build() -> Self {
        let content = gtk::Box::new(gtk::Orientation::Vertical, 18);
        content.add_css_class("workspace");
        content.append(&label("SAVED TO YOUR LIBRARY", "eyebrow"));
        content.append(&label("That's a wrap.", "page-title"));
        content.append(&label(
            "Give it a name. Watch it back. Share it when you're ready.",
            "page-subtitle",
        ));
        let video = gtk::Video::builder().autoplay(false).hexpand(true).build();
        video.add_css_class("review-video");
        let frame = gtk::AspectFrame::builder()
            .ratio(16.0 / 9.0)
            .obey_child(false)
            .child(&video)
            .build();
        content.append(&frame);
        let title = gtk::Entry::builder()
            .placeholder_text(t("Name your recording"))
            .max_length(120)
            .build();
        title.add_css_class("clip-title");
        title.update_property(&[gtk::accessible::Property::Label(&t("Recording title"))]);
        content.append(&title);
        let details = label("", "muted");
        content.append(&details);
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        let save = gtk::Button::with_label(&t("Save a copy…"));
        save.add_css_class("primary");
        let another = gtk::Button::with_label(&t("Record another"));
        row.append(&save);
        row.append(&another);
        let space = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        space.set_hexpand(true);
        row.append(&space);
        let delete = icon_button("user-trash-symbolic", "Delete this recording");
        delete.add_css_class("flat");
        row.append(&delete);
        content.append(&row);
        let note = label(
            "The original stays in your library. Saving a copy never uploads it.",
            "footnote",
        );
        note.set_wrap(true);
        content.append(&note);
        let root = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&content)
            .build();
        Self {
            root,
            video,
            title,
            details,
            save,
            another,
            delete,
        }
    }
}
