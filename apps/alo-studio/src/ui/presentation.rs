//! Navigation and the visible state of recording controls.

use super::*;

impl State {
    pub(super) fn notice(&self, message: &str) {
        self.view.banner_text.set_label(message);
        self.view.banner.set_reveal_child(true);
    }
    pub(super) fn new_recording(self: &Rc<Self>) {
        if self.session.borrow().busy()
            || matches!(
                self.session.borrow().phase(),
                Phase::Choosing | Phase::Checking | Phase::Countdown(_)
            )
        {
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
    pub(super) fn show_library(self: &Rc<Self>) {
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
    pub(super) fn render(&self) {
        let phase = self.session.borrow().phase();
        if phase == Phase::Ready {
            self.view.source_name.set_label(&t(
                "You choose exactly what can be seen in the system picker.",
            ));
        }
        let changing = matches!(phase, Phase::Ready | Phase::Selected | Phase::Review);
        for widget in [
            &self.view.screen.clone().upcast::<gtk::Widget>(),
            &self.view.one_window.clone().upcast(),
            &self.view.microphone.clone().upcast(),
            &self.view.quality.widget.clone().upcast(),
            &self.view.countdown.widget.clone().upcast(),
        ] {
            widget.set_sensitive(changing);
        }
        self.view.new_button.set_sensitive(changing);
        self.view.library_button.set_sensitive(changing);
        self.view.pause.set_visible(matches!(
            phase,
            Phase::Preparing | Phase::Recording | Phase::Paused
        ));
        self.view
            .pause
            .set_sensitive(matches!(phase, Phase::Recording | Phase::Paused));
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
            Phase::Preparing => ("Stop recording", "Preparing your recording…"),
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
            pause.set_sensitive(matches!(phase, Phase::Recording | Phase::Paused));
        }
    }
    pub(super) fn show_bar(self: &Rc<Self>) {
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
        let timer = label("Starting…", "timer");
        timer.set_hexpand(true);
        controls.append(&timer);
        let pause = gtk::Button::with_label(&t("Pause"));
        pause.set_sensitive(self.session.borrow().phase() != Phase::Preparing);
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
        let workspace = self.view.window.downgrade();
        bar.connect_map(move |_| {
            if let Some(workspace) = workspace.upgrade() {
                workspace.set_visible(false);
            }
        });
        let keyboard = gtk::EventControllerKey::new();
        keyboard.set_propagation_phase(gtk::PropagationPhase::Capture);
        let own = self.clone();
        keyboard.connect_key_pressed(move |_, key, _, _| {
            if key == gtk::gdk::Key::space
                && matches!(
                    own.session.borrow().phase(),
                    Phase::Recording | Phase::Paused
                )
            {
                own.pause();
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        bar.add_controller(keyboard);
        bar.present();
        *self.bar.borrow_mut() = Some((bar, timer, pause));
    }
    pub(super) fn close_bar(&self) {
        if let Some((bar, _, _)) = self.bar.borrow_mut().take() {
            bar.close();
        }
    }
    /// Keep the controls available until the review workspace has regained focus.
    pub(super) fn show_workspace(&self) {
        self.view.window.present();
        let Some((bar, _, _)) = self.bar.borrow_mut().take() else {
            return;
        };
        let workspace = self.view.window.clone();
        let waiting = Instant::now();
        glib::timeout_add_local(Duration::from_millis(25), move || {
            if workspace.is_active() || waiting.elapsed() > Duration::from_secs(5) {
                bar.close();
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
    }
}
