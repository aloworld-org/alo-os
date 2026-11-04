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
    ] {
        let action = gio::SimpleAction::new(name, None);
        let own = state.clone();
        action.connect_activate(move |_, _| match name {
            "new" => own.new_recording(),
            "library" => own.show_library(),
            _ => own.primary(),
        });
        application.add_action(&action);
        application.set_accels_for_action(&format!("app.{name}"), &keys);
    }
    let keyboard = gtk::EventControllerKey::new();
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

impl State {
    fn notice(&self, message: &str) {
        self.view.banner_text.set_label(message);
        self.view.banner.set_reveal_child(true);
    }
    fn release_capture(&self) {
        // The pipeline must let go before its PipeWire remote and session.
        self.engine.borrow_mut().take();
        self.capture.borrow_mut().take();
    }
    fn new_recording(self: &Rc<Self>) {
        if self.session.borrow().busy() {
            self.view.window.present();
            return;
        }
        self.release_capture();
        self.session.borrow_mut().reset();
        self.clip.borrow_mut().take();
        self.view
            .review
            .video
            .set_media_stream(gtk::MediaStream::NONE);
        self.view.microphone.set_active(false);
        self.view.preview.set_paintable(gtk::gdk::Paintable::NONE);
        self.view.placeholder.set_visible(true);
        self.view.banner.set_reveal_child(false);
        self.view.pages.set_visible_child_name("record");
        self.view.new_button.add_css_class("nav-active");
        self.view.library_button.remove_css_class("nav-active");
        self.render();
    }
    fn show_library(self: &Rc<Self>) {
        if self.session.borrow().busy()
            || matches!(
                self.session.borrow().phase(),
                Phase::Countdown(_) | Phase::Choosing
            )
        {
            return;
        }
        self.release_capture();
        self.session.borrow_mut().reset();
        self.view.microphone.set_active(false);
        self.refresh_library();
        self.view.pages.set_visible_child_name("library");
        self.view.new_button.remove_css_class("nav-active");
        self.view.library_button.add_css_class("nav-active");
    }
    fn primary(self: &Rc<Self>) {
        let phase = self.session.borrow().phase();
        match phase {
            Phase::Ready => self.choose(),
            Phase::Selected => self.preflight(),
            Phase::Recording | Phase::Paused => self.stop(),
            Phase::Review => self.new_recording(),
            _ => {}
        }
    }
    fn choose(self: &Rc<Self>) {
        if self.library.is_none() {
            return;
        }
        if let Err(error) = Recorder::available() {
            self.notice(&error.to_string());
            return;
        }
        if self.session.borrow_mut().choosing().is_err() {
            return;
        }
        self.view.banner.set_reveal_child(false);
        self.render();
        let own = self.clone();
        let kind = self.kind.get();
        glib::MainContext::default().spawn_local(async move {
            match crate::portal::choose(kind).await {
                Ok(capture) if !own.closed.get() => {
                    match Recorder::preview((&capture).into()) {
                        Ok(engine) => {
                            own.release_capture();
                            own.view.source_name.set_label(&format!("{} · {} × {}", t(if kind == SourceKind::Screen { "Selected screen" } else { "Selected window" }), capture.width, capture.height));
                            *own.engine.borrow_mut() = Some(engine);
                            *own.capture.borrow_mut() = Some(capture);
                            own.session.borrow_mut().selected();
                        }
                        Err(error) => { own.session.borrow_mut().reset(); own.notice(&error.to_string()); }
                    }
                }
                Ok(_) => {}
                Err(error) => {
                    eprintln!("Screen selection: {error}");
                    own.session.borrow_mut().reset();
                    own.view.microphone.set_active(false);
                    let message = match error {
                        ashpd::Error::Response(ashpd::desktop::ResponseError::Cancelled) => "Selection cancelled. Nothing was recorded.",
                        ashpd::Error::PortalNotFound(_) => "This desktop does not provide screen recording yet. Nothing was recorded.",
                        _ => "The selected source could not be opened. Check your desktop's screen-sharing permission, then try again. Nothing was recorded.",
                    };
                    own.notice(&t(message));
                }
            }
            own.render();
        });
    }
    fn selected_size(&self) -> Result<(i32, i32), String> {
        let quality = match self.view.quality.selected() {
            1 => Quality::Hd,
            2 => Quality::Original,
            _ => Quality::FullHd,
        };
        let size = self
            .engine
            .borrow()
            .as_ref()
            .and_then(Recorder::source_size)
            .ok_or_else(|| t("Please wait for the live preview, then try again."))?;
        quality.size(size.0, size.1).map_err(t)
    }
    fn preflight(self: &Rc<Self>) {
        let (width, height) = match self.selected_size() {
            Ok(size) => size,
            Err(error) => {
                self.notice(&error);
                return;
            }
        };
        if self.session.borrow_mut().checking().is_err() {
            return;
        }
        self.render();
        let (sending, receiving) = std::sync::mpsc::sync_channel(1);
        let worker = std::thread::Builder::new()
            .name("studio-quality-check".to_owned())
            .spawn(move || {
                let _ = sending.send(crate::performance::check(width, height));
            });
        if let Err(error) = worker {
            self.session.borrow_mut().selected();
            self.notice(&error.to_string());
            self.render();
            return;
        }
        let weak = Rc::downgrade(self);
        glib::timeout_add_local(Duration::from_millis(50), move || {
            let Some(own) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            if own.closed.get() || own.session.borrow().phase() != Phase::Checking {
                return glib::ControlFlow::Break;
            }
            match receiving.try_recv() {
                Ok(result) => {
                    own.session.borrow_mut().selected();
                    match result {
                        Ok(()) => own.countdown(),
                        Err(error) => {
                            own.notice(&error.to_string());
                            own.render();
                        }
                    }
                    glib::ControlFlow::Break
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    own.session.borrow_mut().selected();
                    own.notice(&t(
                        "Recording quality could not be checked. Please try again.",
                    ));
                    own.render();
                    glib::ControlFlow::Break
                }
            }
        });
    }
    fn countdown(self: &Rc<Self>) {
        let seconds = match self.view.countdown.selected() {
            0 => 3,
            1 => 5,
            _ => 0,
        };
        if self.session.borrow_mut().countdown(seconds).is_err() {
            return;
        }
        self.render();
        if seconds == 0 {
            self.start();
            return;
        }
        let weak = Rc::downgrade(self);
        glib::timeout_add_local(Duration::from_secs(1), move || {
            let Some(own) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            if !matches!(own.session.borrow().phase(), Phase::Countdown(_)) {
                return glib::ControlFlow::Break;
            }
            let start = own.session.borrow_mut().tick().unwrap_or(false);
            if start {
                own.start();
                glib::ControlFlow::Break
            } else {
                own.render();
                glib::ControlFlow::Continue
            }
        });
    }
    fn cancel_countdown(&self) {
        let _ = self.session.borrow_mut().cancel_countdown();
        self.render();
    }
    fn start(self: &Rc<Self>) {
        let size = self.selected_size();
        let result = (|| {
            let capture = self.capture.borrow();
            let capture = capture
                .as_ref()
                .ok_or_else(|| t("Please choose your screen again."))?;
            let (width, height) = size?;
            let library = self
                .library
                .as_ref()
                .ok_or_else(|| t("Your recording library is unavailable."))?;
            let microphone = self.session.borrow().microphone();
            let mut clip = library
                .begin(width, height, microphone)
                .map_err(|error| error.to_string())?;
            clip.title = format!("{} {}", t("Recording"), date(clip.created));
            let path = library
                .pending(&clip.id)
                .map_err(|error| error.to_string())?;
            self.engine.borrow_mut().take();
            let engine = Recorder::record(capture.into(), &path, width, height, microphone)
                .map_err(|error| error.to_string())?;
            *self.engine.borrow_mut() = Some(engine);
            *self.clip.borrow_mut() = Some(clip);
            self.session
                .borrow_mut()
                .started(Instant::now())
                .map_err(|error| error.to_string())?;
            Ok::<(), String>(())
        })();
        match result {
            Ok(()) => {
                self.show_bar();
                self.view.window.minimize();
            }
            Err(error) => {
                self.release_capture();
                self.session.borrow_mut().reset();
                self.view.microphone.set_active(false);
                self.notice(&error);
            }
        }
        self.render();
    }
    fn pause(&self) {
        let phase = self.session.borrow().phase();
        let result = self.engine.borrow().as_ref().map(|engine| {
            if phase == Phase::Paused {
                engine.resume()
            } else {
                engine.pause()
            }
        });
        match result {
            Some(Ok(())) => {
                let mut session = self.session.borrow_mut();
                let _ = if phase == Phase::Paused {
                    session.resume(Instant::now())
                } else {
                    session.pause(Instant::now())
                };
            }
            Some(Err(error)) => self.notice(&error.to_string()),
            None => {}
        }
        self.render();
    }
    fn stop(&self) {
        if !matches!(
            self.session.borrow().phase(),
            Phase::Recording | Phase::Paused
        ) {
            return;
        }
        let result = self.engine.borrow_mut().as_mut().map(Recorder::finish);
        match result {
            Some(Ok(())) => {
                let _ = self.session.borrow_mut().finishing(Instant::now());
                self.finish_started.set(Some(Instant::now()));
            }
            Some(Err(error)) => self.failed(&error.to_string()),
            None => self.failed(&t("The recording engine has stopped.")),
        }
        self.render();
    }
    fn poll(self: &Rc<Self>) {
        let frame = self.engine.borrow().as_ref().and_then(Recorder::frame);
        if let Some(frame) = frame {
            let texture = gtk::gdk::MemoryTexture::new(
                frame.width,
                frame.height,
                gtk::gdk::MemoryFormat::R8g8b8a8,
                &glib::Bytes::from_owned(frame.bytes),
                frame.width as usize * 4,
            );
            self.view.preview.set_paintable(Some(&texture));
            self.view.placeholder.set_visible(false);
            if let (Some(clip), Some(library)) = (self.clip.borrow().as_ref(), &self.library)
                && let Ok(path) = library.thumbnail(&clip.id)
                && !path.exists()
                && let Err(error) = texture.save_to_png(&path)
            {
                eprintln!("Saving thumbnail: {error}");
            }
        }
        let event = self.engine.borrow().as_ref().and_then(Recorder::poll);
        match event {
            Some(Event::Finished) if self.session.borrow().busy() => self.completed(),
            Some(Event::Finished) => self.failed(&t("The selected source stopped sharing. Please select it again.")),
            Some(Event::Failed(error)) => self.failed(&format!("{} {error}", t("Recording stopped because the source or audio device could not continue. Any temporary video has been kept."))),
            Some(Event::FellBehind) => {
                self.notice(&t("This device could not keep up. Recording has stopped and the video captured so far is being saved. Try 720p for the next recording."));
                self.stop();
            }
            None => {}
        }
        if let Some(started) = self.finish_started.get()
            && started.elapsed() > Duration::from_secs(20)
        {
            self.failed(&t("Finishing took too long. The temporary video has been kept in your recording library folder."));
        }
        let time = clock(self.session.borrow().elapsed_at(Instant::now()));
        self.view.timer.set_label(&time);
        if let Some((_, timer, _)) = self.bar.borrow().as_ref() {
            timer.set_label(&time);
        }
    }
    fn completed(self: &Rc<Self>) {
        self.finish_started.set(None);
        self.release_capture();
        if self.session.borrow().phase() != Phase::Finishing {
            let _ = self.session.borrow_mut().finishing(Instant::now());
        }
        let result = (|| {
            let mut clip = self.clip.borrow_mut();
            let clip = clip
                .as_mut()
                .ok_or_else(|| t("No recording was returned."))?;
            clip.duration_ms = self
                .session
                .borrow()
                .elapsed_at(Instant::now())
                .as_millis()
                .min(u128::from(u64::MAX)) as u64;
            let library = self
                .library
                .as_ref()
                .ok_or_else(|| t("Your recording library is unavailable."))?;
            library.finish(clip).map_err(|error| error.to_string())?;
            Ok::<Clip, String>(clip.clone())
        })();
        match result {
            Ok(clip) => {
                let _ = self.session.borrow_mut().finished();
                self.view.microphone.set_active(false);
                self.close_bar();
                self.open_clip(&clip);
                self.refresh_library();
                if self.close_after_save.get() { self.view.window.close(); } else { self.view.window.present(); }
            }
            Err(error) => self.failed(&format!("{} {error}", t("The video could not be added to your library. Its temporary file has been kept."))),
        }
        self.render();
    }
    fn failed(&self, message: &str) {
        self.finish_started.set(None);
        self.release_capture();
        self.session.borrow_mut().reset();
        self.view.microphone.set_active(false);
        self.close_bar();
        self.view.window.present();
        self.notice(message);
        self.render();
    }
    fn render(&self) {
        let phase = self.session.borrow().phase();
        let changing = matches!(phase, Phase::Ready | Phase::Selected | Phase::Review);
        for widget in [
            &self.view.screen.clone().upcast::<gtk::Widget>(),
            &self.view.one_window.clone().upcast(),
            &self.view.microphone.clone().upcast(),
            &self.view.quality.clone().upcast(),
            &self.view.countdown.clone().upcast(),
        ] {
            widget.set_sensitive(changing);
        }
        self.view.new_button.set_sensitive(changing);
        self.view.library_button.set_sensitive(changing);
        self.view
            .pause
            .set_visible(matches!(phase, Phase::Recording | Phase::Paused));
        self.view.pause.set_label(&t(if phase == Phase::Paused {
            "Resume"
        } else {
            "Pause"
        }));
        self.view
            .cancel
            .set_visible(matches!(phase, Phase::Countdown(_)));
        self.view
            .countdown_number
            .set_visible(matches!(phase, Phase::Countdown(number) if number > 0));
        if let Phase::Countdown(number) = phase {
            self.view.countdown_number.set_label(&number.to_string());
        }
        let (button, status) = match phase {
            Phase::Ready => (
                if self.kind.get() == SourceKind::Screen {
                    "Choose a screen"
                } else {
                    "Choose a window"
                },
                "●  Nothing is recording",
            ),
            Phase::Choosing => ("Choosing…", "Waiting for your selection"),
            Phase::Selected => ("Start recording", "●  Preview only · not recording"),
            Phase::Checking => (
                "Checking…",
                "Checking that this recording size will run smoothly…",
            ),
            Phase::Countdown(_) => (
                "Starting…",
                "Get ready. Recording starts after the countdown.",
            ),
            Phase::Recording => ("Stop recording", "●  Recording"),
            Phase::Paused => ("Stop recording", "Ⅱ  Paused · microphone paused too"),
            Phase::Finishing => ("Finishing…", "Saving your video safely…"),
            Phase::Review => ("New recording", "✓  Saved to your library"),
        };
        self.view.record.set_label(&t(button));
        self.view.record.set_sensitive(
            self.library.is_some()
                && !matches!(
                    phase,
                    Phase::Choosing | Phase::Checking | Phase::Countdown(_) | Phase::Finishing
                ),
        );
        self.view.status.set_label(&t(status));
        if phase == Phase::Recording {
            self.view.status.add_css_class("recording");
        } else {
            self.view.status.remove_css_class("recording");
        }
        if matches!(phase, Phase::Recording | Phase::Paused) {
            self.view.record.add_css_class("recording-stop");
        } else {
            self.view.record.remove_css_class("recording-stop");
        }
        if let Some((_, _, pause)) = self.bar.borrow().as_ref() {
            pause.set_label(&t(if phase == Phase::Paused {
                "Resume"
            } else {
                "Pause"
            }));
            pause.set_sensitive(phase != Phase::Finishing);
        }
    }
    fn show_bar(self: &Rc<Self>) {
        let Some(application) = self.view.window.application() else {
            return;
        };
        let bar = gtk::ApplicationWindow::builder()
            .application(&application)
            .title(t("alo Studio — Recording"))
            .resizable(false)
            .default_width(360)
            .build();
        let controls = gtk::Box::new(gtk::Orientation::Horizontal, 14);
        controls.add_css_class("control-bar");
        let timer = label("00:00:00", "timer");
        timer.set_hexpand(true);
        controls.append(&timer);
        let pause = gtk::Button::with_label(&t("Pause"));
        let stop = gtk::Button::with_label(&t("Finish"));
        stop.add_css_class("primary");
        stop.add_css_class("recording-stop");
        controls.append(&pause);
        controls.append(&stop);
        bar.set_child(Some(&controls));
        let own = self.clone();
        pause.connect_clicked(move |_| own.pause());
        let own = self.clone();
        stop.connect_clicked(move |_| own.stop());
        let own = self.clone();
        bar.connect_close_request(move |_| {
            if own.session.borrow().busy() {
                own.stop();
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        bar.present();
        *self.bar.borrow_mut() = Some((bar, timer, pause));
    }
    fn close_bar(&self) {
        if let Some((bar, _, _)) = self.bar.borrow_mut().take() {
            bar.close();
        }
    }
    fn open_clip(&self, clip: &Clip) {
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
            "{}  ·  {} × {}  ·  VP9 / WebM  ·  {}",
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
    fn refresh_library(self: &Rc<Self>) {
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
    fn export(self: &Rc<Self>) {
        exports::save(self.clone());
    }
    fn delete(self: &Rc<Self>) {
        exports::delete(self.clone());
    }
}

fn date(seconds: u64) -> String {
    glib::DateTime::from_unix_local(seconds.min(i64::MAX as u64) as i64)
        .and_then(|date| date.format("%b %e, %H:%M"))
        .map(|text| text.to_string())
        .unwrap_or_else(|_| seconds.to_string())
}

#[path = "exports.rs"]
mod exports;
