//! Join deliberate UI actions to permission, capture, finalization, and saving.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

use gtk::{gio, glib, prelude::*};

use super::layout::{View, label};
use crate::model::{Phase, Quality, Session, SourceKind, clock};
use crate::portal::Capture;
use crate::recorder::{Event, Recorder};
use crate::storage::{Clip, Library};
use crate::words::t;

struct State {
    view: View,
    session: RefCell<Session>,
    capture: RefCell<Option<Capture>>,
    engine: RefCell<Option<Recorder>>,
    clip: RefCell<Option<Clip>>,
    library: Option<Library>,
    kind: Cell<SourceKind>,
    finish_started: Cell<Option<Instant>>,
    close_after_save: Cell<bool>,
    closed: Cell<bool>,
    bar: RefCell<Option<(gtk::ApplicationWindow, gtk::Label, gtk::Button)>>,
}

pub fn open(application: &gtk::Application) {
    let view = super::layout::build(application);
    let library = Library::at(glib::user_data_dir().join("alo-studio").join("recordings"));
    let library_error = library.as_ref().err().map(ToString::to_string);
    let state = Rc::new(State {
        view,
        session: RefCell::new(Session::default()),
        capture: RefCell::new(None),
        engine: RefCell::new(None),
        clip: RefCell::new(None),
        library: library.ok(),
        kind: Cell::new(SourceKind::Screen),
        finish_started: Cell::new(None),
        close_after_save: Cell::new(false),
        closed: Cell::new(false),
        bar: RefCell::new(None),
    });
    connect(&state, application);
    state.refresh_library();
    state.render();
    if let Some(error) = library_error {
        state.notice(&format!("{} {error}", t("Your recording library could not be opened. Recording is unavailable until the folder is writable.")));
    } else if let Err(error) = Recorder::available() {
        state.notice(&error.to_string());
    }
    state.view.window.present();
    let weak = Rc::downgrade(&state);
    glib::timeout_add_local(Duration::from_millis(80), move || {
        let Some(state) = weak.upgrade() else {
            return glib::ControlFlow::Break;
        };
        if state.closed.get() {
            return glib::ControlFlow::Break;
        }
        state.poll();
        glib::ControlFlow::Continue
    });
}

