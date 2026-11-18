//! Verify completed media off the UI thread, then commit its library entry.

use super::*;

impl State {
    pub(super) fn completed(self: &Rc<Self>) {
        self.finish_started.set(None);
        let has_video = self
            .engine
            .borrow()
            .as_ref()
            .and_then(Recorder::recorded_duration)
            .is_some();
        self.release_capture();
        if !has_video {
            self.failed(&t(
                "No video frames were recorded. Please choose your source again.",
            ));
            return;
        }
        if self.session.borrow().phase() != Phase::Finishing {
            let _ = self.session.borrow_mut().finishing(Instant::now());
        }
        let path = self
            .clip
            .borrow()
            .as_ref()
            .zip(self.library.as_ref())
            .and_then(|(clip, library)| library.pending(&clip.id).ok());
        let Some(path) = path else {
            self.completion_failed(&t("No recording was returned."));
            return;
        };
        let (sending, receiving) = futures_channel::oneshot::channel();
        let worker = std::thread::Builder::new()
            .name("studio-media-check".to_owned())
            .spawn(move || {
                let _ = sending.send(crate::completed_media::duration(&path));
            });
        if let Err(error) = worker {
            self.completion_failed(&error.to_string());
            return;
        }
        let weak = Rc::downgrade(self);
        glib::MainContext::default().spawn_local(async move {
            let result = receiving.await;
            let Some(own) = weak.upgrade().filter(|own| !own.closed.get()) else {
                return;
            };
            match result {
                Ok(Ok(duration)) => own.commit_recording(duration),
                Ok(Err(error)) => own.completion_failed(&error.to_string()),
                Err(_) => own.completion_failed(&t("The saved video could not be verified.")),
            }
        });
        self.render();
    }

    fn commit_recording(self: &Rc<Self>, duration: Duration) {
        let result = (|| {
            let mut clip = self.clip.borrow_mut();
            let clip = clip
                .as_mut()
                .ok_or_else(|| t("No recording was returned."))?;
            clip.duration_ms = duration.as_millis().min(u128::from(u64::MAX)) as u64;
            self.library
                .as_ref()
                .ok_or_else(|| t("Your recording library is unavailable."))?
                .finish(clip)
                .map_err(|error| error.to_string())?;
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
            Err(error) => self.completion_failed(&error),
        }
        self.render();
    }

    fn completion_failed(&self, error: &str) {
        self.failed(&format!(
            "{} {error}",
            t("The video could not be added to your library. Its temporary file has been kept.")
        ));
    }
}
