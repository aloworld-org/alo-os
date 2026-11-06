//! Browsing, naming, and opening recordings already saved.

use super::*;

impl State {
    pub(super) fn open_clip(&self, clip: &Clip) {
        let Some(library) = &self.library else {
            return;
        };
        self.release_capture();
        self.session.borrow_mut().reviewing();
        if let Ok(video) = library.video(&clip.id) {
            self.view
                .review
                .video
                .set_file(Some(&gio::File::for_path(video)));
        }
        *self.clip.borrow_mut() = Some(clip.clone());
        self.view.review.title.set_text(&clip.title);
        self.view.review.details.set_label(&format!(
            "{}  ·  {} × {}  ·  WebM  ·  {}",
            clock(clip.duration()),
            clip.width,
            clip.height,
            t(if clip.microphone {
                "With microphone"
            } else {
                "Without microphone"
            })
        ));
        self.view.pages.set_visible_child_name("review");
    }
    pub(super) fn refresh_library(self: &Rc<Self>) {
        while let Some(child) = self.view.library_list.first_child() {
            self.view.library_list.remove(&child);
        }
        let Some(library) = &self.library else {
            return;
        };
        let (clips, warnings) = match library.clips() {
            Ok(found) => found,
            Err(error) => {
                self.notice(&error.to_string());
                return;
            }
        };
        for warning in warnings {
            eprintln!("Recording library: {warning}");
        }
        self.view.library_count.set_label(&format!(
            "{} {}",
            clips.len(),
            t(if clips.len() == 1 {
                "recording"
            } else {
                "recordings"
            })
        ));
        let query = self.view.search.text().to_lowercase();
        let mut shown = 0;
        for clip in clips
            .into_iter()
            .filter(|clip| clip.title.to_lowercase().contains(&query))
        {
            shown += 1;
            let row = gtk::ListBoxRow::new();
            let content = gtk::Box::new(gtk::Orientation::Horizontal, 16);
            content.add_css_class("clip-row");
            let picture = gtk::Picture::builder()
                .width_request(100)
                .height_request(60)
                .content_fit(gtk::ContentFit::Cover)
                .build();
            if let Ok(path) = library.thumbnail(&clip.id) {
                picture.set_filename(Some(path));
            }
            content.append(&picture);
            let copy = gtk::Box::new(gtk::Orientation::Vertical, 4);
            copy.set_hexpand(true);
            let title = gtk::Label::new(Some(&clip.title));
            title.set_xalign(0.0);
            title.set_ellipsize(gtk::pango::EllipsizeMode::End);
            title.add_css_class("card-title");
            copy.append(&title);
            copy.append(&label(
                &format!(
                    "{} · {} · {} × {}",
                    date(clip.created),
                    clock(clip.duration()),
                    clip.width,
                    clip.height
                ),
                "muted",
            ));
            content.append(&copy);
            let play = gtk::Button::builder()
                .icon_name("media-playback-start-symbolic")
                .tooltip_text(t("Open recording"))
                .build();
            let own = self.clone();
            play.update_property(&[gtk::accessible::Property::Label(&format!(
                "{}: {}",
                t("Open recording"),
                clip.title
            ))]);
            play.connect_clicked(move |_| own.open_clip(&clip));
            content.append(&play);
            row.set_child(Some(&content));
            self.view.library_list.append(&row);
        }
        if shown == 0 {
            let empty = gtk::Box::new(gtk::Orientation::Vertical, 12);
            empty.add_css_class("empty-library");
            empty.append(&label(
                if query.is_empty() {
                    "A blank page. A clear beginning."
                } else {
                    "No recordings match that name."
                },
                "card-title",
            ));
            empty.append(&label(
                if query.is_empty() {
                    "Your first recording will feel right at home here."
                } else {
                    "Try another word or clear the search."
                },
                "muted",
            ));
            self.view.library_list.append(&empty);
        }
    }
    pub(super) fn export(self: &Rc<Self>) {
        exports::save(self.clone());
    }
    pub(super) fn delete(self: &Rc<Self>) {
        exports::delete(self.clone());
    }
}
