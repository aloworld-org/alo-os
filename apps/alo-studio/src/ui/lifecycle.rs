//! Permission-scoped capture, recording, and completion.

use super::*;

impl State {
    pub(super) fn release_capture(&self) {
        // The pipeline must let go before its PipeWire remote and session.
        self.engine.borrow_mut().take();
        self.capture.borrow_mut().take();
    }
    pub(super) fn primary(self: &Rc<Self>) {
        let phase = self.session.borrow().phase();
        match phase {
            Phase::Ready => self.choose(),
            Phase::Selected => self.preflight(),
            Phase::Recording | Phase::Paused => self.stop(),
            Phase::Review => self.new_recording(),
            _ => {}
        }
    }
    pub(super) fn choose(self: &Rc<Self>) {
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
            match crate::portal::choose(kind, Some(own.view.window.upcast_ref())).await {
                Ok(capture) if !own.closed.get() => {
                    match Recorder::preview((&capture).into()) {
                        Ok(engine) => {
                            own.release_capture();
                            own.view.source_name.set_label(&t("Your source is selected. Preparing the live preview…"));
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
    pub(super) fn selected_size(&self) -> Result<(i32, i32), String> {
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
    pub(super) fn preflight(self: &Rc<Self>) {
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
                    match result {
                        Ok(crate::performance::Assessment::Ready) => {
                            own.session.borrow_mut().selected();
                            own.countdown();
                        }
                        Ok(crate::performance::Assessment::Demanding) => own.confirm_quality(),
                        Err(error) => {
                            own.session.borrow_mut().selected();
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
    fn confirm_quality(self: &Rc<Self>) {
        let own = self.clone();
        glib::MainContext::default().spawn_local(async move {
            let answer = gtk::AlertDialog::builder()
                .message(t("Recording may be slower at this size"))
                .detail(t("The short quality check ran slowly. You can choose a smaller size or continue with your selection. If the encoder falls behind, alo Studio will stop and save the video it has recorded."))
                .buttons([t("Choose another size"), t("Record anyway")])
                .cancel_button(0).default_button(0).build()
                .choose_future(Some(&own.view.window)).await;
            if own.closed.get() || own.session.borrow().phase() != Phase::Checking { return; }
            own.session.borrow_mut().selected();
            if answer == Ok(1) { own.countdown(); } else { own.render(); }
        });
    }
    pub(super) fn countdown(self: &Rc<Self>) {
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
    pub(super) fn cancel_countdown(&self) {
        let _ = self.session.borrow_mut().cancel_countdown();
        self.render();
    }
    pub(super) fn start(self: &Rc<Self>) {
        let size = self.selected_size();
        let result = (|| {
            if self.capture.borrow().is_none() {
                return Err(t("Please choose your screen again."));
            }
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
            // Keep the selected source connected; only the encoding branch starts.
            self.engine
                .borrow_mut()
                .as_mut()
                .ok_or_else(|| t("Please select your source again."))?
                .start_recording(&path, width, height, microphone)
                .map_err(|error| error.to_string())?;
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
            }
            Err(error) => {
                eprintln!("Starting recording: {error}");
                self.release_capture();
                self.session.borrow_mut().reset();
                self.view.microphone.set_active(false);
                self.notice(&t("Recording could not start. Please select your source again. Any temporary video has been kept."));
            }
        }
        self.render();
    }
    pub(super) fn pause(&self) {
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
    pub(super) fn stop(&self) {
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
    pub(super) fn poll(self: &Rc<Self>) {
        if self.capture.borrow().as_ref().is_some_and(Capture::ended)
            && !matches!(
                self.session.borrow().phase(),
                Phase::Finishing | Phase::Review | Phase::Ready
            )
        {
            self.source_ended();
        }
        let frame = self.engine.borrow().as_ref().and_then(Recorder::frame);
        if let Some(frame) = frame {
            let first_frame = self.view.placeholder.is_visible();
            if let Some((width, height)) = self
                .engine
                .borrow()
                .as_ref()
                .and_then(Recorder::source_size)
            {
                self.view.source_name.set_label(&format!(
                    "{} · {width} × {height}",
                    t(if self.kind.get() == SourceKind::Screen {
                        "Selected screen"
                    } else {
                        "Selected window"
                    })
                ));
            }
            let texture = gtk::gdk::MemoryTexture::new(
                frame.width,
                frame.height,
                gtk::gdk::MemoryFormat::R8g8b8a8,
                &glib::Bytes::from_owned(frame.bytes),
                frame.width as usize * 4,
            );
            self.view.preview.set_paintable(Some(&texture));
            self.view.placeholder.set_visible(false);
            if first_frame {
                let preview = self.view.preview.clone();
                let pixels = (frame.width, frame.height);
                glib::timeout_add_local_once(Duration::from_secs(1), move || {
                    eprintln!(
                        "Studio preview: {} x {} pixels, display area {} x {}",
                        pixels.0,
                        pixels.1,
                        preview.width(),
                        preview.height()
                    );
                });
            }
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
            Some(Event::SourceEnded(why)) => {
                eprintln!("The live recording input ended: {why}");
                self.source_ended();
            }
            Some(Event::Finished) if self.session.borrow().busy() => self.completed(),
            Some(Event::Finished) => self.failed(&t(
                "The selected source stopped sharing. Please select it again.",
            )),
            Some(Event::Failed(error)) => {
                eprintln!("Recording media error: {error}");
                let message = if self.session.borrow().busy() {
                    "Recording stopped because the source or audio device could not continue. Any temporary video has been kept."
                } else {
                    "The selected source could not be previewed. Please choose it again. Nothing was recorded."
                };
                self.failed(&t(message));
            }
            Some(Event::FellBehind) => {
                self.notice(&t("This device could not keep up with recording. The video captured so far is being saved. Try recording a smaller window or closing other applications."));
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
    /// A lost permission or source seals queued video and stops microphone input.
    fn source_ended(self: &Rc<Self>) {
        let phase = self.session.borrow().phase();
        if phase == Phase::Finishing {
            return;
        }
        if !matches!(phase, Phase::Recording | Phase::Paused) {
            self.failed(&t(
                "Screen sharing ended. Please select a screen or window again.",
            ));
            return;
        }
        let result = self
            .engine
            .borrow_mut()
            .as_mut()
            .map(Recorder::finish_after_source_ended);
        match result {
            Some(Ok(())) => {
                let _ = self.session.borrow_mut().finishing(Instant::now());
                self.finish_started.set(Some(Instant::now()));
                self.view.microphone.set_active(false);
                self.notice(&t("Screen sharing ended. Your recording is being saved."));
                self.render();
            }
            Some(Err(why)) => self.failed(&why.to_string()),
            None => self.failed(&t("The recording engine has stopped.")),
        }
    }
    pub(super) fn completed(self: &Rc<Self>) {
        self.finish_started.set(None);
        let recorded_duration = self
            .engine
            .borrow()
            .as_ref()
            .and_then(Recorder::recorded_duration);
        self.release_capture();
        if self.session.borrow().phase() != Phase::Finishing {
            let _ = self.session.borrow_mut().finishing(Instant::now());
        }
        let result = (|| {
            let mut clip = self.clip.borrow_mut();
            let clip = clip
                .as_mut()
                .ok_or_else(|| t("No recording was returned."))?;
            clip.duration_ms = recorded_duration
                .ok_or_else(|| {
                    t("No video frames were recorded. Please choose your source again.")
                })?
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
                if self.close_after_save.get() {
                    self.close_bar();
                    self.view.window.close();
                } else {
                    self.open_clip(&clip);
                    self.refresh_library();
                    self.show_workspace();
                }
            }
            Err(error) => self.failed(&format!("{} {error}", t("The video could not be added to your library. Its temporary file has been kept."))),
        }
        self.render();
    }
    pub(super) fn failed(&self, message: &str) {
        self.close_after_save.set(false);
        self.finish_started.set(None);
        self.release_capture();
        self.session.borrow_mut().reset();
        self.view.microphone.set_active(false);
        self.show_workspace();
        self.notice(message);
        self.render();
    }
}
