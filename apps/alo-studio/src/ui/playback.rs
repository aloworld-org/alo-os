//! Persistent keyboard and reader controls for the selected saved recording.

use std::cell::RefCell;
use std::rc::Rc;

use crate::words::t;
use gtk::prelude::*;

struct Playback {
    video: gtk::glib::WeakRef<gtk::Video>,
    button: gtk::Button,
    position: gtk::Scale,
    time: gtk::Label,
    error: gtk::Label,
    watching: RefCell<Option<(gtk::MediaStream, gtk::glib::SignalHandlerId)>>,
}

/// Keep playback reachable without first moving a pointer over the video.
pub(super) fn controls(video: &gtk::Video) -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 8);
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    row.add_css_class("playback-controls");
    let button = gtk::Button::with_label(&t("Play recording"));
    button.set_width_request(150);
    let position = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 1.0);
    position.set_draw_value(false);
    position.set_hexpand(true);
    position.update_property(&[
        gtk::accessible::Property::Label(&t("Playback position")),
        gtk::accessible::Property::Description(&t("Seconds into the recording")),
    ]);
    let time = gtk::Label::new(None);
    time.add_css_class("playback-time");
    time.update_property(&[gtk::accessible::Property::Label(&t("Playback time"))]);
    row.append(&button);
    row.append(&position);
    row.append(&time);
    root.append(&row);
    let error = super::layout::label("This recording could not be played.", "muted");
    error.set_wrap(true);
    error.set_visible(false);
    root.append(&error);
    let own = Rc::new(Playback {
        video: video.downgrade(),
        button,
        position,
        time,
        error,
        watching: RefCell::new(None),
    });
    let weak = Rc::downgrade(&own);
    own.button.connect_clicked(move |_| {
        if let Some(own) = weak.upgrade() {
            own.toggle();
        }
    });
    let weak = Rc::downgrade(&own);
    own.position.connect_change_value(move |_, _, seconds| {
        if let Some(own) = weak.upgrade() {
            own.seek(seconds);
        }
        gtk::glib::Propagation::Stop
    });
    own.follow(video);
    video.connect_media_stream_notify(move |video| own.follow(video));
    root
}

impl Playback {
    fn stream(&self) -> Option<gtk::MediaStream> {
        self.video.upgrade().and_then(|video| video.media_stream())
    }

    fn follow(self: &Rc<Self>, video: &gtk::Video) {
        let stream = video.media_stream();
        if self.watching.borrow().as_ref().map(|(old, _)| old) == stream.as_ref() {
            self.refresh();
            return;
        }
        if let Some((old, signal)) = self.watching.borrow_mut().take() {
            old.disconnect(signal);
            old.pause();
        }
        if let Some(stream) = stream {
            let weak = Rc::downgrade(self);
            let signal = stream.connect_notify_local(None, move |_, _| {
                if let Some(own) = weak.upgrade() {
                    own.refresh();
                }
            });
            *self.watching.borrow_mut() = Some((stream, signal));
        }
        self.refresh();
    }

    fn refresh(&self) {
        let stream = self.stream();
        let failed = stream
            .as_ref()
            .is_some_and(|stream| stream.error().is_some());
        let prepared = !failed && stream.as_ref().is_some_and(|stream| stream.is_prepared());
        let playing = prepared
            && stream
                .as_ref()
                .is_some_and(|stream| stream.is_playing() && !stream.is_ended());
        let seekable = prepared && stream.as_ref().is_some_and(|stream| stream.is_seekable());
        let duration = stream
            .as_ref()
            .filter(|_| prepared)
            .map_or(0, |stream| stream.duration().max(0));
        let timestamp = stream
            .as_ref()
            .filter(|_| prepared)
            .map_or(0, |stream| stream.timestamp().clamp(0, duration));
        self.button.set_label(&t(if playing {
            "Pause playback"
        } else {
            "Play recording"
        }));
        self.button.set_sensitive(
            prepared && (seekable || !stream.as_ref().is_some_and(|stream| stream.is_ended())),
        );
        self.position.set_sensitive(seekable && duration > 0);
        self.position.set_range(0.0, duration as f64 / 1_000_000.0);
        self.position.set_value(timestamp as f64 / 1_000_000.0);
        self.time
            .set_label(&format!("{} / {}", clock(timestamp), clock(duration)));
        self.error.set_visible(failed);
    }

    fn toggle(&self) {
        let Some(stream) = self
            .stream()
            .filter(|stream| stream.is_prepared() && stream.error().is_none())
        else {
            return;
        };
        if stream.is_playing() && !stream.is_ended() {
            stream.pause();
        } else {
            if stream.is_ended() {
                if !stream.is_seekable() {
                    return;
                }
                stream.seek(0);
            }
            stream.play();
        }
    }

    fn seek(&self, seconds: f64) {
        let Some(stream) = self.stream().filter(|stream| {
            stream.is_prepared() && stream.error().is_none() && stream.is_seekable()
        }) else {
            return;
        };
        if seconds.is_finite() {
            let duration = stream.duration().max(0);
            stream.seek(
                (seconds.clamp(0.0, duration as f64 / 1_000_000.0) * 1_000_000.0).round() as i64,
            );
        }
    }
}

fn clock(microseconds: i64) -> String {
    let seconds = microseconds.max(0) / 1_000_000;
    if seconds >= 3600 {
        format!(
            "{}:{:02}:{:02}",
            seconds / 3600,
            seconds / 60 % 60,
            seconds % 60
        )
    } else {
        format!("{}:{:02}", seconds / 60, seconds % 60)
    }
}