fn connect(state: &Rc<State>, application: &gtk::Application) {
    let own = state.clone();
    state.view.window.connect_visible_notify(move |window| {
        if let Some(engine) = own.engine.borrow().as_ref() {
            engine.set_preview_visible(window.is_visible());
        }
    });
    let own = state.clone();
    state.view.record.connect_clicked(move |_| own.primary());
    let own = state.clone();
    state.view.pause.connect_clicked(move |_| own.pause());
    let own = state.clone();
    state
        .view
        .cancel
        .connect_clicked(move |_| own.cancel_countdown());
    let own = state.clone();
    state
        .view
        .new_button
        .connect_clicked(move |_| own.new_recording());
    let own = state.clone();
    state
        .view
        .review
        .another
        .connect_clicked(move |_| own.new_recording());
    let own = state.clone();
    state
        .view
        .library_button
        .connect_clicked(move |_| own.show_library());
    let own = state.clone();
    state
        .view
        .search
        .connect_search_changed(move |_| own.refresh_library());
    for (button, kind) in [
        (&state.view.screen, SourceKind::Screen),
        (&state.view.one_window, SourceKind::Window),
    ] {
        let own = state.clone();
        button.connect_toggled(move |button| {
            if button.is_active() && own.kind.get() != kind {
                own.kind.set(kind);
                own.release_capture();
                own.session.borrow_mut().reset();
                own.view.microphone.set_active(false);
                own.render();
            }
        });
    }
    let own = state.clone();
    state.view.microphone.connect_active_notify(move |switch| {
        let _ = own.session.borrow_mut().set_microphone(switch.is_active());
        own.view
            .microphone_text
            .set_label(&t(if switch.is_active() {
                "Microphone on. People near you may also be recorded."
            } else {
                "Microphone off. A quiet recording, by default."
            }));
    });
    let own = state.clone();
    state
        .view
        .review
        .save
        .connect_clicked(move |_| own.export());
    let own = state.clone();
    state
        .view
        .review
        .delete
        .connect_clicked(move |_| own.delete());
    let own = state.clone();
    state.view.review.title.connect_changed(move |entry| {
        let Some(library) = &own.library else {
            return;
        };
        if let Some(clip) = own.clip.borrow_mut().as_mut() {
            let title = entry.text().trim().to_owned();
            if !title.is_empty() && title != clip.title {
                clip.title = title;
                if let Err(error) = library.update(clip) {
                    own.notice(&format!(
                        "{} {error}",
                        t("The new name could not be saved.")
                    ));
                }
            }
        }
    });
    let own = state.clone();
    state.view.help_button.connect_clicked(move |_| {
        gtk::AlertDialog::builder().message(t("A few helpful shortcuts"))
            .detail(t("Ctrl + N    New recording\nCtrl + L    Your recordings\nCtrl + Shift + R    Start or stop\nSpace    Pause or resume while recording\nEscape    Cancel the countdown\n\nShortcuts work while alo Studio has the keyboard."))
            .buttons([t("Got it")]).build().show(Some(&own.view.window));
    });
    for (name, keys) in [
        ("new", vec!["<Primary>n"]),
        ("library", vec!["<Primary>l"]),
        ("record", vec!["<Primary><Shift>r"]),
        ("pause", vec![]),
    ] {
        let action = gio::SimpleAction::new(name, None);
        let own = state.clone();
        action.connect_activate(move |_, _| match name {
            "new" => own.new_recording(),
            "library" => own.show_library(),
            "pause" => own.pause(),
            _ => own.primary(),
        });
        application.add_action(&action);
        application.set_accels_for_action(&format!("app.{name}"), &keys);
    }
    let keyboard = gtk::EventControllerKey::new();
    // Consume the recording shortcut before a focused button also handles Space.
    keyboard.set_propagation_phase(gtk::PropagationPhase::Capture);
    let own = state.clone();
    keyboard.connect_key_pressed(move |_, key, _, _| {
        let phase = own.session.borrow().phase();
        if key == gtk::gdk::Key::space && matches!(phase, Phase::Recording | Phase::Paused) {
            own.pause();
            return glib::Propagation::Stop;
        }
        if key == gtk::gdk::Key::Escape && matches!(phase, Phase::Countdown(_)) {
            own.cancel_countdown();
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    state.view.window.add_controller(keyboard);
    let own = state.clone();
    state.view.window.connect_close_request(move |_| {
        if own.session.borrow().busy() {
            let pending = own.clone();
            glib::MainContext::default().spawn_local(async move {
                let answer = gtk::AlertDialog::builder()
                    .message(t("Finish this recording before closing?"))
                    .detail(t(
                        "Your video will be saved to your library. Nothing is uploaded.",
                    ))
                    .buttons([t("Keep recording"), t("Finish and close")])
                    .cancel_button(0)
                    .default_button(0)
                    .build()
                    .choose_future(Some(&pending.view.window))
                    .await;
                if answer == Ok(1) {
                    pending.close_after_save.set(true);
                    pending.stop();
                }
            });
            return glib::Propagation::Stop;
        }
        own.closed.set(true);
        own.release_capture();
        glib::Propagation::Proceed
    });
    let own = state.clone();
    application.connect_shutdown(move |_| {
        own.closed.set(true);
        own.release_capture();
    });
}

fn date(seconds: u64) -> String {
    glib::DateTime::from_unix_local(seconds.min(i64::MAX as u64) as i64)
        .and_then(|date| date.format("%b %e, %H:%M"))
        .map(|text| text.to_string())
        .unwrap_or_else(|_| seconds.to_string())
}

#[path = "exports.rs"]
mod exports;

#[path = "completion.rs"]
mod completion;

#[path = "lifecycle.rs"]
mod lifecycle;

#[path = "presentation.rs"]
mod presentation;

#[path = "library.rs"]
mod library;
